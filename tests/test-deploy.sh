#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
script="$root/deploy/deploy.sh"
env_example="$root/deploy/cloud-run.env.example"

fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
contains() { rg -F --quiet -- "$1" "$2" || fail "$2 missing: $1"; }

[[ -x "$script" ]] || fail "$script must exist and be executable"
[[ -f "$env_example" ]] || fail "$env_example must exist"
bash -n "$script"

reentry_output="$(bash -c '
  readonly ROOT=/macos/system/root
  readonly git_sha=hook-owned
  readonly GJ_git_sha=hook-owned-too
  gcloud() {
    if [[ "$1 $2" == "auth list" ]]; then
      printf "%s\n" wrong@example.com
      return 0
    fi
    return 99
  }
  export -f gcloud
  deploy_script="$1"
  source "$deploy_script"
  source "$deploy_script"
  main || true
  main || true
' _ "$script" 2>&1)"
[[ "$(rg -c 'active account must be anthony.lui@archegon.com' <<<"$reentry_output")" == 2 ]] || fail "reentry harness did not reach controlled preflight twice: $reentry_output"
[[ "$reentry_output" != *'readonly variable'* ]] || fail 'deploy script collided with a readonly caller variable'

mock_dir="$(mktemp -d)"
mock_log="$mock_dir/build-args"
mkdir -p "$mock_dir/bin"
cat >"$mock_dir/bin/gcloud" <<'MOCK'
#!/usr/bin/env bash
case "$*" in
  "auth list"*) printf '%s\n' anthony.lui@archegon.com ;;
  "config get-value project"*) printf '%s\n' ninth-airship-386815 ;;
  "beta billing projects describe"*) printf '%s\n' True ;;
  "services list"*) printf '%s\n' artifactregistry.googleapis.com cloudbuild.googleapis.com iam.googleapis.com logging.googleapis.com policytroubleshooter.googleapis.com run.googleapis.com serviceusage.googleapis.com storage.googleapis.com ;;
  *"artifacts repositories describe"*"value(name)"*) printf '%s\n' projects/ninth-airship-386815/locations/australia-southeast1/repositories/graphs-and-jev ;;
  *"artifacts repositories describe"*"value(format)"*) printf '%s\n' DOCKER ;;
  *"iam service-accounts describe graphs-and-jev-builder"*) printf '%s\n' graphs-and-jev-builder@ninth-airship-386815.iam.gserviceaccount.com ;;
  *"iam service-accounts describe graphs-and-jev-runner"*) printf '%s\n' graphs-and-jev-runner@ninth-airship-386815.iam.gserviceaccount.com ;;
  "storage buckets describe"*) printf '%s\n' '{"location":"AUSTRALIA-SOUTHEAST1","location_type":"region","uniform_bucket_level_access":true,"public_access_prevention":"enforced"}' ;;
  "policy-intelligence troubleshoot-policy iam"*) printf '%s\n' ALLOW_ACCESS_STATE_GRANTED ;;
  "builds submit"*) printf '%s\n' "$*" >"$MOCK_BUILD_LOG"; exit 73 ;;
  *) printf 'unexpected gcloud: %s\n' "$*" >&2; exit 72 ;;
esac
MOCK
cat >"$mock_dir/bin/git" <<'MOCK'
#!/usr/bin/env bash
case "$*" in
  "status --porcelain --untracked-files=normal") ;;
  "diff-index --quiet HEAD --") ;;
  "symbolic-ref --quiet --short HEAD") printf '%s\n' feature/cloud-run-rust ;;
  "fetch --quiet origin feature/cloud-run-rust") ;;
  "merge-base --is-ancestor HEAD origin/feature/cloud-run-rust") ;;
  "rev-parse HEAD"|"rev-parse origin/feature/cloud-run-rust") printf '%s\n' 0123456789abcdef0123456789abcdef01234567 ;;
  *) printf 'unexpected git: %s\n' "$*" >&2; exit 71 ;;
esac
MOCK
chmod +x "$mock_dir/bin/gcloud" "$mock_dir/bin/git"
set +e
mock_output="$(PATH="$mock_dir/bin:$PATH" MOCK_BUILD_LOG="$mock_log" "$script" 2>&1)"
mock_status=$?
set -e
[[ "$mock_status" == 73 ]] || fail "mock deployment exited $mock_status: $mock_output"
[[ "$mock_output" != *'command not found'* ]] || fail "bare variable command executed: $mock_output"
mock_build_args="$(cat "$mock_log")"
[[ "$mock_build_args" == *'--tag=australia-southeast1-docker.pkg.dev/ninth-airship-386815/graphs-and-jev/app:0123456789abcdef0123456789abcdef01234567'* ]] || fail "build tag missing full SHA: $mock_build_args"
[[ "$mock_build_args" == *'--service-account=projects/ninth-airship-386815/serviceAccounts/graphs-and-jev-builder@ninth-airship-386815.iam.gserviceaccount.com'* ]] || fail "build service account missing: $mock_build_args"
if rg -n '^  gjd_(git_sha|revision_suffix|image_tag)$' "$script"; then fail 'bare derived-variable command remains'; fi
rm -rf "$mock_dir"

contains 'set -euo pipefail' "$script"
contains 'main() (' "$script"
contains 'local gjd_REPO_ROOT' "$script"
# shellcheck disable=SC2016
contains 'if [[ "${BASH_SOURCE[0]}" == "$0" ]]' "$script"
contains 'main "$@"' "$script"
if rg -n '\b(ROOT|git_sha)=' "$script"; then fail 'unprefixed collision-prone assignment remains'; fi
if rg -n '^(readonly )?(gjd_[A-Za-z0-9_]+|REPO_ROOT|active_account|active_project|billing_enabled|enabled_apis|repository_name|repository_format|expected_repository_name|bucket_json|branch|revision_suffix|image_tag|build_sa_resource|digest|image_digest|service_json|revision_json|iam_json|tmp_output|revision)=' "$script"; then fail 'unprefixed internal assignment remains'; fi
# shellcheck disable=SC2016
contains 'gjd_PROJECT_ID="${PROJECT_ID:-ninth-airship-386815}"' "$script"
contains 'ninth-airship-386815' "$env_example"
contains 'australia-southeast1' "$env_example"
contains 'IMAGE=app' "$env_example"
contains 'SOURCE_BUCKET=ninth-airship-386815-graphs-and-jev-build-source' "$env_example"
contains 'gcloud storage buckets describe' "$script"
contains 'uniform_bucket_level_access' "$script"
contains 'public_access_prevention' "$script"
contains 'storage.objects.get' "$script"
contains 'allowPolicyExplanation.allowAccessState' "$script"
contains 'ALLOW_ACCESS_STATE_GRANTED' "$script"
# shellcheck disable=SC2016
contains '--gcs-source-staging-dir="gs://${gjd_SOURCE_BUCKET}/source"' "$script"
# shellcheck disable=SC2016
contains '[[ "$gjd_revision" == "${gjd_SERVICE}-${gjd_revision_suffix}" ]]' "$script"
contains '--default-buckets-behavior=regional-user-owned-bucket' "$script"
contains 'gcloud beta billing projects describe' "$script"
contains 'gcloud services list' "$script"
contains 'gcloud artifacts repositories describe' "$script"
contains "--format='value(name)'" "$script"
contains "--format='value(format)'" "$script"
if rg -F --quiet "value(name,format)" "$script"; then fail 'combined repository output remains'; fi
contains 'gcloud iam service-accounts describe' "$script"
contains 'troubleshoot-policy iam' "$script"
contains 'iam.serviceAccounts.actAs' "$script"
contains 'logging.logEntries.create' "$script"
contains 'serviceusage.services.use' "$script"
contains 'artifactregistry.repositories.uploadArtifacts' "$script"
if rg -F --quiet 'value(overallAccessState)' "$script"; then fail 'obsolete troubleshooter field remains'; fi
if rg -F --quiet 'cloudbilling.googleapis.com' "$script"; then fail 'billing API must not be required'; fi
contains 'logging.googleapis.com' "$script"
contains 'gcloud run revisions describe' "$script"
contains 'gcloud run services get-iam-policy' "$script"
contains 'sys.argv[15]' "$script"
contains 'json.load' "$script"
# shellcheck disable=SC2016
contains 'graphs-and-jev-builder@${PROJECT_ID}.iam.gserviceaccount.com' "$env_example"
# shellcheck disable=SC2016
contains 'graphs-and-jev-runner@${PROJECT_ID}.iam.gserviceaccount.com' "$env_example"
contains 'anthony.lui@archegon.com' "$script"
contains 'git diff-index --quiet HEAD --' "$script"
contains 'git status --porcelain' "$script"
contains 'git merge-base --is-ancestor HEAD' "$script"
contains 'gcloud builds submit' "$script"
# shellcheck disable=SC2016
contains '--service-account="$gjd_build_sa_resource"' "$script"
contains 'gcloud artifacts docker images describe' "$script"
# shellcheck disable=SC2016
contains '${gjd_IMAGE}@${gjd_digest}' "$script"
contains 'assert_effective_deployment' "$script"
contains 'gcloud beta run deploy' "$script"
contains '--no-traffic' "$script"
contains '--allow-unauthenticated' "$script"
# shellcheck disable=SC2016
contains 'gjd_MIN_INSTANCES="${MIN_INSTANCES:-0}"' "$script"
# shellcheck disable=SC2016
contains 'gjd_MAX_INSTANCES="${MAX_INSTANCES:-2}"' "$script"
# shellcheck disable=SC2016
contains 'gjd_CONCURRENCY="${CONCURRENCY:-20}"' "$script"
# shellcheck disable=SC2016
contains 'gjd_CPU="${CPU:-1}"' "$script"
# shellcheck disable=SC2016
contains 'gjd_MEMORY="${MEMORY:-512Mi}"' "$script"
# shellcheck disable=SC2016
contains 'gjd_TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-30}"' "$script"
# shellcheck disable=SC2016
contains 'gjd_STARTUP_PATH="${STARTUP_PATH:-/api/ready}"' "$script"
# shellcheck disable=SC2016
contains 'gjd_LIVENESS_PATH="${LIVENESS_PATH:-/api/health}"' "$script"
# shellcheck disable=SC2016
contains '--service-account="$gjd_RUNTIME_SERVICE_ACCOUNT"' "$script"

if rg -n '(:latest|--tag=latest)' "$script"; then
  fail 'deploy script must not use latest'
fi

permission_fixture='{"accessTuple":{"permission":"run.services.update"},"allowPolicyExplanation":{"allowAccessState":"ALLOW_ACCESS_STATE_GRANTED","explainedPolicies":[]},"overallAccessState":"UNKNOWN_INFO"}'
parsed_state="$(python3 -c 'import json,sys; print(json.load(sys.stdin)["allowPolicyExplanation"]["allowAccessState"])' <<<"$permission_fixture")"
[[ "$parsed_state" == "ALLOW_ACCESS_STATE_GRANTED" ]] || fail 'realistic granted IAM fixture did not parse'
repository_name_fixture='projects/ninth-airship-386815/locations/australia-southeast1/repositories/graphs-and-jev'
repository_format_fixture=$'DOCKER\t'
repository_name="$(printf '%s\n' "$repository_name_fixture")"
repository_format="$(printf '%s\n' "$repository_format_fixture")"
[[ "$repository_name" == 'projects/ninth-airship-386815/locations/australia-southeast1/repositories/graphs-and-jev' ]] || fail 'realistic repository name did not parse'
[[ "$repository_format" == $'DOCKER\t' ]] || fail 'realistic tab-bearing repository format did not parse'


validator="$(python3 - "$script" <<'PYEXTRACT'
import sys
text = open(sys.argv[1], encoding="utf-8").read()
marker = "python3 - \"$gjd_service_json\""
start = text.index(marker)
start = text.index("<<'PY'", start) + len("<<'PY'") + 1
end = text.index("\nPY\n", start)
print(text[start:end])
PYEXTRACT
)"
validator_dir="$(mktemp -d)"
trap 'rm -rf "$validator_dir"' EXIT
cat >"$validator_dir/service.json" <<'JSON'
{"metadata":{"name":"graphs-and-jev"},"status":{"url":"https://graphs-and-jev.example.run.app","traffic":[]}}
JSON
cat >"$validator_dir/revision.json" <<'JSON'
{"metadata":{"name":"graphs-and-jev-abc123","annotations":{"autoscaling.knative.dev/minScale":"0","autoscaling.knative.dev/maxScale":"2"}},"spec":{"serviceAccountName":"graphs-and-jev-runner@ninth-airship-386815.iam.gserviceaccount.com","containerConcurrency":20,"timeoutSeconds":30,"containers":[{"image":"australia-southeast1-docker.pkg.dev/ninth-airship-386815/graphs-and-jev/app@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","resources":{"limits":{"cpu":"1","memory":"512Mi"}},"startupProbe":{"httpGet":{"path":"/api/ready"}},"livenessProbe":{"httpGet":{"path":"/api/health"}}}]}}
JSON
cat >"$validator_dir/iam.json" <<'JSON'
{"bindings":[{"role":"roles/run.invoker","members":["allUsers"]}]}
JSON
python3 -c "$validator" \
  "$validator_dir/service.json" "$validator_dir/revision.json" "$validator_dir/iam.json" \
  'australia-southeast1-docker.pkg.dev/ninth-airship-386815/graphs-and-jev/app@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa' \
  'graphs-and-jev-runner@ninth-airship-386815.iam.gserviceaccount.com' 'graphs-and-jev-abc123' \
  0 2 20 1 512Mi 30 /api/ready /api/health "$validator_dir/effective.json"
python3 -c 'import json,sys; data=json.load(open(sys.argv[1])); assert data["revision"] == "graphs-and-jev-abc123"; assert data["image_digest"].endswith("a" * 64)' "$validator_dir/effective.json"
if rg -n '\b(gjd_|GJ_)[A-Za-z_]*' <<<"$validator"; then fail 'shell namespace leaked into embedded Python'; fi
trap - EXIT
rm -rf "$validator_dir"

printf 'deploy static checks passed\n'
