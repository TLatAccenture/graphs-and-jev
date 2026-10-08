#!/usr/bin/env bash
set -euo pipefail

GJ_REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly GJ_REPO_ROOT
cd "$GJ_REPO_ROOT"

if [[ $# -gt 1 ]]; then
  printf 'usage: %s [environment-file]\n' "$0" >&2
  exit 64
fi
if [[ $# -eq 1 ]]; then
  # shellcheck disable=SC1090
  source "$1"
fi

readonly GJ_PROJECT_ID="${PROJECT_ID:-ninth-airship-386815}"
readonly GJ_REGION="${REGION:-australia-southeast1}"
readonly GJ_SERVICE="${SERVICE:-graphs-and-jev}"
readonly GJ_ARTIFACT_REPOSITORY="${ARTIFACT_REPOSITORY:-graphs-and-jev}"
readonly GJ_IMAGE="${IMAGE:-app}"
readonly GJ_SOURCE_BUCKET="${SOURCE_BUCKET:-ninth-airship-386815-graphs-and-jev-build-source}"
readonly GJ_BUILD_SERVICE_ACCOUNT="${BUILD_SERVICE_ACCOUNT:-graphs-and-jev-builder@${GJ_PROJECT_ID}.iam.gserviceaccount.com}"
readonly GJ_RUNTIME_SERVICE_ACCOUNT="${RUNTIME_SERVICE_ACCOUNT:-graphs-and-jev-runner@${GJ_PROJECT_ID}.iam.gserviceaccount.com}"
readonly GJ_EXPECTED_ACCOUNT="${EXPECTED_ACCOUNT:-anthony.lui@archegon.com}"
readonly GJ_DEPLOY_OUTPUT="${DEPLOY_OUTPUT:-${TMPDIR:-/tmp}/graphs-and-jev-deployment-result.json}"
readonly GJ_PROJECT_RESOURCE="//cloudresourcemanager.googleapis.com/projects/${GJ_PROJECT_ID}"
readonly GJ_REPOSITORY_RESOURCE="//artifactregistry.googleapis.com/projects/${GJ_PROJECT_ID}/locations/${GJ_REGION}/repositories/${GJ_ARTIFACT_REPOSITORY}"
readonly GJ_RUNTIME_SA_RESOURCE="//iam.googleapis.com/projects/${GJ_PROJECT_ID}/serviceAccounts/${GJ_RUNTIME_SERVICE_ACCOUNT}"
readonly GJ_SOURCE_BUCKET_RESOURCE="//storage.googleapis.com/projects/_/buckets/${GJ_SOURCE_BUCKET}"
readonly GJ_MIN_INSTANCES="${MIN_INSTANCES:-0}"
readonly GJ_MAX_INSTANCES="${MAX_INSTANCES:-2}"
readonly GJ_CONCURRENCY="${CONCURRENCY:-20}"
readonly GJ_CPU="${CPU:-1}"
readonly GJ_MEMORY="${MEMORY:-512Mi}"
readonly GJ_TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-30}"
readonly GJ_STARTUP_PATH="${STARTUP_PATH:-/api/ready}"
readonly GJ_LIVENESS_PATH="${LIVENESS_PATH:-/api/health}"

for GJ_command_name in gcloud git python3; do
  command -v "$GJ_command_name" >/dev/null || { printf '%s is required\n' "$GJ_command_name" >&2; exit 1; }
done

fail() { printf 'preflight failed: %s\n' "$*" >&2; exit 1; }
require_permission() {
  local principal="$1" resource="$2" permission="$3" state
  state="$(gcloud policy-intelligence troubleshoot-policy iam "$resource" \
    --project="$GJ_PROJECT_ID" \
    --principal-email="$principal" \
    --permission="$permission" \
    --format='value(allowPolicyExplanation.allowAccessState)')"
  [[ "$state" == "ALLOW_ACCESS_STATE_GRANTED" ]] || fail "$principal lacks $permission on $resource (state: ${state:-unknown})"
}
require_bucket_permission() {
  local principal="$1" permission="$2" state
  state="$(gcloud policy-intelligence troubleshoot-policy iam "$GJ_SOURCE_BUCKET_RESOURCE" \
    --project="$GJ_PROJECT_ID" \
    --principal-email="$principal" \
    --permission="$permission" \
    --resource-name="${GJ_SOURCE_BUCKET_RESOURCE}/objects/source-preflight" \
    --resource-service=storage.googleapis.com \
    --resource-type=storage.googleapis.com/Object \
    --format='value(allowPolicyExplanation.allowAccessState)')"
  [[ "$state" == "ALLOW_ACCESS_STATE_GRANTED" ]] || fail "$principal lacks $permission on gs://$GJ_SOURCE_BUCKET (state: ${state:-unknown})"
}

GJ_active_account="$(gcloud auth list --filter=status:ACTIVE --format='value(account)')"
[[ "$GJ_active_account" == "$GJ_EXPECTED_ACCOUNT" ]] || fail "active account must be $GJ_EXPECTED_ACCOUNT (found ${GJ_active_account:-none})"
GJ_active_project="$(gcloud config get-value project 2>/dev/null)"
[[ "$GJ_active_project" == "$GJ_PROJECT_ID" ]] || fail "active project must be $GJ_PROJECT_ID (found ${GJ_active_project:-none})"

GJ_billing_enabled="$(gcloud beta billing projects describe "$GJ_PROJECT_ID" --format='value(billingEnabled)')"
[[ "$GJ_billing_enabled" == "True" ]] || fail "billing is not enabled for $GJ_PROJECT_ID"

readonly GJ_required_apis=(
  artifactregistry.googleapis.com
  cloudbuild.googleapis.com
  iam.googleapis.com
  logging.googleapis.com
  policytroubleshooter.googleapis.com
  run.googleapis.com
  serviceusage.googleapis.com
  storage.googleapis.com
)
GJ_enabled_apis="$(gcloud services list --enabled --project="$GJ_PROJECT_ID" --format='value(config.name)')"
for GJ_api in "${GJ_required_apis[@]}"; do
  grep -Fxq "$GJ_api" <<<"$GJ_enabled_apis" || fail "required API is not enabled: $GJ_api"
done

GJ_repository_name="$(gcloud artifacts repositories describe "$GJ_ARTIFACT_REPOSITORY" \
  --project="$GJ_PROJECT_ID" --location="$GJ_REGION" --format='value(name)')"
GJ_repository_format="$(gcloud artifacts repositories describe "$GJ_ARTIFACT_REPOSITORY" \
  --project="$GJ_PROJECT_ID" --location="$GJ_REGION" --format='value(format)')"
readonly GJ_expected_repository_name="projects/${GJ_PROJECT_ID}/locations/${GJ_REGION}/repositories/${GJ_ARTIFACT_REPOSITORY}"
[[ "$GJ_repository_name" == "$GJ_expected_repository_name" ]] || fail "Artifact Registry repository is not $GJ_expected_repository_name (found ${GJ_repository_name:-none})"
[[ "$GJ_repository_format" == "DOCKER" ]] || fail "Artifact Registry repository must use DOCKER format (found ${GJ_repository_format:-none})"
gcloud iam service-accounts describe "$GJ_BUILD_SERVICE_ACCOUNT" --project="$GJ_PROJECT_ID" --format='value(email)' | grep -Fxq "$GJ_BUILD_SERVICE_ACCOUNT" || fail "builder service account does not exist"
gcloud iam service-accounts describe "$GJ_RUNTIME_SERVICE_ACCOUNT" --project="$GJ_PROJECT_ID" --format='value(email)' | grep -Fxq "$GJ_RUNTIME_SERVICE_ACCOUNT" || fail "runtime service account does not exist"

GJ_bucket_json="$(gcloud storage buckets describe "gs://${GJ_SOURCE_BUCKET}" --project="$GJ_PROJECT_ID" --format=json)"
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
    raise SystemExit("source bucket preflight failed: " + ", ".join(failed))' "$GJ_REGION" <<<"$GJ_bucket_json"

# The builder only writes build logs, consumes enabled services, pushes the image, and reads staged source.
require_permission "$GJ_BUILD_SERVICE_ACCOUNT" "$GJ_PROJECT_RESOURCE" logging.logEntries.create
require_permission "$GJ_BUILD_SERVICE_ACCOUNT" "$GJ_PROJECT_RESOURCE" serviceusage.services.use
require_permission "$GJ_BUILD_SERVICE_ACCOUNT" "$GJ_REPOSITORY_RESOURCE" artifactregistry.repositories.uploadArtifacts
require_bucket_permission "$GJ_BUILD_SERVICE_ACCOUNT" storage.objects.get

# The authenticated human submits and deploys, reads the resulting image/state, publishes IAM, and attaches the runtime identity.
require_permission "$GJ_active_account" "$GJ_PROJECT_RESOURCE" cloudbuild.builds.create
require_bucket_permission "$GJ_active_account" storage.objects.create
require_permission "$GJ_active_account" "$GJ_REPOSITORY_RESOURCE" artifactregistry.dockerimages.get
require_permission "$GJ_active_account" "$GJ_PROJECT_RESOURCE" run.services.create
require_permission "$GJ_active_account" "$GJ_PROJECT_RESOURCE" run.services.update
require_permission "$GJ_active_account" "$GJ_PROJECT_RESOURCE" run.services.setIamPolicy
require_permission "$GJ_active_account" "//iam.googleapis.com/projects/${GJ_PROJECT_ID}/serviceAccounts/${GJ_BUILD_SERVICE_ACCOUNT}" iam.serviceAccounts.actAs
require_permission "$GJ_active_account" "$GJ_RUNTIME_SA_RESOURCE" iam.serviceAccounts.actAs

[[ -z "$(git status --porcelain --untracked-files=normal)" ]] || fail 'worktree must be clean'
git diff-index --quiet HEAD -- || fail 'tracked files differ from HEAD'
GJ_branch="$(git symbolic-ref --quiet --short HEAD)" || fail 'deployment requires a named GJ_branch'
git fetch --quiet origin "$GJ_branch"
git merge-base --is-ancestor HEAD "origin/$GJ_branch" || fail "HEAD is not pushed to origin/$GJ_branch"
[[ "$(git rev-parse HEAD)" == "$(git rev-parse "origin/$GJ_branch")" ]] || fail "HEAD must exactly match origin/$GJ_branch"

readonly GJ_git_sha
readonly GJ_revision_suffix
readonly GJ_image_tag
readonly GJ_build_sa_resource="projects/${GJ_PROJECT_ID}/serviceAccounts/${GJ_BUILD_SERVICE_ACCOUNT}"
GJ_git_sha="$(git rev-parse HEAD)"
GJ_revision_suffix="${GJ_git_sha:0:12}"
GJ_image_tag="${GJ_REGION}-docker.pkg.dev/${GJ_PROJECT_ID}/${GJ_ARTIFACT_REPOSITORY}/${GJ_IMAGE}:${GJ_git_sha}"

gcloud builds submit . \
  --project="$GJ_PROJECT_ID" \
  --region="$GJ_REGION" \
  --service-account="$GJ_build_sa_resource" \
  --default-buckets-behavior=regional-user-owned-bucket \
  --gcs-source-staging-dir="gs://${GJ_SOURCE_BUCKET}/source" \
  --tag="$GJ_image_tag"

GJ_digest="$(gcloud artifacts docker images describe "$GJ_image_tag" \
  --project="$GJ_PROJECT_ID" --format='value(image_summary.digest)')"
[[ "$GJ_digest" =~ ^sha256:[0-9a-f]{64}$ ]] || fail "Artifact Registry returned an invalid digest: $GJ_digest"
readonly GJ_image_digest="${GJ_REGION}-docker.pkg.dev/${GJ_PROJECT_ID}/${GJ_ARTIFACT_REPOSITORY}/${GJ_IMAGE}@${GJ_digest}"

gcloud beta run deploy "$GJ_SERVICE" \
  --project="$GJ_PROJECT_ID" \
  --region="$GJ_REGION" \
  --image="$GJ_image_digest" \
  --revision-suffix="$GJ_revision_suffix" \
  --no-traffic \
  --allow-unauthenticated \
  --min-instances="$GJ_MIN_INSTANCES" \
  --max-instances="$GJ_MAX_INSTANCES" \
  --concurrency="$GJ_CONCURRENCY" \
  --cpu="$GJ_CPU" \
  --memory="$GJ_MEMORY" \
  --timeout="${GJ_TIMEOUT_SECONDS}s" \
  --port=8080 \
  --startup-probe="initialDelaySeconds=0,timeoutSeconds=2,periodSeconds=2,failureThreshold=15,httpGet.port=8080,httpGet.path=${GJ_STARTUP_PATH}" \
  --liveness-probe="initialDelaySeconds=10,timeoutSeconds=2,periodSeconds=10,failureThreshold=3,httpGet.port=8080,httpGet.path=${GJ_LIVENESS_PATH}" \
  --service-account="$GJ_RUNTIME_SERVICE_ACCOUNT" \
  --quiet

mkdir -p "$(dirname "$GJ_DEPLOY_OUTPUT")"
GJ_service_json="$(mktemp)"
GJ_revision_json="$(mktemp)"
GJ_iam_json="$(mktemp)"
GJ_tmp_output="$(mktemp "${GJ_DEPLOY_OUTPUT}.tmp.XXXXXX")"
trap 'rm -f "$GJ_service_json" "$GJ_revision_json" "$GJ_iam_json" "$GJ_tmp_output"' EXIT
chmod 600 "$GJ_service_json" "$GJ_revision_json" "$GJ_iam_json" "$GJ_tmp_output"
gcloud run services describe "$GJ_SERVICE" --project="$GJ_PROJECT_ID" --region="$GJ_REGION" --format=json >"$GJ_service_json"
GJ_revision="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["status"]["latestCreatedRevisionName"])' "$GJ_service_json")"
[[ "$GJ_revision" == "${GJ_SERVICE}-${GJ_revision_suffix}" ]] || fail "created revision $GJ_revision does not match commit suffix $GJ_revision_suffix"
gcloud run revisions describe "$GJ_revision" --project="$GJ_PROJECT_ID" --region="$GJ_REGION" --format=json >"$GJ_revision_json"
gcloud run services get-iam-policy "$GJ_SERVICE" --project="$GJ_PROJECT_ID" --region="$GJ_REGION" --format=json >"$GJ_iam_json"

export GJ_EFFECTIVE_SETTINGS_JSON="$GJ_tmp_output"
python3 - "$GJ_service_json" "$GJ_revision_json" "$GJ_iam_json" "$GJ_image_digest" "$GJ_RUNTIME_SERVICE_ACCOUNT" "$GJ_revision" "$GJ_MIN_INSTANCES" "$GJ_MAX_INSTANCES" "$GJ_CONCURRENCY" "$GJ_CPU" "$GJ_MEMORY" "$GJ_TIMEOUT_SECONDS" "$GJ_STARTUP_PATH" "$GJ_LIVENESS_PATH" <<'PY'
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
spec = GJ_revision["spec"]
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
with open(os.environ["GJ_EFFECTIVE_SETTINGS_JSON"], "w", encoding="utf-8") as output:
    json.dump(actual, output, indent=2, sort_keys=True)
    output.write("\n")
PY

mv "$GJ_tmp_output" "$GJ_DEPLOY_OUTPUT"
trap - EXIT
rm -f "$GJ_service_json" "$GJ_revision_json" "$GJ_iam_json"
printf 'Validated revision %s at 0%% traffic\n' "$GJ_revision"
printf 'Captured verified effective settings in %s\n' "$GJ_DEPLOY_OUTPUT"
