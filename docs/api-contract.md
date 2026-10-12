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
- The service makes no outbound network calls.

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
| `BRAND_NAME` | The MCP Tool Shed | `brand.name` | Any text |
| `BRAND_TAGLINE` | Agentic migration and modernisation | `brand.tagline` | Any text |
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

An empty variable counts as unset. Put only public settings here; the whole object is sent to every browser.

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
| `backend` | string | Optional, legacy. Defaults to `catalogue`; any other value returns 404. |

Status codes:

| Status | When |
|---|---|
| 200 | A bounded outcome (see below) |
| 404 | Unknown domain (`{"detail": "Unknown domain. Available: catalogue, lineage"}`) or unknown backend |
| 413 | Body larger than 64 KiB |
| 422 | Missing, empty or over-long `request`, over-long `preference`, or malformed JSON |
| 429 | Rate limit reached |
| 501 | `domain: "lineage"` on the Rust service: `{"detail": "Lineage domain not implemented on this service."}` |

### Common response fields

```json
{
  "outcome": "answer | clarify | reject | review | no_data",
  "message": "string or null, shown for non-answer outcomes",
  "explanation": {"by": "template", "text": "first line is the answer; later lines are reasons"},
  "stages": [{"stage": "gate", "title": "string"}],
  "backend_label": "string",
  "model": "string",
  "residency": "string",
  "totals": {"decisions": 0, "calls": 0, "wall_ms": 0}
}
```

The page labels outcomes as described in handover Section 5.3. `error` is never returned by a backend; the page uses it for failed calls.

### Domain `catalogue`

Adds `query` (the resolved pollutants, levels and filters), `solutions[]` (each with `id`, `summary`, `sources`, `profiles`, `cells` and `notes`), `subgraph` and `policy`. The Rust service sets `backend_label` to "Catalogue rules", `model` to `catalogue-rules-v1` and `residency` to `local`. `static/fixtures/catalogue.js` returns the same shape with `residency: "browser"`.

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

A lower-level route that answers typed questions about one request with the same catalogue rules. The pages do not call it.

```json
{
  "backend": "catalogue",
  "state": {"request": "annual CO2 for Australia in 2024", "preference": "optional"},
  "questions": {
    "gate": {"type": "choice", "criteria": {"answer": "...", "clarify": "...", "reject": "..."}}
  }
}
```

- `backend` is required and must be `catalogue` (404 otherwise).
- At most 100 questions; names at most 100 characters; `state.request` at most 500 characters; `state.preference` at most 300; any string inside a question at most 500; serialised `state` at most `MAX_STATE_CHARS` (default 4,000, 413 above it).
- Question `type` is `choice` (labels are the `criteria` keys), `noul` (yes or no) or `score` (labels are the indices of a `criteria` array). Supported question names: `gate`, a dimension name (`GEO`, `TIME`, `SECTOR`), `group:{id}`, `pollutant:{id}`, `relevant`, and `fit` when `state.solution` is present. Anything else returns 422.

Response:

```json
{
  "backend": "catalogue", "model": "catalogue-rules-v1", "latency_ms": 0.0, "input_tokens": null, "cost_usd": 0.0, "residency": "local",
  "detail": {"by": "catalogue rules", "calibration": "not a model probability"},
  "answers": {"gate": {"type": "choice", "probabilities": {"answer": 1.0, "clarify": 0.0, "reject": 0.0}, "top": "answer", "confidence": 1.0}}
}
```

## Response headers (Rust service)

- `X-Content-Type-Options: nosniff` and `Referrer-Policy: strict-origin-when-cross-origin` on every response.
- `Content-Security-Policy` on every response: `default-src 'self'`; scripts from `'self'` plus `'unsafe-inline'` (the pages' inline scripts and handlers) and, only on `/technical-guide/` and `/technical-guide/index.html`, `'unsafe-eval'` (the labs run edited code); styles from `'self'`, `'unsafe-inline'` and `https://fonts.googleapis.com`; fonts from `https://fonts.gstatic.com`; images from `'self'` and `data:`; `connect-src 'self'` plus the origin of `API_BASE_URL` when set; `object-src 'none'`, `base-uri 'self'`, `form-action 'self'`, `frame-ancestors 'none'`.
- `Cache-Control: no-cache` for HTML pages and `/config.js`; `public, max-age=604800` for `/assets/`.

## CORS

Same origin, as with this Rust service, needs no CORS.

If `API_BASE_URL` points at a different origin, that API must:

- return `Access-Control-Allow-Origin` set to the site's origin (not `*` if you ever add credentials; the pages send none);
- allow `GET` for health and catalogue;
- allow `POST` with `Content-Type: application/json` for pipeline, which means answering the `OPTIONS` preflight with `Access-Control-Allow-Methods: GET, POST` and `Access-Control-Allow-Headers: Content-Type`.

The site's own CSP already adds the `API_BASE_URL` origin to `connect-src`.
