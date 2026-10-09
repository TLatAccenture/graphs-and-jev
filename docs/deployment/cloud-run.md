# Cloud Run deployment record

This record captures the verified production state on 9 October 2026. Re-run the commands below before relying on it after a deployment.

## Published service

- GCP project: `ninth-airship-386815`
- Region: `australia-southeast1`
- Service: `graphs-and-jev`
- Canonical URL: <https://graphs-and-jev-956922431929.australia-southeast1.run.app>
- Active revision: `graphs-and-jev-e7cead061975`
- Source commit: `e7cead061975dcd120dc113d72e25f0cbc818f6d`
- Immutable image: `australia-southeast1-docker.pkg.dev/ninth-airship-386815/graphs-and-jev/app@sha256:79f5fab2630f04682c59cf2a3f1be64304ef8260dbee9b97c89e09c5f2b615f3`
- Runtime identity: `graphs-and-jev-runner@ninth-airship-386815.iam.gserviceaccount.com`

The single Rust process serves the static field guide and same-origin API. It runs from a `scratch` image as UID/GID 65532. Cloud Run ingress is `all`; `allUsers` has `roles/run.invoker`, so the publication is intentionally public. GitHub PR [#1](https://github.com/TLatAccenture/graphs-and-jev/pull/1) is merged, the Pages workflow has been removed, and the GitHub Pages API returns 404 for this repository.

## Effective runtime configuration

The active revision reports minimum instances 0, maximum instances 2, concurrency 20, 1 CPU, 512 MiB memory, and a 30-second request timeout. Startup and liveness probes use `/api/ready` and `/api/health`. Inspect the revision rather than relying on the service's historical top-level `run.googleapis.com/maxScale` annotation:

```bash
gcloud run revisions describe graphs-and-jev-e7cead061975 \
  --project=ninth-airship-386815 \
  --region=australia-southeast1 \
  --format=yaml
```

`RATE_LIMIT_PER_MINUTE` is not set in the revision, so the compiled default is 120 expensive requests per minute. The limiter is process-local and global within each instance. With at most two instances, the aggregate ceiling can therefore reach 240 requests per minute and is not a stable per-client quota. Move enforcement to an edge or shared store before increasing the instance limit or requiring client-level guarantees.

The service makes bounded HTTPS requests to Wikipedia and Europe PMC for the Parkinson's research path. A live canonical-URL request returned citations from both providers. No hosted-model credential or API token is present in the runtime configuration.

## Build provenance and access

Successful Cloud Build IDs for the promoted runtime lineage were:

- `47024d2f-1936-4ecc-b461-572bec8a0f2c` — commit `e48a4869dbcc91e84f41e988e791117ed8f3d918`, distroless candidate, not promoted.
- `336f2ec2-cae7-4095-8312-caa84e0931a5` — commit `6a52f35f2aa793b24f1aa952cb7ff01f48dde059`, first scratch candidate.
- `9a059894-f911-4c7f-8471-a35dcc10fdf0` — commit `e7cead061975dcd120dc113d72e25f0cbc818f6d`, current scratch image.

Cloud Build staged source in `gs://ninth-airship-386815-graphs-and-jev-build-source/source/` and wrote logs to `gs://956922431929-australia-southeast1-cloudbuild-logs/logs`. Both buckets are regional, use uniform bucket-level access, and enforce public-access prevention. The dedicated builder is `graphs-and-jev-builder@ninth-airship-386815.iam.gserviceaccount.com`; verified grants are Artifact Registry writer on the repository, project-level log writer and Service Usage consumer, source-object read access, and log-bucket object management required by Cloud Build. The runtime service account has no verified project-wide role.

## Security scan evidence

The Debian slim and pinned distroless runtime candidates reported critical/high operating-system findings in image scanning and were not promoted. The current runtime is `scratch`. An OS-package scanner cannot establish an OS vulnerability result for it because the image contains no package database or OS metadata; this is not evidence of zero CVEs.

The compensating checks are exact and reproducible:

- the final root filesystem allowlist permits only `/app/serve`, `/app/catalogue.json`, and `/app/static/**`;
- `/app/serve` must be a static x86-64 ELF with no `INTERP` program header and no `NEEDED` dynamic dependencies;
- the container runs as numeric non-root UID/GID 65532;
- `cargo audit` scanned 154 locked dependencies against 1,295 RustSec advisories and exited cleanly on 9 October 2026.

Run `IMAGE=graphs-and-jev:test scripts/smoke-container.sh` to repeat the root-filesystem, static-ELF, identity, health, API, static-page, and live-retrieval checks.

## Logging and monitoring

Application logs are one JSON object per request with `request_id`, `route`, `status`, `latency_ms`, `outcome`, and `upstream_error_class`. The API returns the request ID in `x-request-id`. Tests assert that request and preference text do not enter structured logs, and upstream error messages are reduced to a class. Cloud Run platform request logs remain separate from these application records.

Do not add raw request text, response evidence bodies, credentials, tokens, or personal data to logs. Monitor request status and latency by route, decision outcomes, upstream error classes, revision health, instance count, and cost.

## Release verification gates

Do not send production traffic to a candidate until all gates pass:

1. `cargo test` and `cargo build --release` succeed from the committed tree.
2. The local scratch image builds and `scripts/smoke-container.sh` passes.
3. The image is addressed by digest and its revision reports the expected account, probes, limits, and zero candidate traffic.
4. Candidate checks pass for health, readiness, all static routes, DCCEEW answer/clarify/reject, Parkinson's clinical rejection before retrieval, and either HTTPS research citations or an explicit bounded upstream failure.
5. Boundary checks return the expected 413/422 and 429 responses; logs contain request IDs and no submitted request text.
6. Desktop and 390 px browser checks have no console errors or horizontal overflow.
7. Canonical-URL checks pass after the traffic update.
8. Git status is clean and tracked browser assets contain no secret, token, credential, GCP project ID, analytics identifier, authentication code, or raw personal data.

## Rollback and restore

List available revisions:

```bash
gcloud run revisions list \
  --service=graphs-and-jev \
  --project=ninth-airship-386815 \
  --region=australia-southeast1
```

Route all traffic to a known healthy revision:

```bash
gcloud run services update-traffic graphs-and-jev \
  --project=ninth-airship-386815 \
  --region=australia-southeast1 \
  --to-revisions=graphs-and-jev-6a52f35f2aa7=100
```

Verify `/api/health`, `/api/ready`, `/`, `/dcceew/`, `/parkinsons/`, `/little-bunnies/`, and representative API outcomes. Restore the current revision with:

```bash
gcloud run services update-traffic graphs-and-jev \
  --project=ninth-airship-386815 \
  --region=australia-southeast1 \
  --to-revisions=graphs-and-jev-e7cead061975=100
```

The completed drill moved 100% traffic from `graphs-and-jev-e7cead061975` to `graphs-and-jev-6a52f35f2aa7`, verified the older revision, then restored and re-verified `graphs-and-jev-e7cead061975`. Live state after the drill is restored: the `e7cead` revision is ready and receives 100% of traffic.
