#!/usr/bin/env bash
set -euo pipefail

readonly ROOT
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if [[ $# -gt 1 ]]; then
  printf 'usage: %s [environment-file]\n' "$0" >&2
  exit 64
fi
if [[ $# -eq 1 ]]; then
  # shellcheck disable=SC1090
  source "$1"
fi

readonly PROJECT_ID="${PROJECT_ID:-ninth-airship-386815}"
readonly REGION="${REGION:-australia-southeast1}"
readonly SERVICE="${SERVICE:-graphs-and-jev}"
readonly ARTIFACT_REPOSITORY="${ARTIFACT_REPOSITORY:-graphs-and-jev}"
readonly IMAGE="${IMAGE:-graphs-and-jev}"
readonly BUILD_SERVICE_ACCOUNT="${BUILD_SERVICE_ACCOUNT:-graphs-and-jev-builder@${PROJECT_ID}.iam.gserviceaccount.com}"
readonly RUNTIME_SERVICE_ACCOUNT="${RUNTIME_SERVICE_ACCOUNT:-graphs-and-jev-runner@${PROJECT_ID}.iam.gserviceaccount.com}"
readonly EXPECTED_ACCOUNT="${EXPECTED_ACCOUNT:-anthony.lui@archegon.com}"
readonly DEPLOY_OUTPUT="${DEPLOY_OUTPUT:-${TMPDIR:-/tmp}/graphs-and-jev-deployment-result.env}"

command -v gcloud >/dev/null || { printf 'gcloud is required\n' >&2; exit 1; }
command -v git >/dev/null || { printf 'git is required\n' >&2; exit 1; }

active_account="$(gcloud auth list --filter=status:ACTIVE --format='value(account)')"
[[ "$active_account" == "$EXPECTED_ACCOUNT" ]] || {
  printf 'active gcloud account must be %s (found %s)\n' "$EXPECTED_ACCOUNT" "${active_account:-none}" >&2
  exit 1
}
active_project="$(gcloud config get-value project 2>/dev/null)"
[[ "$active_project" == "$PROJECT_ID" ]] || {
  printf 'active gcloud project must be %s (found %s)\n' "$PROJECT_ID" "${active_project:-none}" >&2
  exit 1
}

[[ -z "$(git status --porcelain --untracked-files=normal)" ]] || {
  printf 'worktree must be clean before deployment\n' >&2
  exit 1
}
git diff-index --quiet HEAD -- || { printf 'tracked files differ from HEAD\n' >&2; exit 1; }
branch="$(git symbolic-ref --quiet --short HEAD)" || {
  printf 'deployment requires a named branch\n' >&2
  exit 1
}
git fetch --quiet origin "$branch"
git merge-base --is-ancestor HEAD "origin/$branch" || {
  printf 'HEAD is not pushed to origin/%s\n' "$branch" >&2
  exit 1
}
[[ "$(git rev-parse HEAD)" == "$(git rev-parse "origin/$branch")" ]] || {
  printf 'HEAD must exactly match origin/%s\n' "$branch" >&2
  exit 1
}

readonly git_sha
git_sha="$(git rev-parse HEAD)"
readonly revision_suffix="${git_sha:0:12}"
readonly image_tag="${REGION}-docker.pkg.dev/${PROJECT_ID}/${ARTIFACT_REPOSITORY}/${IMAGE}:${git_sha}"
readonly build_sa_resource="projects/${PROJECT_ID}/serviceAccounts/${BUILD_SERVICE_ACCOUNT}"

gcloud builds submit . \
  --project="$PROJECT_ID" \
  --region="$REGION" \
  --service-account="$build_sa_resource" \
  --tag="$image_tag"

digest="$(gcloud artifacts docker images describe "$image_tag" \
  --project="$PROJECT_ID" \
  --format='value(image_summary.digest)')"
[[ "$digest" =~ ^sha256:[0-9a-f]{64}$ ]] || {
  printf 'Artifact Registry returned an invalid digest: %s\n' "$digest" >&2
  exit 1
}
readonly image_digest="${REGION}-docker.pkg.dev/${PROJECT_ID}/${ARTIFACT_REPOSITORY}/${IMAGE}@${digest}"

revision="$(gcloud beta run deploy "$SERVICE" \
  --project="$PROJECT_ID" \
  --region="$REGION" \
  --image="$image_digest" \
  --revision-suffix="$revision_suffix" \
  --no-traffic \
  --allow-unauthenticated \
  --min-instances=0 \
  --max-instances=2 \
  --concurrency=20 \
  --cpu=1 \
  --memory=512Mi \
  --timeout=30s \
  --port=8080 \
  --startup-probe='initialDelaySeconds=0,timeoutSeconds=2,periodSeconds=2,failureThreshold=15,httpGet.port=8080,httpGet.path=/api/ready' \
  --liveness-probe='initialDelaySeconds=10,timeoutSeconds=2,periodSeconds=10,failureThreshold=3,httpGet.port=8080,httpGet.path=/api/health' \
  --service-account="$RUNTIME_SERVICE_ACCOUNT" \
  --format='value(metadata.name)')"
[[ -n "$revision" ]] || { printf 'Cloud Run returned no revision name\n' >&2; exit 1; }

service_url="$(gcloud run services describe "$SERVICE" \
  --project="$PROJECT_ID" \
  --region="$REGION" \
  --format='value(status.url)')"
[[ "$service_url" == https://* ]] || { printf 'Cloud Run returned an invalid service URL\n' >&2; exit 1; }

mkdir -p "$(dirname "$DEPLOY_OUTPUT")"
tmp_output="$(mktemp "${DEPLOY_OUTPUT}.tmp.XXXXXX")"
trap 'rm -f "$tmp_output"' EXIT
chmod 600 "$tmp_output"
{
  printf 'PROJECT_ID=%q\n' "$PROJECT_ID"
  printf 'REGION=%q\n' "$REGION"
  printf 'SERVICE=%q\n' "$SERVICE"
  printf 'SERVICE_URL=%q\n' "$service_url"
  printf 'REVISION=%q\n' "$revision"
  printf 'GIT_SHA=%q\n' "$git_sha"
  printf 'IMAGE_DIGEST=%q\n' "$image_digest"
  printf 'RUNTIME_SERVICE_ACCOUNT=%q\n' "$RUNTIME_SERVICE_ACCOUNT"
  printf 'TRAFFIC=%q\n' '0'
  printf 'PUBLIC_INVOCATION=%q\n' 'true'
  printf 'MIN_INSTANCES=%q\n' '0'
  printf 'MAX_INSTANCES=%q\n' '2'
  printf 'CONCURRENCY=%q\n' '20'
  printf 'CPU=%q\n' '1'
  printf 'MEMORY=%q\n' '512Mi'
  printf 'TIMEOUT=%q\n' '30s'
  printf 'STARTUP_PROBE=%q\n' '/api/ready'
  printf 'LIVENESS_PROBE=%q\n' '/api/health'
} > "$tmp_output"
mv "$tmp_output" "$DEPLOY_OUTPUT"
trap - EXIT

printf 'Deployed revision %s at 0%% traffic\n' "$revision"
printf 'Service URL: %s\n' "$service_url"
printf 'Image: %s\n' "$image_digest"
printf 'Captured non-secret deployment metadata in %s\n' "$DEPLOY_OUTPUT"
