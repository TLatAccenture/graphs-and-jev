# Migration Acceleration with Google Cloud: architecture summary

A one-page summary of the current runtime and the planned architecture in [plans/2026-10-13-gcp-migration-acceleration-handover.md](plans/2026-10-13-gcp-migration-acceleration-handover.md), with the landing story from [plans/2026-10-16-gemini-orchestrated-migration-story-plan.md](plans/2026-10-16-gemini-orchestrated-migration-story-plan.md). For the publication name, landing story, Google Cloud product roles and icons, that handover wins. The earlier [MCP shed handover](plans/2026-10-12-mcp-shed-cursor-handover.md) still governs the runtime, API and demonstration contracts.

## Purpose

An Accenture thought piece on accelerating stack migration and modernisation with Google Cloud. It keeps the MCP Tool Shed as the governed connectivity pillar, with System 1 decision models, ADK with Gemini, retrieval, lineage and measurement around it. It is not an official Accenture or Google Cloud product.

## Runtime

One Rust (Axum) process on Cloud Run serves the static pages, `/config.js` and the API from the same origin. It runs from a non-root scratch image with a static musl binary, makes no outbound calls and needs no cloud SDK. The same files also run on any static host in fixtures mode.

The live demonstrations use deterministic Rust rules or browser fixtures. They do not run MCP servers and do not call AlloyDB AI, Google Cloud RAG Engine, ADK, Gemini or DiffusionGemma.

## Planned Google Cloud reference architecture

This reference architecture is planned and not live in the demo. A Gemini Migration Agent, built with ADK, orchestrates each workload through [Google's four migration phases](https://docs.cloud.google.com/architecture/migration-to-gcp-getting-started): Assess, Plan, Deploy and Optimise. Jev gates bounded decisions in every phase and a person approves the plan before Deploy. The agent coordinates and explains; it does not move bytes, count rows or bypass approvals. The product roles are:

1. The source estate is what is being migrated, and it is not on Google Cloud: PySpark and Spark jobs, Hadoop and Hive, SAP, Databricks, Salesforce, AWS S3, Azure Blob and Microsoft 365 files. It keeps running while work moves across.
2. [Governed MCP servers expose tools and resources](https://modelcontextprotocol.io/specification/2026-07-28/server) over the source estate and may run on [Compute Engine](https://cloud.google.com/compute/docs/overview). Raw documents and source objects land in [Cloud Storage](https://cloud.google.com/storage/docs/introduction). [Google Cloud RAG Engine](https://docs.cloud.google.com/gemini-enterprise-agent-platform/build/rag-engine/layout-parser-integration) provides managed document parsing and retrieval where suitable.
3. AlloyDB AI holds operational PostgreSQL state and can call TypeSafe's hosted Jev API through AI functions and Model Endpoint Management, as shown in the [AlloyDB AI and Jev codelab](https://codelabs.developers.google.com/alloydb-ai-jev?hl=en#0). Jev is not embedded in AlloyDB. [DiffusionGemma may run behind the same discriminative-model API on Compute Engine or Cloud Run](https://codelabs.developers.google.com/codelabs/discriminative-models#0).
4. [Agent Development Kit (ADK) with Gemini](https://docs.cloud.google.com/gemini-enterprise-agent-platform/build/adk) provides planned System 2 reasoning, agent orchestration and tool use.
5. [Spanner Graph](https://cloud.google.com/spanner/docs/graph/overview) holds lineage and semantic context, [BigQuery](https://cloud.google.com/bigquery/docs) holds analytical data, eval records and telemetry, and [Looker](https://cloud.google.com/looker/docs) reports outcomes, quality and cost.

Deterministic code and SQL parse exact formats, count rows and validate schemas. Model layers interpret intent, route work and gate bounded decisions.

## Site

| Path | Purpose |
|---|---|
| `static/index.html` | Landing: Gemini-orchestrated hero with live/reference status, problem, four-phase control plane and representative migration lanes, five capabilities, guided example with orchestrator view, use cases, four-phase start, governance |
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
