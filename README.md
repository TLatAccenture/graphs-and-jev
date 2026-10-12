# The MCP Tool Shed

### Agentic migration and modernisation

[**Open the live site**](https://graphs-and-jev-956922431929.australia-southeast1.run.app/)

An Accenture thought piece on agentic migration and modernisation. It shows how we think about giving migration agents the right tools for the job: a governed, curated set of MCP servers (the MCP Tool Shed), with fast decision models, playbooks, a lineage graph and measurement around it. It is not a product, a service offer or a pricing page.

## The architecture

0. **Existing Stack**: AWS and Azure storage, SAP, Salesforce, Databricks, ServiceNow, Atlassian and Microsoft 365 files, left where they are.
1. **MCP Tool Shed** (connectivity): standard, governed MCP servers replace brittle one-off scripts.
2. **System 1 (Jev)** (reflexes): a fast decision model routes, classifies and gates every tool call.
3. **Gemini Enterprise Agent Platform** (logic and context): agents follow skills and hooks, with Spanner Graph as the context fabric for lineage and meaning.
4. **Modernisation Springboard** (visibility and cost): BigQuery as the destination lakehouse and the store for telemetry, audit logs and eval records; Looker reports the result and the cost.

## Pages

- [Landing page](https://graphs-and-jev-956922431929.australia-southeast1.run.app/): the problem, the MCP Tool Shed architecture, a guided example on a sample estate, use cases and governance.
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
- **Rust service:** the service answers `GET /config.js` itself, rendered from environment variables, so no files change between environments. The variables are `BRAND_ORG`, `BRAND_NAME`, `BRAND_TAGLINE`, `BRAND_KIND`, `BRAND_DISCLAIMER`, `API_MODE`, `API_BASE_URL`, `API_TIMEOUT_MS`, `API_ROUTE_HEALTH`, `API_ROUTE_CATALOGUE`, `API_ROUTE_PIPELINE`, `API_MODE_CATALOGUE` and `API_MODE_LINEAGE`. Unset variables fall back to the service defaults.

## Deploy

Deployment to Cloud Run uses two guarded scripts: `deploy/deploy.sh` builds and checks a zero-traffic candidate without changing live traffic or access, and `deploy/promote.sh` moves traffic to it, keeps the service public and verifies the canonical URL, rolling back on failure. See [docs/deployment/cloud-run.md](docs/deployment/cloud-run.md) for the procedure and the current deployment record. For a live presentation, the settings in the "Event build" block of `deploy/cloud-run.env.example` run the catalogue demo in the browser and keep one instance warm; [the presentation plan](docs/plans/2026-10-13-presentation-readiness.md) has the steps.

## About

An Accenture thought piece on agentic migration and modernisation, published by TLatAccenture. It is not a product or service offer from Accenture or Google Cloud. Product names and marks belong to their owners.

## Reuse

No reuse licence is granted yet. All rights are reserved unless a licence is added later.

The fonts in `static/assets/fonts/`, Instrument Sans and JetBrains Mono, are third-party works under the SIL Open Font Licence 1.1; their licences are in the same folder.
