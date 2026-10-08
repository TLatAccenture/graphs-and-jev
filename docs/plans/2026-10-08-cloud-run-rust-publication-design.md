# Graphs + Jev Cloud Run Publication Design

## Decision

Publish Graphs + Jev as one public Cloud Run service in GCP project `ninth-airship-386815`, region `australia-southeast1`. One Rust process serves the field guide and its API on the same origin. The GitHub repository remains public source; GitHub Pages remains available only until the Cloud Run deployment passes its gates, then is disabled.

The service name and Artifact Registry repository are both `graphs-and-jev`. The deployment account is `anthony.lui@archegon.com`. The Gemini API project `gen-lang-client-0365409577` remains separate because this service does not depend on its API key or quota boundary.

## Architecture

The standalone repository becomes the complete application:

- one Rust crate with stage modules for auth/rate control, catalogue, graph, request parsing, bounded decisions, pipeline orchestration, and Parkinson’s research retrieval;
- the field-guide HTML, CSS, JavaScript, and images under `static/`;
- `catalogue.json` as immutable packaged demonstration data;
- a multi-stage Dockerfile producing one non-root Debian image with no cloud SDK;
- Cloud Run listening on the injected `PORT`, serving frontend and API from the same origin.

Public routes are `GET /`, the three use-case pages and assets, `GET /api/health`, `GET /api/ready`, `GET /api/catalogue`, `POST /api/decide`, and `POST /api/pipeline`. DCCEEW uses the real deterministic Rust catalogue pipeline. Parkinson’s uses the real safety gate and live Wikipedia/Europe PMC retrieval. The browser fixtures are removed after the live backend is verified.

## Public-demo boundary

The Cloud Run service allows unauthenticated invocation. No API token is embedded in JavaScript. Safety is provided by bounded behavior rather than a browser secret:

- strict JSON body and field-length limits;
- bounded outbound connection and response timeouts;
- clinical requests rejected before retrieval;
- rate limiting before expensive routes;
- small Cloud Run concurrency and maximum-instance limits;
- no raw request or preference text in logs;
- structured request ID, route, status, latency, outcome, and upstream-failure logs;
- no cookies, user accounts, persistent application state, or hosted model calls.

The first deployment may use process-local rate limits because this is a small public demonstration. This is a deliberate ceiling: limits are per instance, so a multi-instance production service must move them to a shared edge or durable store. Cloud Run infrastructure limits cap the remaining blast radius.

## Reliability and failure behavior

`/api/health` proves the process is alive. `/api/ready` proves the packaged catalogue and static directory loaded. Cloud Run startup and liveness probes use these routes. Important work does not run in process-local background jobs.

Catalogue decisions remain available without external services. Parkinson’s retrieval returns an explicit unavailable response when public sources time out or fail; it never fabricates evidence. The frontend explains the failure and retains source-free safety refusals. Static files use cache headers appropriate to immutable assets while HTML remains revalidatable.

## Delivery

Enable only the required services in `ninth-airship-386815`: Cloud Run, Artifact Registry, Cloud Build, and the IAM/service-control dependencies they require. Create a Docker Artifact Registry repository in `australia-southeast1`. Build an image tagged with the Git commit SHA, deploy it, record the resolved digest, and use a dedicated runtime service account with no broad project role.

The initial Cloud Run settings are conservative: public ingress, minimum instances 0, maximum instances 2, concurrency 20, one CPU, 512 MiB memory, and a request timeout sized for bounded live research retrieval. Adjust only from observed metrics.

## Verification and cutover

Before deployment, run Rust unit/integration tests, static frontend contract tests, a release build, a local container smoke test, and an image vulnerability scan. After deployment, verify health/readiness, all four pages, static assets, DCCEEW answer/clarify/reject paths, Parkinson’s answer and pre-retrieval clinical refusal, outbound timeout behavior, request limits, no raw prompt logging, and desktop/mobile rendering.

Only after those checks pass:

1. update the repository homepage to the Cloud Run URL;
2. update README links;
3. disable GitHub Pages and remove its workflow;
4. verify the old Pages URL no longer appears as the primary publication URL.

Rollback is a Cloud Run traffic change to the previous healthy revision. The GitHub Pages site is not disabled until a rollback-capable Cloud Run revision exists.

## Non-goals

No Gemini integration, custom domain, user authentication, persistent database, compliance claims, production clinical support, multi-region service, global distributed rate limiter, or automated benchmark execution is part of this deployment.
