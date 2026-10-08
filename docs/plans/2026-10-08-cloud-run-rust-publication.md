# Graphs + Jev Cloud Run Publication Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Publish the complete Graphs + Jev Rust application as one public Cloud Run service in `ninth-airship-386815` while preserving the current Pages site until live verification succeeds.

**Architecture:** The repository will contain one Rust crate and its static field guide. A non-root multi-stage container serves HTML and same-origin API routes; Cloud Build writes a commit-tagged image to Artifact Registry and Cloud Run deploys the resolved digest.

**Tech Stack:** Rust 2021, Axum, Tokio, reqwest/rustls, tower-http, Docker, Artifact Registry, Cloud Build, Cloud Run, vanilla HTML/CSS/JavaScript.

---

### Task 1: Import the Rust server without changing public behavior

**Files:**
- Create: `Cargo.toml`
- Create: `Cargo.lock`
- Create: `src/auth.rs`
- Create: `src/catalogue.rs`
- Create: `src/decide.rs`
- Create: `src/graph.rs`
- Create: `src/main.rs`
- Create: `src/pipeline.rs`
- Create: `src/request.rs`
- Create: `src/research.rs`
- Create: `catalogue.json`
- Move: `index.html`, `guide.css`, `dcceew/`, `parkinsons/`, `little-bunnies/`, `assets/` to `static/`
- Test: `tests/static_frontend.rs`

**Steps:**
1. Copy the Rust crate and catalogue from the canonical local source at `/Users/anthonylui/GraphJev/graphragapp/serve/`; do not import Python lab code or plans.
2. Write/port the static route tests first, including HTTP 200 for all four pages/assets and HTTP 404 for a missing asset.
3. Run `cargo test --test static_frontend`; expect failure until static files are under `static/`.
4. Move the field guide into `static/`, retaining the currently approved visual content.
5. Remove `static/demo-data.js` only after Tasks 2 and 3 connect the pages to the Rust API.
6. Run `cargo test`; expect all imported backend and frontend contracts to pass.
7. Commit with only `TLatAccenture` author/committer identity and no co-author trailer.

### Task 2: Harden the public API boundary

**Files:**
- Modify: `src/auth.rs`
- Modify: `src/main.rs`
- Modify: `Cargo.toml`
- Test: `src/auth.rs` unit tests
- Test: `tests/api_boundary.rs`

**Steps:**
1. Add failing tests for oversized JSON, overlong request/preference fields, rate-limit rejection, health/readiness bypass of expensive-route limits, and absence of raw request text in structured logs.
2. Add a bounded Axum body limit and explicit validation at `PipelineIn` and `DecideIn` trust boundaries.
3. Keep unauthenticated invocation for this public demo; do not ship a browser token.
4. Make rate-limit keys proxy-aware using the documented Cloud Run forwarding chain and test spoofed/malformed headers. If client identity is not trustworthy, use a global per-instance key rather than trusting caller-controlled input.
5. Add a `ponytail:` comment documenting that process-local limits are per-instance and must move to an edge/shared store above two instances.
6. Add request IDs and structured fields only: route, status, latency, decision outcome, and upstream error class. Never log raw request/preference text or response evidence bodies.
7. Run focused tests, then `cargo test`.
8. Commit the boundary hardening.

### Task 3: Bound live Parkinson’s retrieval

**Files:**
- Modify: `src/research.rs`
- Modify: `src/main.rs`
- Test: `src/research.rs` tests
- Test: `tests/api_boundary.rs`

**Steps:**
1. Add failing tests proving clinical dose/diagnosis/prescription requests cause zero fetches, connect/read timeout maps to an explicit unavailable result, non-HTTPS citation URLs are rejected, and result/page counts are capped.
2. Configure one reusable reqwest client with explicit connect and total timeouts, HTTPS-only redirects, bounded response handling, and a clear user agent.
3. Preserve source URLs and titles only after validation; never invent an evidence edge when retrieval fails.
4. Run research tests and the complete Rust suite.
5. Commit the retrieval bounds.

### Task 4: Connect the field guide to the same-origin API

**Files:**
- Modify: `static/dcceew/index.html`
- Modify: `static/parkinsons/index.html`
- Modify: `static/index.html`
- Delete: `static/demo-data.js`
- Test: `tests/static_frontend.rs`

**Steps:**
1. Add failing frontend contract tests requiring `/api/catalogue` and `/api/pipeline`, forbidding `demo-data.js`, and retaining the clinical and illustrative-data disclosures.
2. Restore same-origin fetches for DCCEEW and Parkinson’s with accessible working/error states.
3. Update publication copy so DCCEEW describes deterministic Rust rules and Parkinson’s accurately describes live public-source retrieval.
4. Ensure no API token, GCP project identifier, credential, analytics code, or raw personal data is present in browser assets.
5. Run `cargo test --test static_frontend` and `cargo test`.
6. Browser-test answer, clarify, reject, and unavailable states against a local server.
7. Commit the real-backend frontend.

### Task 5: Produce and test the container

**Files:**
- Create: `Dockerfile`
- Create: `.dockerignore`
- Modify: `README.md`
- Test: `scripts/smoke-container.sh`

**Steps:**
1. Write a failing smoke script that expects `/api/health`, `/api/ready`, `/`, and a DCCEEW pipeline answer from a container on a random local port.
2. Add a multi-stage Dockerfile pinned to a supported Rust toolchain and slim Debian runtime; copy only the binary, static files, and catalogue.
3. Run as a non-root user, consume `PORT`, set `CATALOGUE_PATH` and `STATIC_DIR`, and include no cloud SDK or credentials.
4. Build with `docker build --platform linux/amd64 -t graphs-and-jev:test .` and run the smoke script.
5. Inspect the image history and filesystem for credentials and unrelated source artifacts.
6. Run an available local image vulnerability scan and record tool/version/results in the deployment evidence.
7. Commit container delivery files.

### Task 6: Prepare GCP with least privilege

**GCP resources:**
- Project: `ninth-airship-386815`
- Region: `australia-southeast1`
- Artifact Registry: `graphs-and-jev`
- Cloud Run service: `graphs-and-jev`
- Runtime service account: `graphs-and-jev-runner`

**Steps:**
1. Confirm active account is `anthony.lui@archegon.com`, project billing is enabled, and the local branch is clean and pushed.
2. Enable `run.googleapis.com`, `artifactregistry.googleapis.com`, and `cloudbuild.googleapis.com`; record the commands and resulting enabled-service list.
3. Create the regional Docker repository only if absent.
4. Create a dedicated runtime service account only if absent; grant no project-wide role unless a verified runtime dependency requires it.
5. Verify Cloud Build’s principal has only the permissions required to write the repository and deploy Cloud Run.
6. Do not create secrets because this deployment has no browser/API token or hosted-model credential.

### Task 7: Build an immutable image and deploy without traffic

**Files:**
- Create: `deploy/cloud-run.env.example`
- Create: `deploy/deploy.sh`

**Steps:**
1. Write the deploy script with `set -euo pipefail`, explicit project/region/service/repository variables, and preflight account/project checks.
2. Submit the build tagged `australia-southeast1-docker.pkg.dev/ninth-airship-386815/graphs-and-jev/app:$GIT_SHA`.
3. Resolve and record the image digest; deployment must use the digest, not `latest`.
4. Deploy a new Cloud Run revision with no traffic, public invocation, min 0, max 2, concurrency 20, 1 CPU, 512 MiB, bounded request timeout, startup probe `/api/ready`, liveness probe `/api/health`, and the dedicated runtime account.
5. Record service URL, revision, image digest, and effective settings without recording credentials.
6. Commit scripts and deployment evidence template, not generated tokens or local environment files.

### Task 8: Verify the candidate revision and shift traffic

**Steps:**
1. Invoke the tagged candidate revision URL and verify health/readiness and all static routes.
2. Verify DCCEEW answer, clarify, and reject outcomes with exact representative requests.
3. Verify a Parkinson’s clinical request is rejected before retrieval and a research request returns validated HTTPS citations or an explicit bounded upstream failure.
4. Verify oversized requests return 413/422, rate limits return 429, and logs contain request IDs but not request text.
5. Run desktop and 390 px mobile browser checks with no console errors or horizontal overflow.
6. Shift 100% traffic to the candidate only after all gates pass.
7. Re-run checks against the canonical Cloud Run service URL.

### Task 9: Cut over publication metadata and disable Pages

**Files:**
- Modify: `README.md`
- Delete: `.github/workflows/pages.yml`

**Steps:**
1. Change README and repository homepage links to the verified Cloud Run URL.
2. Remove the Pages workflow and push; confirm this does not affect Cloud Run.
3. Disable GitHub Pages through the API only after the Cloud Run URL passes the full smoke suite.
4. Verify the repository remains public, contributor list remains only `TLatAccenture`, and Cloud Run is the sole advertised publication URL.
5. Commit the cutover.

### Task 10: Final evidence and rollback drill

**Files:**
- Create: `docs/deployment/cloud-run.md`

**Steps:**
1. Document architecture, project, region, service URL, revision, image digest, limits, egress, monitoring fields, and known per-instance rate-limit ceiling.
2. Document rollback: list revisions, route traffic to the previous healthy revision, and verify health/API/static routes.
3. Perform one traffic rollback and restore drill while both revisions are available.
4. Run final `cargo test`, release build, container smoke, live smoke, and repository cleanliness checks.
5. Confirm no secrets, tokens, raw PII, or authentication codes are committed or logged.
6. Commit the deployment record.
