# The MCP Shed: architecture summary

A one-page summary of the architecture in [plans/2026-10-12-mcp-shed-cursor-handover.md](plans/2026-10-12-mcp-shed-cursor-handover.md) (Sections 1.2, 4 and 9). If anything here differs from the handover, the handover wins.

## Purpose

A personal research thought piece on agentic migration and modernisation, built around the MCP shed: a governed, curated set of MCP servers, with System 1 decision models, skills and hooks, a lineage graph, and measurement around it. It is not an official Accenture or Google Cloud product.

## Runtime

One Rust (Axum) process on Cloud Run serves the static pages, `/config.js` and the API from the same origin. It runs from a non-root scratch image with a static musl binary, makes no outbound calls and needs no cloud SDK. The same files also run on any static host in fixtures mode.

## Site

| Path | Purpose |
|---|---|
| `static/index.html` | Landing: problem, the shed, five pillars, guided example, use cases, governance |
| `static/technical-guide/` | Each pillar in depth, with labs, engine grades, costs and sources |
| `static/environmental-agency/` | Catalogue discovery for a fictional Environmental Agency; works with the Rust catalogue API |
| `static/lineage-migration/` | Lineage-guarded migration demo; in-browser fixtures for now |
| `static/little-bunnies/` | Small-world experiment: two hops of lineage are usually enough context |
| `static/config.js` | Brand text and API defaults for static hosting |
| `static/js/api.js` | `window.siteApi`: the only code that calls an API or fixtures |
| `static/js/estate-graph.js` | Shared sample estate graph |
| `static/fixtures/*.js` | Catalogue and lineage fixtures |

All internal links and asset paths are relative, so the site works at `/` and at any sub-path.

## Configuration

`window.SITE_CONFIG` holds `brand` (`org`, `name`, `tagline`, `kind`, `disclaimer`) and `api` (`mode`, `baseUrl`, `timeoutMs`, `routes`, per-domain `domains` overrides). On static hosting it comes from `static/config.js`. On the Rust service, `GET /config.js` renders the same object from environment variables (`BRAND_*`, `API_*`), so switching between fixtures and the live API needs no code edits. Elements marked `data-brand` take their text from `brand`.

`siteApi` exposes `catalogue()`, `pipeline({domain, request, preference})`, `health()`, `mode(domain)` and `source(domain)`. In fixtures mode it calls `window.SITE_FIXTURES[domain]`. In live mode it calls the configured routes with a timeout and no credentials. A failed live call shows "Service unavailable"; it never falls back to fixtures.

## API

| Method | Route | Purpose |
|---|---|---|
| GET | `/api/health` | Liveness |
| GET | `/api/ready` | Readiness; Cloud Run startup probe |
| GET | `/config.js` | `window.SITE_CONFIG` from environment variables |
| GET | `/api/catalogue` | Catalogue summary and sources |
| POST | `/api/pipeline` | `{domain, request, preference}`; `catalogue` (alias `air`) is answered, `lineage` returns 501 until a backend exists |

Responses carry a bounded `outcome` (`answer`, `clarify`, `reject`, `review`, `no_data`), a message, an explanation, stages, provenance fields and totals. Errors are non-2xx with `{"detail": "..."}`. The full contract is in `docs/api-contract.md`.
