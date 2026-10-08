#!/usr/bin/env bash
set -euo pipefail

IMAGE="${IMAGE:-graphs-and-jev:test}"
NAME="graphs-and-jev-smoke-$$"
PORT="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')"
BASE_URL="http://127.0.0.1:${PORT}"
ROOTFS="$(mktemp -d)"
CONTAINER_ID=""

cleanup() {
  docker rm -f "$NAME" "$CONTAINER_ID" >/dev/null 2>&1 || true
  rm -rf "$ROOTFS"
}
trap cleanup EXIT

[[ -f Dockerfile ]] || { echo "Dockerfile is required" >&2; exit 1; }

image_user="$(docker image inspect --format '{{.Config.User}}' "$IMAGE")"
[[ "$image_user" == "65532" || "$image_user" == "65532:65532" ]] || {
  echo "image must run as nonroot UID 65532, got: $image_user" >&2
  exit 1
}

CONTAINER_ID="$(docker create --platform linux/amd64 "$IMAGE")"
docker export "$CONTAINER_ID" >"$ROOTFS/rootfs.tar"
tar -tf "$ROOTFS/rootfs.tar" >"$ROOTFS/files"
for file in app/serve app/catalogue.json app/static/index.html; do
  grep -qx "$file" "$ROOTFS/files" || { echo "missing runtime file: /$file" >&2; exit 1; }
done
if grep -Eq '(^|/)(bin/)?(ba)?sh$' "$ROOTFS/files"; then
  echo "runtime unexpectedly contains a shell" >&2
  exit 1
fi
tar -xf "$ROOTFS/rootfs.tar" -C "$ROOTFS" app/serve app/catalogue.json app/static/index.html
[[ -x "$ROOTFS/app/serve" && -r "$ROOTFS/app/catalogue.json" && -r "$ROOTFS/app/static/index.html" ]] || {
  echo "runtime files are not readable/executable by the inspection user" >&2
  exit 1
}

docker run --platform linux/amd64 --detach --name "$NAME" \
  --publish "127.0.0.1:${PORT}:8080" "$IMAGE" >/dev/null

python3 - "$BASE_URL" <<'PYTHON'
import json
import sys
import time
import urllib.error
import urllib.request

base_url = sys.argv[1]

def request(path, body=None):
    data = json.dumps(body).encode() if body is not None else None
    headers = {"Content-Type": "application/json"} if body is not None else {}
    with urllib.request.urlopen(
        urllib.request.Request(base_url + path, data=data, headers=headers), timeout=2
    ) as response:
        return response.status, response.read()

last_error = None
for _ in range(60):
    try:
        status, health = request("/api/health")
        if status == 200 and json.loads(health)["ok"] is True:
            break
    except (OSError, urllib.error.URLError, json.JSONDecodeError, KeyError) as error:
        last_error = error
    time.sleep(0.25)
else:
    raise SystemExit(f"container did not become healthy: {last_error}")

status, ready = request("/api/ready")
assert status == 200 and json.loads(ready)["ok"] is True, ready
status, catalogue = request("/api/catalogue")
assert status == 200 and json.loads(catalogue), catalogue
status, decision = request(
    "/api/decide",
    {
        "backend": "catalogue",
        "state": {"request": "annual CO2 for Australia in 2024"},
        "questions": {
            "gate": {
                "type": "choice",
                "criteria": {
                    "answer": "Answer from bounded evidence.",
                    "clarify": "Ask for missing detail.",
                    "reject": "Reject unsupported scope.",
                },
            }
        },
    },
)
decision = json.loads(decision)
assert status == 200 and decision["answers"]["gate"]["top"] == "answer", decision
status, page = request("/")
assert status == 200 and b"Graphs + Jev" in page, "root page missing publication title"
status, answer = request(
    "/api/pipeline",
    {"backend": "catalogue", "request": "annual CO2 for Australia in 2024"},
)
answer = json.loads(answer)
assert status == 200 and answer["outcome"] == "answer", answer
print(f"container smoke passed at {base_url}")
PYTHON
