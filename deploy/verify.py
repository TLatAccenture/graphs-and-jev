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
jev_verify = os.environ.get("JEV_VERIFY") == "1"
bounded_outcomes = {"answer", "clarify", "reject", "review"}
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
    # A hosted decision may differ from the rules, except small talk, which the rules always answer.
    hosted = jev_verify and text_request != "hi"
    valid_outcome = result.get("outcome") in bounded_outcomes if hosted else result.get("outcome") == outcome
    check(status == 200 and valid_outcome, f"catalogue {outcome}")
    if text_request == "hi":
        check(
            "Hello!" in result.get("message", ""),
            "catalogue hi clarify missing friendly Hello marker",
        )
        if jev_verify:
            check(result.get("backend") == "catalogue", "hi spent a hosted decision call")
        check("interpretation" not in result, "hi spent a Gemini call")
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
    policy = result.get("policy")
    check(isinstance(policy, dict), "Jev smoke policy provenance")
    for field in ("backend", "model", "residency"):
        check(policy.get(field) == result.get(field), f"Jev policy {field} disagrees with provenance")
    gate = next((stage.get("answer") for stage in result.get("stages", []) if stage.get("stage") == "gate"), None)
    check(isinstance(gate, dict), "Jev smoke gate answer")
    check(gate.get("type") == "choice", "Jev smoke typed gate")
    confidence = gate.get("confidence")
    probabilities = gate.get("probabilities")
    check(isinstance(confidence, (int, float)) and math.isfinite(confidence), "Jev smoke gate confidence")
    check(isinstance(probabilities, dict) and len(probabilities) > 1, "Jev smoke gate probabilities")
    check(all(isinstance(value, (int, float)) and math.isfinite(value) and 0 <= value <= 1 for value in probabilities.values()), "Jev smoke gate probabilities")
    check(abs(sum(probabilities.values()) - 1) <= 0.001, "Jev smoke gate probability sum")
    high = max(probabilities.values())
    expected_tops = {label for label, value in probabilities.items() if value == high}
    check(gate.get("top") in expected_tops and gate.get("choice") == gate.get("top"), "Jev smoke gate choice")
    top = gate["top"]
    uniform = 1 / len(probabilities)
    expected_confidence = (probabilities[top] - uniform) / (1 - uniform)
    check(abs(confidence - expected_confidence) <= 0.005 + math.ulp(expected_confidence), "Jev smoke gate concentration confidence")
    check(result.get("outcome") in bounded_outcomes, "Jev smoke bounded outcome")
    stages = result.get("stages")
    check(isinstance(stages, list) and stages, "Jev smoke completed stage")
    totals = result.get("totals", {})
    check(isinstance(totals.get("calls"), int) and totals["calls"] > 0, "Jev smoke completed call")
    check(totals.get("backend_errors") == 0, "Jev smoke backend error")
    check(totals.get("usage_complete") is True, "Jev smoke usage completeness")
    check(isinstance(totals.get("input_tokens"), int), "Jev smoke input token usage")
    if os.environ.get("GEMINI_VERIFY") == "1":
        check(result.get("interpretation", {}).get("by") == "gemini", "Combined smoke did not use Gemini interpretation")

print(f"verify passed at {base_url}")
