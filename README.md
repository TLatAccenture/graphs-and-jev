# Accelerating stack migration and modernisation with Google Cloud

### Stack migration and modernisation

[**Open the live site**](https://graphs-and-jev-956922431929.australia-southeast1.run.app/)

An Accenture thought piece on accelerating stack migration and modernisation with Google Cloud. It presents five delivery pillars: the existing stack, a governed MCP Tool Shed, System 1 and Jev, ADK with Gemini and connected context, then measured outcomes. It is not a product, a service offer or a pricing page.

## Current service and planned architecture

The live service is one Rust process on Cloud Run. It serves the static publication and same-origin API. The demonstrations use deterministic Rust rules or browser fixtures. They do not run MCP servers or call AlloyDB AI, RAG Engine, ADK, Gemini or DiffusionGemma.

The planned Google Cloud reference architecture assigns these roles:

0. **Existing Stack**: source systems remain in place, with [Cloud Storage](https://cloud.google.com/storage/docs/introduction) as the planned landing zone for raw documents and source objects.
1. **MCP Tool Shed** (connectivity): [governed MCP servers expose scoped tools and resources](https://modelcontextprotocol.io/specification/2026-07-28/server). [Compute Engine](https://cloud.google.com/compute/docs/overview) is one possible planned host.
2. **System 1 (Jev)** (reflexes): a discriminative model routes, classifies and gates bounded decisions. [AlloyDB AI can call TypeSafe's hosted Jev API](https://codelabs.developers.google.com/alloydb-ai-jev?hl=en#0); Jev is not embedded in AlloyDB.
3. **ADK with Gemini** (logic and context): [System 2 orchestration and tool use](https://docs.cloud.google.com/gemini-enterprise-agent-platform/build/adk), with [RAG Engine](https://docs.cloud.google.com/gemini-enterprise-agent-platform/build/rag-engine/layout-parser-integration) for managed retrieval and [Spanner Graph](https://cloud.google.com/spanner/docs/graph/overview) for lineage and semantic context.
4. **Modernisation Springboard** (visibility and cost): [BigQuery](https://cloud.google.com/bigquery/docs) stores analytical data, eval records and telemetry; [Looker](https://cloud.google.com/looker/docs) reports outcomes, quality and cost.

The product roles and supporting primary sources are recorded in [the implementation handover](docs/plans/2026-10-13-gcp-migration-acceleration-handover.md). The architecture is planned and is not connected to the live demonstrations.

## Pages

- [Landing page](https://graphs-and-jev-956922431929.australia-southeast1.run.app/): the migration problem, planned Google Cloud reference architecture, five pillars, a guided example on a sample estate, use cases and governance.
- [Technical guide](https://graphs-and-jev-956922431929.australia-southeast1.run.app/technical-guide/): each layer in depth, with browser labs, graph-engine grades and costs, and sources.
- [Environmental Agency catalogue discovery](https://graphs-and-jev-956922431929.australia-southeast1.run.app/environmental-agency/): discovering a fictional agency's data estate before migrating it.
- [Lineage-guarded migration](https://graphs-and-jev-956922431929.australia-southeast1.run.app/lineage-migration/): seeing the blast radius of a change before it runs.
- [Little Bunnies](https://graphs-and-jev-956922431929.australia-southeast1.run.app/little-bunnies/): why two hops of lineage are usually enough context.

The pages are static HTML, CSS and vanilla JavaScript, with their fonts served from the site, so they make no request to any other host. They are served together with the API by one Rust process on Cloud Run, from one origin. The demonstrations use a small sample graph and deterministic rules; they are illustrative, not a production migration tool.

## Run locally

In-browser fixtures only (no server code):

```bash
cd static && python3 -m http.server 8000
```

Open <http://127.0.0.1:8000/>.

Full service (pages, `/config.js` and the API from one process):

```bash
cargo run
```

Open <http://127.0.0.1:8080/>. Readiness and liveness are at `/api/ready` and `/api/health`.

## Configure

- **Static hosting:** edit `static/config.js`. It holds the brand text and API settings (`api.mode` is `"fixtures"` or `"live"`, with optional per-domain overrides).
- **Rust service:** the service answers `GET /config.js` itself, rendered from environment variables, so no files change between environments. The variables are `BRAND_ORG`, `BRAND_NAME`, `BRAND_TAGLINE`, `BRAND_KIND`, `BRAND_DISCLAIMER`, `API_MODE`, `API_BASE_URL`, `API_TIMEOUT_MS`, `API_ROUTE_HEALTH`, `API_ROUTE_CATALOGUE`, `API_ROUTE_PIPELINE`, `API_MODE_CATALOGUE`, `API_MODE_LINEAGE`, `CATALOGUE_INTERPRETER` and `GEMINI_MODEL`. Unset variables fall back to the service defaults.
- **Gemini presenter build:** with `CATALOGUE_INTERPRETER=gemini`, Gemini on Vertex AI reads typed Environmental Agency requests and maps them to catalogue terms; the rules still decide, and any Gemini failure falls back to the rules with a visible label. It is for a private presenter service only. See [docs/api-contract.md](docs/api-contract.md) and [the presenter runbook](docs/deployment/presenter-gemini.md).

## Deploy

Deployment to Cloud Run uses two guarded scripts: `deploy/deploy.sh` builds and checks a zero-traffic candidate without changing live traffic or access, and `deploy/promote.sh` moves traffic to it, keeps the service public and verifies the canonical URL, rolling back on failure. See [docs/deployment/cloud-run.md](docs/deployment/cloud-run.md) for the procedure and the current deployment record. For a live presentation, the settings in the "Event build" block of `deploy/cloud-run.env.example` run the catalogue demo in the browser and keep one instance warm; [the presentation plan](docs/plans/2026-10-13-presentation-readiness.md) has the steps.

## About

An Accenture thought piece on agentic migration and modernisation, published by TLatAccenture. It is not a product or service offer from Accenture or Google Cloud. Product names and marks belong to their owners.

## Reuse

No reuse licence is granted yet. All rights are reserved unless a licence is added later.

The fonts in `static/assets/fonts/`, Instrument Sans and JetBrains Mono, are third-party works under the SIL Open Font Licence 1.1; their licences are in the same folder.
