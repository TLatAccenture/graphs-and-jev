#!/usr/bin/env python3
"""Release checks against a deployed URL: python3 deploy/verify.py BASE_URL.

Set VERIFY_BEARER to send an identity token (candidate tag URLs on a private service).
The token is read from the environment so it never appears in argv or output.
Exits non-zero with a one-line reason on the first failed check.
"""

import json
import os
import sys
import time
import urllib.error
import urllib.request

base_url = sys.argv[1].rstrip("/")
bearer = os.environ.get("VERIFY_BEARER", "")


def request(path, body=None, timeout=15):
    headers = {"Content-Type": "application/json"} if body is not None else {}
    if bearer:
        headers["Authorization"] = f"Bearer {bearer}"
    data = json.dumps(body).encode() if body is not None else None
    req = urllib.request.Request(base_url + path, data=data, headers=headers)
    try:
        with urllib.request.urlopen(req, timeout=timeout) as response:
            return response.status, response.read(), response.geturl()
    except urllib.error.HTTPError as error:
        return error.code, error.read(), error.geturl()


def check(condition, message):
    if not condition:
        raise SystemExit(f"verify failed at {base_url}: {message}")


for attempt in range(12):
    try:
        status, body, _ = request("/api/health")
        if status == 200 and json.loads(body).get("ok") is True:
            break
    except (OSError, ValueError):
        pass
    time.sleep(5)
else:
    check(False, "/api/health did not return ok")

status, body, _ = request("/api/ready")
check(status == 200 and json.loads(body).get("ok") is True, "/api/ready")
status, body, _ = request("/api/catalogue")
check(status == 200 and json.loads(body), "/api/catalogue")
for path in (
    "/",
    "/technical-guide/",
    "/environmental-agency/",
    "/lineage-migration/",
    "/little-bunnies/",
):
    status, body, _ = request(path)
    check(
        status == 200 and b'data-brand="name"' in body, f"{path} did not serve the page"
    )
for old, new in (
    ("/dcceew/", "/environmental-agency/"),
    ("/parkinsons/", "/lineage-migration/"),
):
    status, _, final = request(old)
    check(status == 200 and final.endswith(new), f"{old} did not redirect to {new}")
status, body, _ = request("/config.js")
text = body.decode()
prefix, suffix = "window.SITE_CONFIG = ", ";\n"
check(
    status == 200 and text.startswith(prefix) and text.endswith(suffix),
    "/config.js shape",
)
json.loads(text[len(prefix) : -len(suffix)])
for text_request, outcome in (
    ("annual CO2 for Australia in 2024", "answer"),
    ("emissions", "clarify"),
    ("write me a poem about the sea", "reject"),
):
    status, body, _ = request(
        "/api/pipeline",
        {"domain": "catalogue", "request": text_request, "preference": None},
    )
    check(
        status == 200 and json.loads(body).get("outcome") == outcome,
        f"catalogue {outcome}",
    )
status, _, _ = request(
    "/api/pipeline",
    {
        "domain": "lineage",
        "request": "Drop table KNA1 after cutover",
        "preference": None,
    },
)
check(status == 501, "lineage should return 501")
print(f"verify passed at {base_url}")
