#!/usr/bin/env bash

main() (
  set -euo pipefail
  local gjd_REPO_ROOT gjd_PROJECT_ID gjd_REGION gjd_SERVICE gjd_ARTIFACT_REPOSITORY gjd_IMAGE
  local gjd_SOURCE_BUCKET gjd_BUILD_SERVICE_ACCOUNT gjd_RUNTIME_SERVICE_ACCOUNT gjd_EXPECTED_ACCOUNT
  local gjd_DEPLOY_OUTPUT gjd_PROJECT_RESOURCE gjd_REPOSITORY_RESOURCE gjd_RUNTIME_SA_RESOURCE
  local gjd_SOURCE_BUCKET_RESOURCE gjd_MIN_INSTANCES gjd_MAX_INSTANCES gjd_CONCURRENCY gjd_CPU
  local gjd_MEMORY gjd_TIMEOUT_SECONDS gjd_STARTUP_PATH gjd_LIVENESS_PATH gjd_command_name
  local gjd_active_account gjd_active_project gjd_billing_enabled gjd_enabled_apis gjd_api
  local gjd_repository_name gjd_repository_format gjd_expected_repository_name gjd_bucket_json
  local gjd_branch gjd_git_sha gjd_revision_suffix gjd_image_tag gjd_build_sa_resource gjd_digest
  local gjd_image_digest gjd_service_json gjd_revision_json gjd_iam_json gjd_tmp_output gjd_revision
  local -a gjd_required_apis

  gjd_REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
  cd "$gjd_REPO_ROOT"

  if [[ $# -gt 1 ]]; then
    printf 'usage: %s [environment-file]\n' "$0" >&2
    exit 64
  fi
  if [[ $# -eq 1 ]]; then
    # shellcheck disable=SC1090
    source "$1"
  fi

  gjd_PROJECT_ID="${PROJECT_ID:-ninth-airship-386815}"
  gjd_REGION="${REGION:-australia-southeast1}"
  gjd_SERVICE="${SERVICE:-graphs-and-jev}"
  gjd_ARTIFACT_REPOSITORY="${ARTIFACT_REPOSITORY:-graphs-and-jev}"
  gjd_IMAGE="${IMAGE:-app}"
  gjd_SOURCE_BUCKET="${SOURCE_BUCKET:-ninth-airship-386815-graphs-and-jev-build-source}"
  gjd_BUILD_SERVICE_ACCOUNT="${BUILD_SERVICE_ACCOUNT:-graphs-and-jev-builder@${gjd_PROJECT_ID}.iam.gserviceaccount.com}"
  gjd_RUNTIME_SERVICE_ACCOUNT="${RUNTIME_SERVICE_ACCOUNT:-graphs-and-jev-runner@${gjd_PROJECT_ID}.iam.gserviceaccount.com}"
  gjd_EXPECTED_ACCOUNT="${EXPECTED_ACCOUNT:-anthony.lui@archegon.com}"
  gjd_DEPLOY_OUTPUT="${DEPLOY_OUTPUT:-${TMPDIR:-/tmp}/graphs-and-jev-deployment-result.json}"
  gjd_PROJECT_RESOURCE="//cloudresourcemanager.googleapis.com/projects/${gjd_PROJECT_ID}"
  gjd_REPOSITORY_RESOURCE="//artifactregistry.googleapis.com/projects/${gjd_PROJECT_ID}/locations/${gjd_REGION}/repositories/${gjd_ARTIFACT_REPOSITORY}"
  gjd_RUNTIME_SA_RESOURCE="//iam.googleapis.com/projects/${gjd_PROJECT_ID}/serviceAccounts/${gjd_RUNTIME_SERVICE_ACCOUNT}"
  gjd_SOURCE_BUCKET_RESOURCE="//storage.googleapis.com/projects/_/buckets/${gjd_SOURCE_BUCKET}"
  gjd_MIN_INSTANCES="${MIN_INSTANCES:-0}"
  gjd_MAX_INSTANCES="${MAX_INSTANCES:-2}"
  gjd_CONCURRENCY="${CONCURRENCY:-20}"
  gjd_CPU="${CPU:-1}"
  gjd_MEMORY="${MEMORY:-512Mi}"
  gjd_TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-30}"
  gjd_STARTUP_PATH="${STARTUP_PATH:-/api/ready}"
  gjd_LIVENESS_PATH="${LIVENESS_PATH:-/api/health}"

  for gjd_command_name in gcloud git python3; do
    command -v "$gjd_command_name" >/dev/null || { printf '%s is required\n' "$gjd_command_name" >&2; exit 1; }
  done

  fail() { printf 'preflight failed: %s\n' "$*" >&2; exit 1; }
  require_permission() {
    local principal="$1" resource="$2" permission="$3" state
    state="$(gcloud policy-intelligence troubleshoot-policy iam "$resource" \
      --project="$gjd_PROJECT_ID" \
      --principal-email="$principal" \
      --permission="$permission" \
      --format='value(allowPolicyExplanation.allowAccessState)')"
    [[ "$state" == "ALLOW_ACCESS_STATE_GRANTED" ]] || fail "$principal lacks $permission on $resource (state: ${state:-unknown})"
  }
  require_bucket_permission() {
    local principal="$1" permission="$2" state
    state="$(gcloud policy-intelligence troubleshoot-policy iam "$gjd_SOURCE_BUCKET_RESOURCE" \
      --project="$gjd_PROJECT_ID" \
      --principal-email="$principal" \
      --permission="$permission" \
      --resource-name="${gjd_SOURCE_BUCKET_RESOURCE}/objects/source-preflight" \
      --resource-service=storage.googleapis.com \
      --resource-type=storage.googleapis.com/Object \
      --format='value(allowPolicyExplanation.allowAccessState)')"
    [[ "$state" == "ALLOW_ACCESS_STATE_GRANTED" ]] || fail "$principal lacks $permission on gs://$gjd_SOURCE_BUCKET (state: ${state:-unknown})"
  }

  gjd_active_account="$(gcloud auth list --filter=status:ACTIVE --format='value(account)')"
  [[ "$gjd_active_account" == "$gjd_EXPECTED_ACCOUNT" ]] || fail "active account must be $gjd_EXPECTED_ACCOUNT (found ${gjd_active_account:-none})"
  gjd_active_project="$(gcloud config get-value project 2>/dev/null)"
  [[ "$gjd_active_project" == "$gjd_PROJECT_ID" ]] || fail "active project must be $gjd_PROJECT_ID (found ${gjd_active_project:-none})"

  gjd_billing_enabled="$(gcloud beta billing projects describe "$gjd_PROJECT_ID" --format='value(billingEnabled)')"
  [[ "$gjd_billing_enabled" == "True" ]] || fail "billing is not enabled for $gjd_PROJECT_ID"

  gjd_required_apis=(
    artifactregistry.googleapis.com
    cloudbuild.googleapis.com
    iam.googleapis.com
    logging.googleapis.com
    policytroubleshooter.googleapis.com
    run.googleapis.com
    serviceusage.googleapis.com
    storage.googleapis.com
  )
  gjd_enabled_apis="$(gcloud services list --enabled --project="$gjd_PROJECT_ID" --format='value(config.name)')"
  for gjd_api in "${gjd_required_apis[@]}"; do
    grep -Fxq "$gjd_api" <<<"$gjd_enabled_apis" || fail "required API is not enabled: $gjd_api"
  done

  gjd_repository_name="$(gcloud artifacts repositories describe "$gjd_ARTIFACT_REPOSITORY" \
    --project="$gjd_PROJECT_ID" --location="$gjd_REGION" --format='value(name)')"
  gjd_repository_format="$(gcloud artifacts repositories describe "$gjd_ARTIFACT_REPOSITORY" \
    --project="$gjd_PROJECT_ID" --location="$gjd_REGION" --format='value(format)')"
  gjd_expected_repository_name="projects/${gjd_PROJECT_ID}/locations/${gjd_REGION}/repositories/${gjd_ARTIFACT_REPOSITORY}"
  [[ "$gjd_repository_name" == "$gjd_expected_repository_name" ]] || fail "Artifact Registry repository is not $gjd_expected_repository_name (found ${gjd_repository_name:-none})"
  [[ "$gjd_repository_format" == "DOCKER" ]] || fail "Artifact Registry repository must use DOCKER format (found ${gjd_repository_format:-none})"
  gcloud iam service-accounts describe "$gjd_BUILD_SERVICE_ACCOUNT" --project="$gjd_PROJECT_ID" --format='value(email)' | grep -Fxq "$gjd_BUILD_SERVICE_ACCOUNT" || fail "builder service account does not exist"
  gcloud iam service-accounts describe "$gjd_RUNTIME_SERVICE_ACCOUNT" --project="$gjd_PROJECT_ID" --format='value(email)' | grep -Fxq "$gjd_RUNTIME_SERVICE_ACCOUNT" || fail "runtime service account does not exist"

  gjd_bucket_json="$(gcloud storage buckets describe "gs://${gjd_SOURCE_BUCKET}" --project="$gjd_PROJECT_ID" --format=json)"
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
    raise SystemExit("source bucket preflight failed: " + ", ".join(failed))' "$gjd_REGION" <<<"$gjd_bucket_json"

  # The builder only writes build logs, consumes enabled services, pushes the image, and reads staged source.
  require_permission "$gjd_BUILD_SERVICE_ACCOUNT" "$gjd_PROJECT_RESOURCE" logging.logEntries.create
  require_permission "$gjd_BUILD_SERVICE_ACCOUNT" "$gjd_PROJECT_RESOURCE" serviceusage.services.use
  require_permission "$gjd_BUILD_SERVICE_ACCOUNT" "$gjd_REPOSITORY_RESOURCE" artifactregistry.repositories.uploadArtifacts
  require_bucket_permission "$gjd_BUILD_SERVICE_ACCOUNT" storage.objects.get

  # The authenticated human submits and deploys, reads the resulting image/state, publishes IAM, and attaches the runtime identity.
  require_permission "$gjd_active_account" "$gjd_PROJECT_RESOURCE" cloudbuild.builds.create
  require_bucket_permission "$gjd_active_account" storage.objects.create
  require_permission "$gjd_active_account" "$gjd_REPOSITORY_RESOURCE" artifactregistry.dockerimages.get
  require_permission "$gjd_active_account" "$gjd_PROJECT_RESOURCE" run.services.create
  require_permission "$gjd_active_account" "$gjd_PROJECT_RESOURCE" run.services.update
  require_permission "$gjd_active_account" "$gjd_PROJECT_RESOURCE" run.services.setIamPolicy
  require_permission "$gjd_active_account" "//iam.googleapis.com/projects/${gjd_PROJECT_ID}/serviceAccounts/${gjd_BUILD_SERVICE_ACCOUNT}" iam.serviceAccounts.actAs
  require_permission "$gjd_active_account" "$gjd_RUNTIME_SA_RESOURCE" iam.serviceAccounts.actAs

  [[ -z "$(git status --porcelain --untracked-files=normal)" ]] || fail 'worktree must be clean'
  git diff-index --quiet HEAD -- || fail 'tracked files differ from HEAD'
  gjd_branch="$(git symbolic-ref --quiet --short HEAD)" || fail 'deployment requires a named gjd_branch'
  git fetch --quiet origin "$gjd_branch"
  git merge-base --is-ancestor HEAD "origin/$gjd_branch" || fail "HEAD is not pushed to origin/$gjd_branch"
  [[ "$(git rev-parse HEAD)" == "$(git rev-parse "origin/$gjd_branch")" ]] || fail "HEAD must exactly match origin/$gjd_branch"

  gjd_git_sha="$(git rev-parse HEAD)"
  gjd_revision_suffix="${gjd_git_sha:0:12}"
  gjd_image_tag="${gjd_REGION}-docker.pkg.dev/${gjd_PROJECT_ID}/${gjd_ARTIFACT_REPOSITORY}/${gjd_IMAGE}:${gjd_git_sha}"
  gjd_build_sa_resource="projects/${gjd_PROJECT_ID}/serviceAccounts/${gjd_BUILD_SERVICE_ACCOUNT}"

  gcloud builds submit . \
    --project="$gjd_PROJECT_ID" \
    --region="$gjd_REGION" \
    --service-account="$gjd_build_sa_resource" \
    --default-buckets-behavior=regional-user-owned-bucket \
    --gcs-source-staging-dir="gs://${gjd_SOURCE_BUCKET}/source" \
    --tag="$gjd_image_tag"

  gjd_digest="$(gcloud artifacts docker images describe "$gjd_image_tag" \
    --project="$gjd_PROJECT_ID" --format='value(image_summary.digest)')"
  [[ "$gjd_digest" =~ ^sha256:[0-9a-f]{64}$ ]] || fail "Artifact Registry returned an invalid digest: $gjd_digest"
  gjd_image_digest="${gjd_REGION}-docker.pkg.dev/${gjd_PROJECT_ID}/${gjd_ARTIFACT_REPOSITORY}/${gjd_IMAGE}@${gjd_digest}"

  gcloud beta run deploy "$gjd_SERVICE" \
    --project="$gjd_PROJECT_ID" \
    --region="$gjd_REGION" \
    --image="$gjd_image_digest" \
    --revision-suffix="$gjd_revision_suffix" \
    --no-traffic \
    --allow-unauthenticated \
    --min-instances="$gjd_MIN_INSTANCES" \
    --max-instances="$gjd_MAX_INSTANCES" \
    --concurrency="$gjd_CONCURRENCY" \
    --cpu="$gjd_CPU" \
    --memory="$gjd_MEMORY" \
    --timeout="${gjd_TIMEOUT_SECONDS}s" \
    --port=8080 \
    --startup-probe="initialDelaySeconds=0,timeoutSeconds=2,periodSeconds=2,failureThreshold=15,httpGet.port=8080,httpGet.path=${gjd_STARTUP_PATH}" \
    --liveness-probe="initialDelaySeconds=10,timeoutSeconds=2,periodSeconds=10,failureThreshold=3,httpGet.port=8080,httpGet.path=${gjd_LIVENESS_PATH}" \
    --service-account="$gjd_RUNTIME_SERVICE_ACCOUNT" \
    --quiet

  mkdir -p "$(dirname "$gjd_DEPLOY_OUTPUT")"
  gjd_service_json="$(mktemp)"
  gjd_revision_json="$(mktemp)"
  gjd_iam_json="$(mktemp)"
  gjd_tmp_output="$(mktemp "${gjd_DEPLOY_OUTPUT}.tmp.XXXXXX")"
  trap 'rm -f "$gjd_service_json" "$gjd_revision_json" "$gjd_iam_json" "$gjd_tmp_output"' EXIT
  chmod 600 "$gjd_service_json" "$gjd_revision_json" "$gjd_iam_json" "$gjd_tmp_output"
  gcloud run services describe "$gjd_SERVICE" --project="$gjd_PROJECT_ID" --region="$gjd_REGION" --format=json >"$gjd_service_json"
  gjd_revision="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["status"]["latestCreatedRevisionName"])' "$gjd_service_json")"
  [[ "$gjd_revision" == "${gjd_SERVICE}-${gjd_revision_suffix}" ]] || fail "created revision $gjd_revision does not match commit suffix $gjd_revision_suffix"
  gcloud run revisions describe "$gjd_revision" --project="$gjd_PROJECT_ID" --region="$gjd_REGION" --format=json >"$gjd_revision_json"
  gcloud run services get-iam-policy "$gjd_SERVICE" --project="$gjd_PROJECT_ID" --region="$gjd_REGION" --format=json >"$gjd_iam_json"

  python3 - "$gjd_service_json" "$gjd_revision_json" "$gjd_iam_json" "$gjd_image_digest" "$gjd_RUNTIME_SERVICE_ACCOUNT" "$gjd_revision" "$gjd_MIN_INSTANCES" "$gjd_MAX_INSTANCES" "$gjd_CONCURRENCY" "$gjd_CPU" "$gjd_MEMORY" "$gjd_TIMEOUT_SECONDS" "$gjd_STARTUP_PATH" "$gjd_LIVENESS_PATH" "$gjd_tmp_output" <<'PY'
import json
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
with open(sys.argv[15], "w", encoding="utf-8") as output:
    json.dump(actual, output, indent=2, sort_keys=True)
    output.write("\n")
PY

  mv "$gjd_tmp_output" "$gjd_DEPLOY_OUTPUT"
  trap - EXIT
  rm -f "$gjd_service_json" "$gjd_revision_json" "$gjd_iam_json"
  printf 'Validated revision %s at 0%% traffic\n' "$gjd_revision"
  printf 'Captured verified effective settings in %s\n' "$gjd_DEPLOY_OUTPUT"
)

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  main "$@"
fi
