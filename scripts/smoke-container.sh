#!/usr/bin/env bash
set -euo pipefail

IMAGE="${IMAGE:-graphs-and-jev:test}"
NAME="graphs-and-jev-smoke-$$"
PORT="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')"
BASE_URL="http://127.0.0.1:${PORT}"

[[ -f Dockerfile ]] || { echo "Dockerfile is required" >&2; exit 1; }

cleanup() {
  docker rm -f "$NAME" >/dev/null 2>&1 || true
}
trap cleanup EXIT

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
