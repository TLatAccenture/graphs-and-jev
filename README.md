# The MCP Shed

### Agentic migration and modernisation

[**Open the live site**](https://graphs-and-jev-956922431929.australia-southeast1.run.app/)

A personal research thought piece on agentic migration and modernisation. It shows how we think about giving migration agents the right tools for the job: a governed, curated set of MCP servers (the MCP shed), with fast decision models, playbooks, a lineage graph and measurement around it. It is not a product, a service offer or a pricing page.

## The five pillars

1. **Connectivity**: standard, governed MCP servers replace brittle one-off scripts.
2. **Relational context**: a graph holds lineage and the enterprise ontology.
3. **Logic**: skills and hooks turn migration playbooks into enforced steps.
4. **Reflexes**: System 1 models route, classify and gate in milliseconds, not seconds.
5. **Visibility and cost**: evals show what agents did; BigQuery and Looker show users the result and the cost.

## Pages

- [Landing page](https://graphs-and-jev-956922431929.australia-southeast1.run.app/): the problem, the MCP shed, the five pillars, a guided example on a sample estate, use cases and governance.
- [Technical guide](https://graphs-and-jev-956922431929.australia-southeast1.run.app/technical-guide/): each pillar in depth, with browser labs, graph-engine grades and costs, and sources.
- [Environmental Agency catalogue discovery](https://graphs-and-jev-956922431929.australia-southeast1.run.app/environmental-agency/): discovering a fictional agency's data estate before migrating it.
- [Lineage-guarded migration](https://graphs-and-jev-956922431929.australia-southeast1.run.app/lineage-migration/): seeing the blast radius of a change before it runs.
- [Little Bunnies](https://graphs-and-jev-956922431929.australia-southeast1.run.app/little-bunnies/): why two hops of lineage are usually enough context.

The pages are static HTML, CSS and vanilla JavaScript. They are served together with the API by one Rust process on Cloud Run, from one origin. The demonstrations use a small sample graph and deterministic rules; they are illustrative, not a production migration tool.

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

Deployment to Cloud Run uses the guarded release script `deploy/deploy.sh` (private candidate, verify, promote). See [docs/deployment/cloud-run.md](docs/deployment/cloud-run.md) for the procedure and the current deployment record.

## About

A personal research thought piece on agentic migration and modernisation by TLatAccenture. It is not an official Accenture or Google Cloud product, a service offer or a statement of capability. Product names and marks belong to their owners.

## Reuse

No reuse licence is granted yet. All rights are reserved unless a licence is added later.
