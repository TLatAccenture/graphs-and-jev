#!/usr/bin/env python3
"""Release checks against a deployed URL: python3 deploy/verify.py BASE_URL.

Set VERIFY_BEARER to send an identity token (candidate tag URLs on a private service).
The token is read from the environment so it never appears in argv or output.
Exits non-zero with a one-line reason on the first failed check.
"""

import json
import math
import os
import sys
import time
import urllib.error
import urllib.request
from urllib.parse import urljoin, urlsplit

base_url = sys.argv[1].rstrip("/")
bearer = os.environ.get("VERIFY_BEARER", "")
attempts = int(os.environ.get("VERIFY_ATTEMPTS", "12"))


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


opener = urllib.request.build_opener(NoRedirect)


def request(path, body=None, timeout=15):
    headers = {"Content-Type": "application/json"} if body is not None else {}
    if bearer:
        headers["Authorization"] = f"Bearer {bearer}"
    data = json.dumps(body).encode() if body is not None else None
    req = urllib.request.Request(base_url + path, data=data, headers=headers)
    try:
        with opener.open(req, timeout=timeout) as response:
            return response.status, response.read(), response.geturl()
    except urllib.error.HTTPError as error:
        location = error.headers.get("Location", "")
        if 300 <= error.code < 400:
            return error.code, error.read(), location
        return error.code, error.read(), error.geturl()


def check(condition, message):
    if not condition:
        raise SystemExit(f"verify failed at {base_url}: {message}")


for attempt in range(attempts):
    try:
        status, body, final = request("/api/health")
        if status == 200 and json.loads(body).get("ok") is True:
            break
    except (OSError, ValueError):
        pass
    if attempt + 1 < attempts:
        time.sleep(5)
else:
    detail = f"/api/health returned HTTP {status}"
    if 300 <= status < 400:
        detail += f" redirect to {final or 'unknown location'} (redirects disabled)"
    check(False, detail)

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
base = urlsplit(base_url)
for old, expected_path in (
    ("/dcceew/", "/environmental-agency/"),
    ("/parkinsons/", "/lineage-migration/"),
):
    status, _, location = request(old)
    target = urlsplit(urljoin(base_url + "/", location))
    safe = (
        target.scheme == base.scheme
        and target.hostname == base.hostname
        and target.port == base.port
        and target.path == expected_path
        and not target.query
        and not target.fragment
        and target.username is None
        and target.password is None
    )
    check(
        status == 308 and bool(location) and safe,
        f"{old} expected HTTP 308 to same-origin {expected_path}; got HTTP {status} Location {location or '<missing>'}",
    )
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
    ("hi", "clarify"),
    ("write me a poem about the sea", "reject"),
):
    status, body, _ = request(
        "/api/pipeline",
        {"domain": "catalogue", "request": text_request, "preference": None},
    )
    result = json.loads(body) if status == 200 else {}
    check(status == 200 and result.get("outcome") == outcome, f"catalogue {outcome}")
    if text_request == "hi":
        check(
            "Hello!" in result.get("message", ""),
            "catalogue hi clarify missing friendly Hello marker",
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
if os.environ.get("GEMINI_VERIFY") == "1" or os.environ.get("JEV_VERIFY") == "1":
    status, body, _ = request(
        "/api/pipeline",
        {"domain": "catalogue", "request": "could I see fine particles for each nation, yearly, 2019 through 2024?", "preference": None},
    )
    result = json.loads(body) if status == 200 else {}
if os.environ.get("GEMINI_VERIFY") == "1":
    expected_model = os.environ.get("VERIFY_EXPECTED_MODEL", "")
    interpretation = result.get("interpretation", {})
    check(interpretation.get("by") == "gemini", "Gemini smoke did not use Gemini")
    check(interpretation.get("model") == expected_model, "Gemini smoke model")
    check(interpretation.get("request") == "PM2.5 by country and year from 2019 to 2024", "Gemini smoke canonical request")
if os.environ.get("JEV_VERIFY") == "1":
    expected_jev_model = os.environ.get("JEV_VERIFY_EXPECTED_MODEL", "")
    check(result.get("backend") == "jev", "Jev smoke did not use hosted Jev")
    check(result.get("model") == expected_jev_model, "Jev smoke model")
    check(result.get("residency") == "hosted", "Jev smoke residency")
    policy = result.get("policy", {})
    for field in ("backend", "model", "residency"):
        if field in policy:
            check(policy[field] == result.get(field), f"Jev policy {field} disagrees with provenance")
    gate = next((stage.get("answer") for stage in result.get("stages", []) if stage.get("stage") == "gate"), None)
    check(isinstance(gate, dict), "Jev smoke gate answer")
    confidence = gate.get("confidence")
    probabilities = gate.get("probabilities")
    check(isinstance(confidence, (int, float)) and math.isfinite(confidence), "Jev smoke gate confidence")
    check(isinstance(probabilities, dict) and probabilities, "Jev smoke gate probabilities")
    check(all(isinstance(value, (int, float)) and math.isfinite(value) for value in probabilities.values()), "Jev smoke gate probabilities")
    check("usage_complete" in result.get("totals", {}), "Jev smoke usage completeness")

print(f"verify passed at {base_url}")
