# API contract

This is the request and response contract between the static pages in `static/` and a live backend. The Rust service in this repository implements it and serves the pages from the same origin. A separate backend team can implement the same routes and point the pages at it with `API_BASE_URL`.

The pages never call the API directly. `static/js/api.js` (`window.siteApi`) is the only client. It sends no credentials and no cookies, applies a timeout of `api.timeoutMs` to every call, and in live mode never falls back to in-browser fixtures: a failed call becomes "Service unavailable" on the page.

## Routes

| Method | Route | Purpose | Rate limited |
|---|---|---|---|
| GET | `/api/health` | Liveness | No |
| GET | `/api/ready` | Readiness: catalogue and static directory loaded | No |
| GET | `/config.js` | `window.SITE_CONFIG`, rendered from environment variables (Rust service only) | No |
| GET | `/api/catalogue` | The packaged catalogue the catalogue domain answers from | No |
| POST | `/api/pipeline` | Run one request through a domain and return a bounded outcome | Yes |
| POST | `/api/decide` | Answer typed questions about one request (lower-level route, not used by the pages) | Yes |

Route paths are configurable in the pages through `api.routes` (`API_ROUTE_HEALTH`, `API_ROUTE_CATALOGUE`, `API_ROUTE_PIPELINE`). `/api/ready` and `/api/decide` are not called by the pages.

The Rust service also answers the retired page paths with `308 Permanent Redirect`: `/dcceew`, `/dcceew/` and anything below it go to `/environmental-agency/`; `/parkinsons` and below go to `/lineage-migration/`.

## Common rules

- Request bodies are JSON with `Content-Type: application/json`, at most 64 KiB. Larger bodies return 413.
- Errors are non-2xx with a JSON body `{"detail": "string"}`. The page shows "Service unavailable" and the detail in its technical details.
- Every response carries `x-request-id`. Structured logs record the request ID, route, status, latency and outcome, never the request or preference text.
- `POST` routes share one process-local limit (`RATE_LIMIT_PER_MINUTE`, default 120 per instance). Over the limit they return 429 before the body is parsed. With at most two instances the effective ceiling is twice that; it is not a per-client quota.
- By default (`CATALOGUE_INTERPRETER=rules`), the service makes no outbound network calls. With the explicit `gemini` interpreter, it calls only the fixed Cloud Run metadata host for a runtime token/project ID and the location-derived Google Vertex AI endpoint.

## GET /api/health

```json
{"status": "ok", "ok": true, "version": "0.1.0", "auth_required": false, "backend": "catalogue", "sources": 8, "pollutants": 24}
```

Only `status` is part of the contract for other backends. The other fields are informational. Cloud Run's liveness probe uses this route.

## GET /api/ready

`{"ok": true}` once the catalogue has sources and pollutants. Cloud Run's startup probe uses this route.

## GET /config.js

Returns JavaScript, not JSON:

```js
window.SITE_CONFIG = {"brand":{...},"api":{...}};
```

- `Content-Type: text/javascript; charset=utf-8`, `Cache-Control: no-cache`.
- The object is serialised with `serde_json`, then every `<` is written as `\u003c` (and U+2028, U+2029 as escapes), so values containing quotes, backslashes, newlines or `</script>` stay inert.
- This route takes precedence over `static/config.js`, which is only used by plain static hosting.
- Invalid settings stop the service at start-up rather than serving a broken config.

| Variable | Default | Maps to | Valid values |
|---|---|---|---|
| `BRAND_ORG` | Accenture | `brand.org` | Any text |
| `BRAND_NAME` | Migration Acceleration with Google Cloud | `brand.name` | Any text |
| `BRAND_TAGLINE` | Stack migration and modernisation | `brand.tagline` | Any text |
| `BRAND_KIND` | Thought piece | `brand.kind` | Any text |
| `BRAND_DISCLAIMER` | The disclaimer in handover Section 4.1 | `brand.disclaimer` | Any text |
| `API_MODE` | `live` | `api.mode` | `live`, `fixtures` |
| `API_BASE_URL` | empty (same origin) | `api.baseUrl` | Empty, or an origin only: `http(s)://host[:port]`, where the host is a DNS name, an IPv4 address or a bracketed IPv6 address and the port is 0 to 65535. No path, trailing slash, query, fragment, user info or whitespace: the pages request `baseUrl + route`, so put any path prefix in the routes (for example `API_ROUTE_PIPELINE=/v1/pipeline`). The service refuses to start otherwise, and `deploy/deploy.sh` applies the same rule before any `gcloud` call. |
| `API_TIMEOUT_MS` | `15000` | `api.timeoutMs` | Positive integer |
| `API_ROUTE_HEALTH` | `/api/health` | `api.routes.health` | Absolute path starting with a single `/` |
| `API_ROUTE_CATALOGUE` | `/api/catalogue` | `api.routes.catalogue` | Absolute path starting with a single `/` |
| `API_ROUTE_PIPELINE` | `/api/pipeline` | `api.routes.pipeline` | Absolute path starting with a single `/` |
| `API_MODE_CATALOGUE` | value of `API_MODE` | `api.domains.catalogue` | `live`, `fixtures` |
| `API_MODE_LINEAGE` | `fixtures` | `api.domains.lineage` | `live`, `fixtures` |
| `CATALOGUE_INTERPRETER` | `rules` | `api.interpreter.by` | `rules`, `gemini`. With `gemini`, Gemini on Vertex AI reads typed catalogue requests first (see below). |
| `GEMINI_MODEL` | `gemini-3.8-flash` | `api.interpreter.model` (only when the interpreter is `gemini`, else `null`) | Lower-case model id |

An empty variable counts as unset. Put only public settings here; the whole object is sent to every browser.

### Gemini interpreter settings (server only)

These are read at start-up when `CATALOGUE_INTERPRETER=gemini` and never reach the browser. Invalid values stop the service. The presenter runbook is [docs/deployment/presenter-gemini.md](deployment/presenter-gemini.md).

| Variable | Default | Valid values |
|---|---|---|
| `GEMINI_LOCATION` | `global` | Vertex AI location. `global` uses `aiplatform.googleapis.com`; any other uses `{location}-aiplatform.googleapis.com`. |
| `GEMINI_TIMEOUT_MS` | `8000` | 1 to 60000. Keep it below `API_TIMEOUT_MS` so a slow reply falls back before the page gives up. |
| `GEMINI_THINKING_LEVEL` | `low` (sent as Vertex `LOW`) | `minimal`, `low`, `medium`, `high`, or `model-default` to send no thinking setting. |
| `GOOGLE_CLOUD_PROJECT` | from the metadata server | Project id that owns the Vertex AI call. |
| `GEMINI_ACCESS_TOKEN` | none | Local runs only. `deploy/deploy.sh` refuses it; Cloud Run uses the runtime service account through the metadata server. |
Production endpoint and metadata hosts are not configurable: `global` maps exactly to `https://aiplatform.googleapis.com`, a validated region maps to `https://{region}-aiplatform.googleapis.com`, and metadata uses `metadata.google.internal`. Redirects are refused. Loopback fake origins exist only through the Rust integration-test constructor, not environment variables.

## GET /api/catalogue

```json
{
  "summary": "string",
  "pollutants": [{"id": "CO2", "label": "CO2", "name": "carbon dioxide", "unit": "Mt", "groups": ["ghg"]}],
  "groups": {"ghg": "Greenhouse gases"},
  "dimensions": {"TIME": ["month", "year"], "SECTOR": ["subsector", "macrosector"], "GEO": ["region", "country", "continent"]},
  "defaults": {"SECTOR": "macrosector", "GEO": "country", "TIME": "year"},
  "sources": [{"id": "S1", "name": "string", "description": "string", "publisher": "official | research", "updated": 2025, "levels": {"GEO": "country"}, "pollutants": ["CO2"], "coverage": {"GEO": ["first", "last", 12]}}]
}
```

Source `id` values are strings such as `"S1"`. Each `coverage` entry is `[first member, last member, member count]`, or `[]` when empty.

## POST /api/pipeline

Request, exactly as `siteApi.pipeline` sends it:

```json
{"domain": "catalogue", "request": "Annual CO2 for Australia in 2024", "preference": null}
```

| Field | Type | Rules |
|---|---|---|
| `domain` | string | `catalogue` (alias `air` for older clients) or `lineage`. Defaults to `catalogue` when omitted or empty. |
| `request` | string | Required. 1 to 500 characters after trimming. |
| `preference` | string or null | Optional. At most 300 characters. |
| `backend` | string | Optional, legacy. When present it must exactly match the configured `DECISION_BACKEND` (`catalogue` or `jev`); a mismatch returns 404. It does not select or override the backend. |

Status codes:

| Status | When |
|---|---|
| 200 | A bounded outcome (see below) |
| 404 | Unknown domain (`{"detail": "Unknown domain. Available: catalogue, lineage"}`) or unknown backend |
| 413 | Body larger than 64 KiB |
| 422 | Missing, empty or over-long input, malformed JSON, or a request the selected backend rejects as invalid |
| 429 | Service rate limit reached |
| 502 | `/api/decide`: the selected backend returned a malformed or otherwise invalid upstream response |
| 503 | `/api/decide`: the selected backend was unavailable or timed out; `/api/pipeline`: only unexpected outer service/backend unavailability before a normal review response can be formed |
| 501 | `domain: "lineage"` on the Rust service: `{"detail": "Lineage domain not implemented on this service."}` |

### Common response fields

```json
{
  "outcome": "answer | clarify | reject | review | no_data",
  "message": "string or null, shown for non-answer outcomes",
  "explanation": {"by": "template", "text": "first line is the answer; later lines are reasons"},
  "stages": [{"stage": "gate", "title": "string"}],
  "backend": "catalogue | jev",
  "backend_label": "string",
  "model": "string",
  "residency": "local | hosted",
  "policy": {"backend": "catalogue | jev", "model": "string", "residency": "local | hosted"},
  "totals": {"decisions": 0, "calls": 0, "backend_errors": 0, "wall_ms": 0, "input_tokens": null, "cost_usd": null, "usage_complete": false}
}
```

`DECISION_BACKEND` selects `catalogue` (default deterministic rules) or `jev` (TypeSafe hosted Jev) for both `/api/decide` and every decision stage in `/api/pipeline`; clients do not select it per request. Small talk sent to `/api/pipeline` is the exception (`/api/decide` always uses the configured backend): a request that is only a greeting, thanks, a goodbye, an acknowledgement such as "ok", or a question about what the service can do gets a `clarify` reply from the local catalogue rules whatever `DECISION_BACKEND` and `CATALOGUE_INTERPRETER` are. It makes no Gemini or hosted call, carries no `interpretation`, and reports `backend: "catalogue"`, `model: "catalogue-rules-v1"` and `residency: "local"`. Pipeline and policy provenance report the selected backend, model and residency. Gemini interpretation, when enabled, runs first on every other request; the configured decision backend then decides on Gemini's catalogue-form request or on the reader's words after an interpretation fallback. Aggregate `input_tokens` and `cost_usd` are nullable; `usage_complete` is false whenever any backend call omits usage and true only when every decision call reported usage. The page labels outcomes as described in handover Section 5.3. `error` is never returned by a backend; the page uses it for failed calls.

### Domain `catalogue`

Adds `query` (the resolved pollutants, levels and filters), `solutions[]` (each with `id`, `summary`, `sources`, `profiles`, `cells` and `notes`), `subgraph` and `policy`. With the default `DECISION_BACKEND=catalogue`, the Rust service reports `backend: "catalogue"`, `backend_label: "Catalogue rules"`, `model: "catalogue-rules-v1"` and `residency: "local"`. With `DECISION_BACKEND=jev`, it reports `backend: "jev"`, the pinned Jev model and `residency: "hosted"`, except for small talk (see above). The top-level and `policy` backend/model/residency fields agree. `static/fixtures/catalogue.js` uses deterministic browser rules and `residency: "browser"`.

When the service runs with `CATALOGUE_INTERPRETER=gemini`, the response also carries `interpretation` (except for small talk), and `request` stays the reader's own text:

```json
{"interpretation": {"by": "gemini | rules", "model": "gemini-3.8-flash", "request": "PM2.5 by country and year from 2019 to 2024", "note": "string or null"}}
```

- Gemini only maps the reader's words to catalogue terms, through a response schema whose values are the catalogue's own pollutants, groups, levels, places and sectors. The service writes those terms back as `interpretation.request`, and the configured `catalogue` or `jev` decision backend decides on that text. Gemini never chooses the outcome, a dataset or a number.
- `by: "gemini"` with `note: null`: the configured decision backend decided on Gemini's reading.
- `by: "gemini"` with an outside-catalogue note: Gemini found something the catalogue does not hold, so the configured backend decided on the reader's text.
- `by: "rules"`: Gemini failed (no token, HTTP error, timeout, unreadable reply or a term outside the catalogue), so deterministic interpretation passed the reader's words to the configured decision backend. The outcome is never an error merely because Gemini failed.
- The request log gains an `interpreter` field (`gemini`, `gemini_outside_catalogue`, `gemini_no_terms` or `rules_fallback: <reason>`). It never holds request text. Small-talk log lines have no `interpreter` field.
- Without the setting there is no `interpretation` field and no outbound call.

### Domain `lineage`

The Rust service returns 501 and the page runs this domain from `static/fixtures/lineage.js`. A backend that implements it adds:

```json
{
  "graph": {"edges": [{"from": "id", "from_label": "string", "ty": "MAPS_TO", "to": "id", "to_label": "string"}]},
  "playbook": {"skill": "string", "steps": [{"name": "string", "status": "done | skipped | blocked"}]},
  "eval": {"run_id": "string", "decision": "string", "confidence": 0.0, "tools_called": ["string"], "tokens_in": 0, "tokens_out": 0, "latency_ms": 0, "cost_usd": 0.0}
}
```

Stage ids are `gate`, `lineage`, `playbook`, `hook` and `eval`. To switch the page to it, set `API_MODE_LINEAGE=live`.

## POST /api/decide

A lower-level route that sends typed questions about one request directly to the configured `catalogue` or `jev` decision backend. It does not run Gemini interpretation. The pages do not call it.

```json
{
  "backend": "catalogue",
  "state": {"request": "annual CO2 for Australia in 2024", "preference": "optional"},
  "questions": {
    "gate": {"type": "choice", "criteria": {"answer": "...", "clarify": "...", "reject": "..."}}
  }
}
```

- The request `backend` is optional and legacy. When present, it must exactly equal the configured backend name (`catalogue` or `jev`); a mismatch returns 404. It never overrides `DECISION_BACKEND`.
- Responses report the configured `backend`, `model` and `residency`. Validation failures return 422, malformed upstream responses return 502, and unavailable or timed-out backends return 503.
- Hosted Jev confidence is a concentration statistic, not the largest probability. For `choice`, confidence is `(p_max - 1/n) / (1 - 1/n)`; probabilities `0.32 / 0.64 / 0.04` have confidence `0.46`, and a tied provider choice may be any argmax. For ordered `score` probabilities, `m` is the first maximum index, `spread = sum(p_i * abs(i-m))`, `MAD_uniform = sum(abs(i-(n-1)/2))/n`, and confidence is `max(0, 1-spread/MAD_uniform)`; for example `0 / 0.57 / 0.43` gives approximately `0.355`. Choice and score require at least two labels. Hosted `noul` supplies only its yes probability, so the service derives yes/no probabilities and confidence `abs(2p - 1)` (`0.5` is `0`, `0.9` is `0.8`). Provider-supplied `choice`/`score` confidence, score expected value, and score legend must match these semantics or the response is rejected. The local catalogue backend retains its deterministic confidence values.
- At most 100 questions; names at most 100 characters; `state.request` at most 500 characters; `state.preference` at most 300; any string inside a question at most 500; serialised `state` at most `MAX_STATE_CHARS` (default 4,000, 413 above it).
- Question `type` is `choice` (labels are the `criteria` keys), `noul` (yes or no) or `score` (labels are the indices of a `criteria` array). Supported question names: `gate`, a dimension name (`GEO`, `TIME`, `SECTOR`), `group:{id}`, `pollutant:{id}`, `relevant`, and `fit` when `state.solution` is present. Anything else returns 422.

Response (default catalogue backend shown; hosted Jev reports `backend: "jev"`, its pinned model and `residency: "hosted"`):

```json
{
  "backend": "catalogue", "model": "catalogue-rules-v1", "latency_ms": 0.0, "input_tokens": null, "cost_usd": 0.0, "residency": "local",
  "detail": {"by": "catalogue rules", "calibration": "not a model probability"},
  "answers": {"gate": {"type": "choice", "probabilities": {"answer": 1.0, "clarify": 0.0, "reject": 0.0}, "top": "answer", "confidence": 1.0}}
}
```

## Response headers (Rust service)

- `X-Content-Type-Options: nosniff` and `Referrer-Policy: strict-origin-when-cross-origin` on every response.
- `Content-Security-Policy` on every response: `default-src 'self'`; scripts from `'self'` plus `'unsafe-inline'` (the pages' inline scripts and handlers) and, only on `/technical-guide/` and `/technical-guide/index.html`, `'unsafe-eval'` (the labs run edited code); styles from `'self'` and `'unsafe-inline'`; fonts from `'self'` only (the site serves its own fonts); images from `'self'` and `data:`; `connect-src 'self'` plus the origin of `API_BASE_URL` when set; `object-src 'none'`, `base-uri 'self'`, `form-action 'self'`, `frame-ancestors 'none'`.
- `Cache-Control: no-cache` for HTML pages and `/config.js`; `public, max-age=604800` for `/assets/`.

## CORS

Same origin, as with this Rust service, needs no CORS.

If `API_BASE_URL` points at a different origin, that API must:

- return `Access-Control-Allow-Origin` set to the site's origin (not `*` if you ever add credentials; the pages send none);
- allow `GET` for health and catalogue;
- allow `POST` with `Content-Type: application/json` for pipeline, which means answering the `OPTIONS` preflight with `Access-Control-Allow-Methods: GET, POST` and `Access-Control-Allow-Headers: Content-Type`.

The site's own CSP already adds the `API_BASE_URL` origin to `connect-src`.


## Private hosted Jev presenter

The private Archegon presenter sets `DECISION_BACKEND=jev`, `SERVICE_AUTH_MODE=cloud-run-iam`, and an exact `JEV_MODEL`. It returns `backend: "jev"`, the pinned model, `residency: "hosted"`, matching policy provenance, an actual typed gate with finite concentration confidence/probabilities, at least one completed stage and backend call, zero backend errors, and complete aggregate usage. Private release smoke accepts any HTTP 200 bounded outcome (`answer`, `clarify`, `reject`, or `review`) for its other catalogue probes because a real low-confidence model response may correctly review, but `hi` must return `clarify` with the `Hello!` reply, `backend: "catalogue"` and no `interpretation`, because small talk never reaches a hosted backend; it still rejects empty-stage backend-error reviews and catalogue fallback. Combined Gemini plus Jev smoke additionally requires `interpretation.by: "gemini"`. Public/default smoke continues to require the exact deterministic catalogue outcomes. `TYPESAFE_API_KEY` is runtime-only and must be supplied through a Cloud Run Secret Manager reference; it is never part of `/config.js`, deployment app environment, logs, or verification output.

On `/api/decide`, selected-backend request validation failures return 422, malformed upstream responses return 502, and unavailable or timed-out calls return 503. In `/api/pipeline`, an ordinary decision-stage backend failure or timeout returns HTTP 200 with `outcome: "review"`, a screened message, completed stages and preserved usage totals. A pipeline 503 is reserved for unexpected outer service/backend unavailability before that normal review response can be formed. Service-level rate limiting returns 429. Gemini is separate: it may interpret typed catalogue text before the selected decision backend runs. Enabling Gemini does not select Jev, and selecting Jev does not enable Gemini. `/config.js` exposes only safe `decisionBackend` provenance (`by`, `label`, `model`, `residency`), never secret names, versions, keys, or endpoints.
