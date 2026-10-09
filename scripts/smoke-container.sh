#!/usr/bin/env bash
set -euo pipefail

IMAGE="${IMAGE:-graphs-and-jev:test}"
ELF_INSPECTOR="rust:1.97-bookworm@sha256:0e2bcaef56d041a486784e54104a81aebe0da44bd03019bd70bc0401e42e4a97"
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

docker image save "$IMAGE" -o "$ROOTFS/image.tar"
mkdir "$ROOTFS/image"
tar -xf "$ROOTFS/image.tar" -C "$ROOTFS/image"
python3 - "$ROOTFS/image" >"$ROOTFS/files" <<'PYTHON'
import json
import pathlib
import sys
import tarfile

image = pathlib.Path(sys.argv[1])
manifest = json.loads((image / "manifest.json").read_text())[0]
files = set()
for layer in manifest["Layers"]:
    with tarfile.open(image / layer) as archive:
        for member in archive.getmembers():
            name = member.name.removeprefix("./").rstrip("/")
            if name and not pathlib.PurePosixPath(name).name.startswith(".wh."):
                files.add(name)
print("\n".join(sorted(files)))
PYTHON
awk '!/^(app|app\/serve|app\/catalogue.json|app\/static($|\/))/ { print "unexpected scratch rootfs entry: /" $0; bad=1 } END { exit bad }' "$ROOTFS/files"
for file in app/serve app/catalogue.json app/static/index.html; do
  grep -qx "$file" "$ROOTFS/files" || { echo "missing runtime file: /$file" >&2; exit 1; }
done
CONTAINER_ID="$(docker create --platform linux/amd64 "$IMAGE")"
docker export "$CONTAINER_ID" >"$ROOTFS/rootfs.tar"
tar -xf "$ROOTFS/rootfs.tar" -C "$ROOTFS" app/serve app/catalogue.json app/static/index.html
[[ -x "$ROOTFS/app/serve" && -r "$ROOTFS/app/catalogue.json" && -r "$ROOTFS/app/static/index.html" ]] || {
  echo "runtime files are not readable/executable by the inspection user" >&2
  exit 1
}
file "$ROOTFS/app/serve" | grep -Eq 'ELF 64-bit.*x86-64' || {
  file "$ROOTFS/app/serve" >&2
  echo "runtime binary must be a static x86-64 ELF" >&2
  exit 1
}
if command -v readelf >/dev/null; then
  ! readelf -l "$ROOTFS/app/serve" | grep -q 'INTERP' || {
    echo "runtime binary has an ELF interpreter" >&2
    exit 1
  }
  ! readelf -d "$ROOTFS/app/serve" | grep -q 'NEEDED' || {
    echo "runtime binary has dynamic library dependencies" >&2
    exit 1
  }
elif command -v ldd >/dev/null; then
  ldd "$ROOTFS/app/serve" 2>&1 | grep -Eq 'not a dynamic executable|statically linked' || {
    echo "runtime binary has dynamic dependencies" >&2
    exit 1
  }
else
  docker run --rm --platform linux/amd64 --entrypoint readelf \
    --volume "$ROOTFS/app/serve:/inspect/serve:ro" "$ELF_INSPECTOR" -l /inspect/serve \
    | grep -q 'INTERP' && { echo "runtime binary has an ELF interpreter" >&2; exit 1; }
  docker run --rm --platform linux/amd64 --entrypoint readelf \
    --volume "$ROOTFS/app/serve:/inspect/serve:ro" "$ELF_INSPECTOR" -d /inspect/serve \
    | grep -q 'NEEDED' && { echo "runtime binary has dynamic library dependencies" >&2; exit 1; }
fi

docker run --platform linux/amd64 --detach --name "$NAME" \
  --publish "127.0.0.1:${PORT}:8080" "$IMAGE" >/dev/null

python3 - "$BASE_URL" <<'PYTHON'
import json
import sys
import time
import urllib.error
import urllib.request

base_url = sys.argv[1]

def request(path, body=None, timeout=2):
    data = json.dumps(body).encode() if body is not None else None
    headers = {"Content-Type": "application/json"} if body is not None else {}
    with urllib.request.urlopen(
        urllib.request.Request(base_url + path, data=data, headers=headers), timeout=timeout
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
status, research = request(
    "/api/pipeline",
    {"backend": "catalogue", "domain": "parkinsons", "request": "LRRK2 associated with Parkinson's disease"},
    timeout=20,
)
research = json.loads(research)
assert status == 200 and research.get("citations"), research
assert research.get("outcome") in {"answer", "clarify", "review"}, research
print(f"container smoke passed at {base_url}")
PYTHON
