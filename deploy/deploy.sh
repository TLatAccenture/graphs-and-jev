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
readonly IMAGE="${IMAGE:-app}"
readonly SOURCE_BUCKET="${SOURCE_BUCKET:-ninth-airship-386815-graphs-and-jev-build-source}"
readonly BUILD_SERVICE_ACCOUNT="${BUILD_SERVICE_ACCOUNT:-graphs-and-jev-builder@${PROJECT_ID}.iam.gserviceaccount.com}"
readonly RUNTIME_SERVICE_ACCOUNT="${RUNTIME_SERVICE_ACCOUNT:-graphs-and-jev-runner@${PROJECT_ID}.iam.gserviceaccount.com}"
readonly EXPECTED_ACCOUNT="${EXPECTED_ACCOUNT:-anthony.lui@archegon.com}"
readonly DEPLOY_OUTPUT="${DEPLOY_OUTPUT:-${TMPDIR:-/tmp}/graphs-and-jev-deployment-result.json}"
readonly PROJECT_RESOURCE="//cloudresourcemanager.googleapis.com/projects/${PROJECT_ID}"
readonly REPOSITORY_RESOURCE="//artifactregistry.googleapis.com/projects/${PROJECT_ID}/locations/${REGION}/repositories/${ARTIFACT_REPOSITORY}"
readonly RUNTIME_SA_RESOURCE="//iam.googleapis.com/projects/${PROJECT_ID}/serviceAccounts/${RUNTIME_SERVICE_ACCOUNT}"
readonly SOURCE_BUCKET_RESOURCE="//storage.googleapis.com/projects/_/buckets/${SOURCE_BUCKET}"
readonly MIN_INSTANCES="${MIN_INSTANCES:-0}"
readonly MAX_INSTANCES="${MAX_INSTANCES:-2}"
readonly CONCURRENCY="${CONCURRENCY:-20}"
readonly CPU="${CPU:-1}"
readonly MEMORY="${MEMORY:-512Mi}"
readonly TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-30}"
readonly STARTUP_PATH="${STARTUP_PATH:-/api/ready}"
readonly LIVENESS_PATH="${LIVENESS_PATH:-/api/health}"

for command_name in gcloud git python3; do
  command -v "$command_name" >/dev/null || { printf '%s is required\n' "$command_name" >&2; exit 1; }
done

fail() { printf 'preflight failed: %s\n' "$*" >&2; exit 1; }
require_permission() {
  local principal="$1" resource="$2" permission="$3" state
  state="$(gcloud policy-intelligence troubleshoot-policy iam "$resource" \
    --project="$PROJECT_ID" \
    --principal-email="$principal" \
    --permission="$permission" \
    --format='value(allowPolicyExplanation.allowAccessState)')"
  [[ "$state" == "ALLOW_ACCESS_STATE_GRANTED" ]] || fail "$principal lacks $permission on $resource (state: ${state:-unknown})"
}
require_bucket_permission() {
  local principal="$1" permission="$2" state
  state="$(gcloud policy-intelligence troubleshoot-policy iam "$SOURCE_BUCKET_RESOURCE" \
    --project="$PROJECT_ID" \
    --principal-email="$principal" \
    --permission="$permission" \
    --resource-name="${SOURCE_BUCKET_RESOURCE}/objects/source-preflight" \
    --resource-service=storage.googleapis.com \
    --resource-type=storage.googleapis.com/Object \
    --format='value(allowPolicyExplanation.allowAccessState)')"
  [[ "$state" == "ALLOW_ACCESS_STATE_GRANTED" ]] || fail "$principal lacks $permission on gs://$SOURCE_BUCKET (state: ${state:-unknown})"
}

active_account="$(gcloud auth list --filter=status:ACTIVE --format='value(account)')"
[[ "$active_account" == "$EXPECTED_ACCOUNT" ]] || fail "active account must be $EXPECTED_ACCOUNT (found ${active_account:-none})"
active_project="$(gcloud config get-value project 2>/dev/null)"
[[ "$active_project" == "$PROJECT_ID" ]] || fail "active project must be $PROJECT_ID (found ${active_project:-none})"

billing_enabled="$(gcloud beta billing projects describe "$PROJECT_ID" --format='value(billingEnabled)')"
[[ "$billing_enabled" == "True" ]] || fail "billing is not enabled for $PROJECT_ID"

readonly required_apis=(
  artifactregistry.googleapis.com
  cloudbuild.googleapis.com
  iam.googleapis.com
  logging.googleapis.com
  policytroubleshooter.googleapis.com
  run.googleapis.com
  serviceusage.googleapis.com
  storage.googleapis.com
)
enabled_apis="$(gcloud services list --enabled --project="$PROJECT_ID" --format='value(config.name)')"
for api in "${required_apis[@]}"; do
  grep -Fxq "$api" <<<"$enabled_apis" || fail "required API is not enabled: $api"
done

repository_state="$(gcloud artifacts repositories describe "$ARTIFACT_REPOSITORY" \
  --project="$PROJECT_ID" --location="$REGION" --format='value(name,format)')"
readonly expected_repository_state="projects/${PROJECT_ID}/locations/${REGION}/repositories/${ARTIFACT_REPOSITORY} DOCKER"
[[ "$repository_state" == "$expected_repository_state" ]] || fail "Artifact Registry repository must be DOCKER in $REGION (found ${repository_state:-none})"
gcloud iam service-accounts describe "$BUILD_SERVICE_ACCOUNT" --project="$PROJECT_ID" --format='value(email)' | grep -Fxq "$BUILD_SERVICE_ACCOUNT" || fail "builder service account does not exist"
gcloud iam service-accounts describe "$RUNTIME_SERVICE_ACCOUNT" --project="$PROJECT_ID" --format='value(email)' | grep -Fxq "$RUNTIME_SERVICE_ACCOUNT" || fail "runtime service account does not exist"

bucket_json="$(gcloud storage buckets describe "gs://${SOURCE_BUCKET}" --project="$PROJECT_ID" --format=json)"
python3 -c 'import json,sys
bucket=json.load(sys.stdin)
expected=sys.argv[1].upper()
checks={
    "location": bucket.get("location") == expected,
    "regional location type": bucket.get("location_type") == "region",
    "uniform bucket-level access": bucket.get("uniform_bucket_level_access") is True,
    "public access prevention": bucket.get("public_access_prevention") == "enforced",
}
failed=[name for name, ok in checks.items() if not ok]
if failed:
    raise SystemExit("source bucket preflight failed: " + ", ".join(failed))' "$REGION" <<<"$bucket_json"

# The builder only writes build logs, consumes enabled services, pushes the image, and reads staged source.
require_permission "$BUILD_SERVICE_ACCOUNT" "$PROJECT_RESOURCE" logging.logEntries.create
require_permission "$BUILD_SERVICE_ACCOUNT" "$PROJECT_RESOURCE" serviceusage.services.use
require_permission "$BUILD_SERVICE_ACCOUNT" "$REPOSITORY_RESOURCE" artifactregistry.repositories.uploadArtifacts
require_bucket_permission "$BUILD_SERVICE_ACCOUNT" storage.objects.get

# The authenticated human submits and deploys, reads the resulting image/state, publishes IAM, and attaches the runtime identity.
require_permission "$active_account" "$PROJECT_RESOURCE" cloudbuild.builds.create
require_bucket_permission "$active_account" storage.objects.create
require_permission "$active_account" "$REPOSITORY_RESOURCE" artifactregistry.dockerimages.get
require_permission "$active_account" "$PROJECT_RESOURCE" run.services.create
require_permission "$active_account" "$PROJECT_RESOURCE" run.services.update
require_permission "$active_account" "$PROJECT_RESOURCE" run.services.setIamPolicy
require_permission "$active_account" "//iam.googleapis.com/projects/${PROJECT_ID}/serviceAccounts/${BUILD_SERVICE_ACCOUNT}" iam.serviceAccounts.actAs
require_permission "$active_account" "$RUNTIME_SA_RESOURCE" iam.serviceAccounts.actAs

[[ -z "$(git status --porcelain --untracked-files=normal)" ]] || fail 'worktree must be clean'
git diff-index --quiet HEAD -- || fail 'tracked files differ from HEAD'
branch="$(git symbolic-ref --quiet --short HEAD)" || fail 'deployment requires a named branch'
git fetch --quiet origin "$branch"
git merge-base --is-ancestor HEAD "origin/$branch" || fail "HEAD is not pushed to origin/$branch"
[[ "$(git rev-parse HEAD)" == "$(git rev-parse "origin/$branch")" ]] || fail "HEAD must exactly match origin/$branch"

readonly git_sha
readonly revision_suffix
readonly image_tag
readonly build_sa_resource="projects/${PROJECT_ID}/serviceAccounts/${BUILD_SERVICE_ACCOUNT}"
git_sha="$(git rev-parse HEAD)"
revision_suffix="${git_sha:0:12}"
image_tag="${REGION}-docker.pkg.dev/${PROJECT_ID}/${ARTIFACT_REPOSITORY}/${IMAGE}:${git_sha}"

gcloud builds submit . \
  --project="$PROJECT_ID" \
  --region="$REGION" \
  --service-account="$build_sa_resource" \
  --default-buckets-behavior=regional-user-owned-bucket \
  --gcs-source-staging-dir="gs://${SOURCE_BUCKET}/source" \
  --tag="$image_tag"

digest="$(gcloud artifacts docker images describe "$image_tag" \
  --project="$PROJECT_ID" --format='value(image_summary.digest)')"
[[ "$digest" =~ ^sha256:[0-9a-f]{64}$ ]] || fail "Artifact Registry returned an invalid digest: $digest"
readonly image_digest="${REGION}-docker.pkg.dev/${PROJECT_ID}/${ARTIFACT_REPOSITORY}/${IMAGE}@${digest}"

gcloud beta run deploy "$SERVICE" \
  --project="$PROJECT_ID" \
  --region="$REGION" \
  --image="$image_digest" \
  --revision-suffix="$revision_suffix" \
  --no-traffic \
  --allow-unauthenticated \
  --min-instances="$MIN_INSTANCES" \
  --max-instances="$MAX_INSTANCES" \
  --concurrency="$CONCURRENCY" \
  --cpu="$CPU" \
  --memory="$MEMORY" \
  --timeout="${TIMEOUT_SECONDS}s" \
  --port=8080 \
  --startup-probe="initialDelaySeconds=0,timeoutSeconds=2,periodSeconds=2,failureThreshold=15,httpGet.port=8080,httpGet.path=${STARTUP_PATH}" \
  --liveness-probe="initialDelaySeconds=10,timeoutSeconds=2,periodSeconds=10,failureThreshold=3,httpGet.port=8080,httpGet.path=${LIVENESS_PATH}" \
  --service-account="$RUNTIME_SERVICE_ACCOUNT" \
  --quiet

mkdir -p "$(dirname "$DEPLOY_OUTPUT")"
service_json="$(mktemp)"
revision_json="$(mktemp)"
iam_json="$(mktemp)"
tmp_output="$(mktemp "${DEPLOY_OUTPUT}.tmp.XXXXXX")"
trap 'rm -f "$service_json" "$revision_json" "$iam_json" "$tmp_output"' EXIT
chmod 600 "$service_json" "$revision_json" "$iam_json" "$tmp_output"
gcloud run services describe "$SERVICE" --project="$PROJECT_ID" --region="$REGION" --format=json >"$service_json"
revision="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["status"]["latestCreatedRevisionName"])' "$service_json")"
[[ "$revision" == "${SERVICE}-${revision_suffix}" ]] || fail "created revision $revision does not match commit suffix $revision_suffix"
gcloud run revisions describe "$revision" --project="$PROJECT_ID" --region="$REGION" --format=json >"$revision_json"
gcloud run services get-iam-policy "$SERVICE" --project="$PROJECT_ID" --region="$REGION" --format=json >"$iam_json"

export EFFECTIVE_SETTINGS_JSON="$tmp_output"
python3 - "$service_json" "$revision_json" "$iam_json" "$image_digest" "$RUNTIME_SERVICE_ACCOUNT" "$revision" "$MIN_INSTANCES" "$MAX_INSTANCES" "$CONCURRENCY" "$CPU" "$MEMORY" "$TIMEOUT_SECONDS" "$STARTUP_PATH" "$LIVENESS_PATH" <<'PY'
import json
import os
import sys

service = json.load(open(sys.argv[1], encoding="utf-8"))
revision = json.load(open(sys.argv[2], encoding="utf-8"))
iam = json.load(open(sys.argv[3], encoding="utf-8"))
expected_image, expected_sa, expected_revision = sys.argv[4:7]
expected_min, expected_max, expected_concurrency, expected_cpu, expected_memory, expected_timeout = sys.argv[7:13]
expected_startup, expected_liveness = sys.argv[13:15]
metadata = revision["metadata"]
spec = revision["spec"]
container = spec["containers"][0]
annotations = metadata.get("annotations", {})
limits = container.get("resources", {}).get("limits", {})
traffic = service.get("status", {}).get("traffic", [])
public = any(
    binding.get("role") == "roles/run.invoker" and "allUsers" in binding.get("members", [])
    for binding in iam.get("bindings", [])
)
actual = {
    "service": service["metadata"]["name"],
    "service_url": service["status"]["url"],
    "revision": metadata["name"],
    "image_digest": container["image"],
    "runtime_service_account": spec["serviceAccountName"],
    "min_instances": int(annotations.get("autoscaling.knative.dev/minScale", "0")),
    "max_instances": int(annotations["autoscaling.knative.dev/maxScale"]),
    "concurrency": spec["containerConcurrency"],
    "cpu": limits["cpu"],
    "memory": limits["memory"],
    "timeout_seconds": int(spec["timeoutSeconds"]),
    "startup_probe": container["startupProbe"],
    "liveness_probe": container["livenessProbe"],
    "traffic": traffic,
    "public_invocation": public,
}

def assert_effective_deployment(condition, message):
    if not condition:
        raise SystemExit(f"effective deployment mismatch: {message}")

assert_effective_deployment(actual["revision"] == expected_revision, "revision")
assert_effective_deployment(actual["image_digest"] == expected_image, "image digest")
assert_effective_deployment(actual["runtime_service_account"] == expected_sa, "runtime service account")
assert_effective_deployment(actual["min_instances"] == int(expected_min) and actual["max_instances"] == int(expected_max), "instance bounds")
assert_effective_deployment(actual["concurrency"] == int(expected_concurrency), "concurrency")
actual_cpu = str(actual["cpu"])
assert_effective_deployment(actual_cpu in {expected_cpu, f"{int(expected_cpu) * 1000}m"} and actual["memory"] == expected_memory, "CPU or memory")
assert_effective_deployment(actual["timeout_seconds"] == int(expected_timeout), "timeout")
assert_effective_deployment(actual["startup_probe"].get("httpGet", {}).get("path") == expected_startup, "startup probe")
assert_effective_deployment(actual["liveness_probe"].get("httpGet", {}).get("path") == expected_liveness, "liveness probe")
assert_effective_deployment(not any(item.get("revisionName") == expected_revision and item.get("percent", 0) for item in traffic), "new revision has traffic")
assert_effective_deployment(actual["public_invocation"], "public invocation IAM")
with open(os.environ["EFFECTIVE_SETTINGS_JSON"], "w", encoding="utf-8") as output:
    json.dump(actual, output, indent=2, sort_keys=True)
    output.write("\n")
PY

mv "$tmp_output" "$DEPLOY_OUTPUT"
trap - EXIT
rm -f "$service_json" "$revision_json" "$iam_json"
printf 'Validated revision %s at 0%% traffic\n' "$revision"
printf 'Captured verified effective settings in %s\n' "$DEPLOY_OUTPUT"
