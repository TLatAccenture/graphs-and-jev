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

[[ -f "$root/.gcloudignore" ]] || fail '.gcloudignore must exist'
upload_files="$(cd "$root" && gcloud meta list-files-for-upload . | LC_ALL=C sort)"
for required in Dockerfile Cargo.toml Cargo.lock catalogue.json; do
  grep -Fxq "$required" <<<"$upload_files" || fail "upload context missing $required"
done
if rg -n '(^|/)(target|\.git|docs|deploy|tests|scripts|\.env|credentials?)(/|$)' <<<"$upload_files"; then
  fail 'upload context contains forbidden files'
fi
unexpected="$(grep -Ev '^(Dockerfile|Cargo\.toml|Cargo\.lock|catalogue\.json|src/.+|static/.+)$' <<<"$upload_files" || true)"
[[ -z "$unexpected" ]] || fail "unexpected upload files: $unexpected"
upload_count="$(wc -l <<<"$upload_files" | tr -d ' ')"
upload_bytes="$(while IFS= read -r file; do wc -c <"$root/$file"; done <<<"$upload_files" | awk '{sum += $1} END {print sum + 0}')"
(( upload_count > 4 && upload_count < 500 )) || fail "unsafe upload file count: $upload_count"
(( upload_bytes < 5242880 )) || fail "unsafe upload bytes: $upload_bytes"

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
  "projects describe"*) printf '%s\n' 956922431929 ;;
  "beta billing projects describe"*) printf '%s\n' True ;;
  "services list"*) printf '%s\n' artifactregistry.googleapis.com cloudbuild.googleapis.com iam.googleapis.com logging.googleapis.com policytroubleshooter.googleapis.com run.googleapis.com serviceusage.googleapis.com storage.googleapis.com ${MOCK_EXTRA_APIS:-} ;;
  "secrets describe"*) printf '%s\n' projects/956922431929/secrets/typesafe-api-key ;;
  "secrets versions describe"*) printf '%s\n' ENABLED ;;
  "policy-intelligence troubleshoot-policy iam"*"--permission=secretmanager.versions.access"*) printf '%s\n' "${MOCK_SECRET_ACCESS_STATE:-ALLOW_ACCESS_STATE_GRANTED}" ;;
  "policy-intelligence troubleshoot-policy iam"*"--permission=aiplatform.endpoints.predict"*) printf '%s\n' "${MOCK_PREDICT_STATE:-ALLOW_ACCESS_STATE_GRANTED}" ;;
  *"artifacts repositories describe"*"value(name)"*) printf '%s\n' projects/ninth-airship-386815/locations/australia-southeast1/repositories/graphs-and-jev ;;
  *"artifacts repositories describe"*"value(format)"*) printf '%s\n' DOCKER ;;
  *"iam service-accounts describe graphs-and-jev-builder"*) printf '%s\n' graphs-and-jev-builder@ninth-airship-386815.iam.gserviceaccount.com ;;
  *"iam service-accounts describe graphs-and-jev-runner"*) printf '%s\n' graphs-and-jev-runner@ninth-airship-386815.iam.gserviceaccount.com ;;
  "storage buckets describe"*) printf '%s\n' '{"location":"AUSTRALIA-SOUTHEAST1","location_type":"region","uniform_bucket_level_access":true,"public_access_prevention":"enforced"}' ;;
  "policy-intelligence troubleshoot-policy iam"*) printf '%s\n' ALLOW_ACCESS_STATE_GRANTED ;;
  "meta list-files-for-upload .") cat <<'FILES'
Dockerfile
Cargo.toml
Cargo.lock
catalogue.json
src/main.rs
static/index.html
FILES
    ;;
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
[[ "$mock_output" != *'storage.buckets.list'* ]] || fail "bucket-list permission was checked: $mock_output"
for field in location location_type uniform_bucket_level_access public_access_prevention; do
  contains "\"${field}\"" "$script"
done
mock_build_args="$(cat "$mock_log")"
[[ "$mock_build_args" == *'--tag=australia-southeast1-docker.pkg.dev/ninth-airship-386815/graphs-and-jev/app:0123456789abcdef0123456789abcdef01234567'* ]] || fail "build tag missing full SHA: $mock_build_args"
[[ "$mock_build_args" == *'--gcs-log-dir=gs://956922431929-australia-southeast1-cloudbuild-logs/logs'* ]] || fail "explicit logs bucket missing: $mock_build_args"
[[ "$mock_build_args" == *'--service-account=projects/ninth-airship-386815/serviceAccounts/graphs-and-jev-builder@ninth-airship-386815.iam.gserviceaccount.com'* ]] || fail "build service account missing: $mock_build_args"
cat >"$mock_dir/app.env" <<'ENV'
BRAND_NAME='Shed "test" \ </script>'
API_MODE=live
API_MODE_LINEAGE=fixtures
API_BASE_URL=https://api.example.run.app
ENV
rm -f "$mock_log"
set +e
mock_output="$(PATH="$mock_dir/bin:$PATH" MOCK_BUILD_LOG="$mock_log" "$script" "$mock_dir/app.env" 2>&1)"
mock_status=$?
set -e
[[ "$mock_status" == 73 ]] || fail "valid app settings did not reach the build: $mock_status $mock_output"
for bad_setting in 'API_MODE=remote' 'API_MODE_LINEAGE=Live' 'API_TIMEOUT_MS=soon' 'API_BASE_URL="https://api.example.com; script-src *"' "API_BASE_URL='javascript:alert(1)'"; do
  printf '%s\n' "$bad_setting" >"$mock_dir/bad.env"
  rm -f "$mock_log"
  set +e
  mock_output="$(PATH="$mock_dir/bin:$PATH" MOCK_BUILD_LOG="$mock_log" "$script" "$mock_dir/bad.env" 2>&1)"
  mock_status=$?
  set -e
  [[ "$mock_status" == 1 && "$mock_output" == *'invalid app environment variables'* ]] || fail "bad app setting passed preflight: $bad_setting ($mock_status) $mock_output"
  [[ ! -e "$mock_log" ]] || fail "bad app setting reached the build: $bad_setting"
done
# A Gemini presenter build needs the Vertex AI API and predict permission for the runtime identity,
# and never takes a token from the env file.
printf '%s\n' CATALOGUE_INTERPRETER=gemini SERVICE=graphs-and-jev-presenter >"$mock_dir/gemini.env"
gemini_deploy() {
  rm -f "$mock_log"
  set +e
  mock_output="$(PATH="$mock_dir/bin:$PATH" MOCK_BUILD_LOG="$mock_log" "$@" "$script" "$mock_dir/gemini.env" 2>&1)"
  mock_status=$?
  set -e
}
gemini_deploy env
[[ "$mock_status" == 1 && "$mock_output" == *'required API is not enabled: aiplatform.googleapis.com'* ]] || fail "Gemini build passed without the Vertex AI API: $mock_status $mock_output"
gemini_deploy env MOCK_EXTRA_APIS=aiplatform.googleapis.com MOCK_PREDICT_STATE=ALLOW_ACCESS_STATE_NOT_GRANTED
[[ "$mock_status" == 1 && "$mock_output" == *'lacks aiplatform.endpoints.predict'* ]] || fail "Gemini build passed without predict permission: $mock_status $mock_output"
gemini_deploy env MOCK_EXTRA_APIS=aiplatform.googleapis.com
[[ "$mock_status" == 73 ]] || fail "valid Gemini build did not reach the build: $mock_status $mock_output"
gemini_deploy env MOCK_EXTRA_APIS=aiplatform.googleapis.com GEMINI_ACCESS_TOKEN=ya29.local
[[ "$mock_status" == 1 && "$mock_output" == *'GEMINI_ACCESS_TOKEN is for local runs only'* && ! -e "$mock_log" ]] || fail "deploy accepted a Gemini access token: $mock_status $mock_output"
[[ "$mock_output" != *ya29.local* ]] || fail 'access token echoed by deploy'
printf '%s\n' TYPESAFE_API_KEY=plaintext >"$mock_dir/plaintext.env"
set +e; mock_output="$(PATH="$mock_dir/bin:$PATH" MOCK_BUILD_LOG="$mock_log" "$script" "$mock_dir/plaintext.env" 2>&1)"; mock_status=$?; set -e
[[ "$mock_status" == 1 && "$mock_output" == *'TYPESAFE_API_KEY plaintext is forbidden'* ]] || fail "plaintext TypeSafe key was not safely rejected: $mock_output"
printf '%s\n' DECISION_BACKEND=jev SERVICE_AUTH_MODE=cloud-run-iam JEV_MODEL=jev-1.2.3 TYPESAFE_SECRET_RESOURCE=bad TYPESAFE_EGRESS_ACKNOWLEDGED=yes >"$mock_dir/jev.env"
set +e; mock_output="$(PATH="$mock_dir/bin:$PATH" MOCK_BUILD_LOG="$mock_log" "$script" "$mock_dir/jev.env" 2>&1)"; mock_status=$?; set -e
[[ "$mock_status" == 1 && "$mock_output" == *'must be projects/PROJECT/secrets/SECRET/versions/VERSION'* ]] || fail "malformed secret resource accepted: $mock_output"
sed -i.bak 's#TYPESAFE_SECRET_RESOURCE=bad#TYPESAFE_SECRET_RESOURCE=projects/other-project/secrets/typesafe-api-key/versions/1#' "$mock_dir/jev.env"
set +e; mock_output="$(PATH="$mock_dir/bin:$PATH" MOCK_BUILD_LOG="$mock_log" "$script" "$mock_dir/jev.env" 2>&1)"; mock_status=$?; set -e
[[ "$mock_status" == 1 && "$mock_output" == *'must be in the deployment project'* ]] || fail "cross-project secret accepted: $mock_output"
sed -i.bak 's#projects/other-project/secrets/typesafe-api-key/versions/1#projects/123456789012/secrets/typesafe-api-key/versions/1#' "$mock_dir/jev.env"
set +e; mock_output="$(PATH="$mock_dir/bin:$PATH" MOCK_BUILD_LOG="$mock_log" "$script" "$mock_dir/jev.env" 2>&1)"; mock_status=$?; set -e
[[ "$mock_status" == 1 && "$mock_output" == *'must be in the deployment project'* ]] || fail "cross-project number accepted: $mock_output"
sed -i.bak 's#projects/123456789012/secrets/typesafe-api-key/versions/1#projects/956922431929/secrets/typesafe-api-key/versions/1#' "$mock_dir/jev.env"
set +e; mock_output="$(PATH="$mock_dir/bin:$PATH" MOCK_BUILD_LOG="$mock_log" MOCK_EXTRA_APIS=secretmanager.googleapis.com "$script" "$mock_dir/jev.env" 2>&1)"; mock_status=$?; set -e
[[ "$mock_status" == 73 ]] || fail "canonical project-number secret did not reach build: $mock_status $mock_output"
sed -i.bak 's#projects/956922431929/secrets/typesafe-api-key/versions/1#projects/ninth-airship-386815/secrets/typesafe-api-key/versions/latest#' "$mock_dir/jev.env"
set +e; mock_output="$(PATH="$mock_dir/bin:$PATH" MOCK_BUILD_LOG="$mock_log" "$script" "$mock_dir/jev.env" 2>&1)"; mock_status=$?; set -e
[[ "$mock_status" == 1 && "$mock_output" == *'latest is forbidden'* ]] || fail "latest secret version accepted: $mock_output"
sed -i.bak 's#versions/latest#versions/1#' "$mock_dir/jev.env"
sed -i.bak 's#projects/other-project#projects/ninth-airship-386815#' "$mock_dir/jev.env"
set +e; mock_output="$(PATH="$mock_dir/bin:$PATH" MOCK_BUILD_LOG="$mock_log" MOCK_EXTRA_APIS=secretmanager.googleapis.com MOCK_SECRET_ACCESS_STATE=ALLOW_ACCESS_STATE_NOT_GRANTED "$script" "$mock_dir/jev.env" 2>&1)"; mock_status=$?; set -e
[[ "$mock_status" == 1 && "$mock_output" == *'lacks secretmanager.versions.access'* ]] || fail "Jev deploy accepted runtime identity without secret access: $mock_output"
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
contains 'LOG_BUCKET=956922431929-australia-southeast1-cloudbuild-logs' "$env_example"
contains 'storage.buckets.get' "$script"
if rg -F --quiet 'storage.buckets.list' "$script"; then fail 'inapplicable bucket-list permission remains'; fi
contains 'storage.buckets.get' "$script"
for permission in storage.objects.create storage.objects.get storage.objects.update storage.objects.delete; do
  contains "$permission" "$script"
done
# shellcheck disable=SC2016
contains '--gcs-log-dir="gs://${gjd_LOG_BUCKET}/logs"' "$script"
if rg -F --quiet -- '--default-buckets-behavior' "$script"; then fail 'implicit logs bucket behavior remains'; fi
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
contains 'sys.argv[17]' "$script"
contains 'json.load' "$script"
# shellcheck disable=SC2016
contains 'graphs-and-jev-builder@${PROJECT_ID}.iam.gserviceaccount.com' "$env_example"
# shellcheck disable=SC2016
contains 'graphs-and-jev-runner@${PROJECT_ID}.iam.gserviceaccount.com' "$env_example"
contains 'anthony.lui@archegon.com' "$script"
contains 'git diff-index --quiet HEAD --' "$script"
contains 'git status --porcelain' "$script"
contains 'git merge-base --is-ancestor HEAD' "$script"
contains 'gcloud meta list-files-for-upload' "$script"
contains '5242880' "$script"
contains 'gcloud builds submit' "$script"
# shellcheck disable=SC2016
contains '--service-account="$gjd_build_sa_resource"' "$script"
contains 'gcloud artifacts docker images describe' "$script"
# shellcheck disable=SC2016
contains '${gjd_IMAGE}@${gjd_digest}' "$script"
contains 'assert_effective_deployment' "$script"
contains 'gcloud beta run deploy' "$script"
# shellcheck disable=SC2016
contains '--tag="$gjd_candidate_tag"' "$script"
contains 'candidate_url' "$script"
contains 'candidate_tag' "$script"
contains 'check_anonymous' "$script"
contains '401|403' "$script"
contains 'deploy/verify.py' "$script"
contains 'gcloud auth print-identity-token' "$script"
contains 'gjd_service_exists=false' "$script"
contains 'classify_service_describe()' "$script"
contains 'gjd_describe_status' "$script"
contains 'gjd_describe_stderr' "$script"
if rg -F --quiet '>/dev/null 2>&1; then' "$script"; then fail 'service describe still collapses failures'; fi
# shellcheck disable=SC1090
source "$script"
classifier_dir="$(mktemp -d)"
printf '%s' '{"metadata":{"name":"graphs-and-jev"}}' >"$classifier_dir/out"
: >"$classifier_dir/err"
[[ "$(classify_service_describe 0 graphs-and-jev "$classifier_dir/out" "$classifier_dir/err")" == present ]] || fail 'existing service not classified present'
printf '\n' >"$classifier_dir/out"
printf '%s\n' 'ERROR: (gcloud.run.services.describe) Cannot find service [graphs-and-jev]' >"$classifier_dir/err"
[[ "$(classify_service_describe 1 graphs-and-jev "$classifier_dir/out" "$classifier_dir/err")" == absent ]] || fail 'newline-only stdout was not classified absent'
printf 'not-empty\n' >"$classifier_dir/out"
if classify_service_describe 1 graphs-and-jev "$classifier_dir/out" "$classifier_dir/err" >/dev/null 2>&1; then fail 'non-whitespace stdout was classified absent'; fi
printf ' \t\n' >"$classifier_dir/out"
[[ "$(classify_service_describe 1 graphs-and-jev "$classifier_dir/out" "$classifier_dir/err")" == absent ]] || fail 'whitespace-only stdout was not classified absent'
: >"$classifier_dir/out"
for code5_message in 'ERROR: (gcloud.run.services.describe) Cannot find service [graphs-and-jev]' NOT_FOUND PERMISSION_DENIED; do
  printf '%s
' "$code5_message" >"$classifier_dir/err"
  if classify_service_describe 5 graphs-and-jev "$classifier_dir/out" "$classifier_dir/err" >/dev/null 2>&1; then fail "exit 5 was misclassified absent: $code5_message"; fi
done
for failure in PERMISSION_DENIED UNAVAILABLE RESOURCE_EXHAUSTED; do
  printf '%s\n' "$failure" >"$classifier_dir/err"
  if classify_service_describe 1 graphs-and-jev "$classifier_dir/out" "$classifier_dir/err" >/dev/null 2>&1; then fail "$failure was misclassified absent"; fi
done
: >"$classifier_dir/err"
printf '%s' '{"metadata":{}}' >"$classifier_dir/out"
if classify_service_describe 0 graphs-and-jev "$classifier_dir/out" "$classifier_dir/err" >/dev/null 2>&1; then fail 'malformed success was classified present'; fi
rm -rf "$classifier_dir"

[[ "$(app_env_json)" == '{}' ]] || fail 'empty app settings did not render {}'
app_env_rendered="$(app_env_json 'BRAND_NAME=Shed "test" \ </script>, a=b' API_MODE=fixtures API_TIMEOUT_MS=9000)"
python3 -c 'import json,sys; data=json.loads(sys.argv[1]); assert data == {"API_MODE": "fixtures", "API_TIMEOUT_MS": "9000", "BRAND_NAME": "Shed \"test\" \\ </script>, a=b"}, data' "$app_env_rendered" || fail "app settings did not round-trip: $app_env_rendered"
# Same cases as invalid_site_settings_fail_at_startup / browser_parseable_base_urls_are_accepted in tests/api_boundary.rs.
for good_url in https://api.example.run.app http://localhost:8080 http://127.0.0.1:8080 'https://[::1]:8443'; do
  app_env_json "API_BASE_URL=$good_url" >/dev/null || fail "app_env_json rejected $good_url"
done
for bad_setting in API_MODE=remote API_MODE_CATALOGUE=LIVE API_TIMEOUT_MS=0 API_TIMEOUT_MS=-5 'API_BASE_URL=ftp://x' 'API_BASE_URL=https://user@x' 'API_BASE_URL=https://x y' \
  'API_BASE_URL=https://' 'API_BASE_URL=https://:' 'API_BASE_URL=https://[]' 'API_BASE_URL=https://[::1' 'API_BASE_URL=https://[::1]x' 'API_BASE_URL=https://[fe80::1%eth0]' \
  'API_BASE_URL=https://api.example.com:' 'API_BASE_URL=https://api.example.com:99999' 'API_BASE_URL=https://api.example.com:+80' \
  'API_BASE_URL=https://api..example.com' 'API_BASE_URL=https://-api.example.com' 'API_BASE_URL=https://1.2.3.999' 'API_BASE_URL=https://api.example.com/a b' \
  'API_BASE_URL=https://api.example.run.app/v1' 'API_BASE_URL=https://api.example.run.app/' 'API_BASE_URL=https://[::1]:8443?x=1' 'API_BASE_URL=https://api.example.run.app#f' \
  'API_ROUTE_HEALTH=api/health' 'API_ROUTE_CATALOGUE=//evil.example/catalogue' 'API_ROUTE_PIPELINE=/api/pipe line' 'API_ROUTE_PIPELINE=/\evil.example' \
  CATALOGUE_INTERPRETER=llm CATALOGUE_INTERPRETER=Gemini 'GEMINI_MODEL=Gemini 3.8' GEMINI_LOCATION=global/x GOOGLE_CLOUD_PROJECT=a/b \
  GEMINI_TIMEOUT_MS=0 GEMINI_TIMEOUT_MS=60001 GEMINI_THINKING_LEVEL=max; do
  if app_env_json "$bad_setting" >/dev/null 2>&1; then fail "app_env_json accepted $bad_setting"; fi
done
app_env_json CATALOGUE_INTERPRETER=gemini GEMINI_MODEL=gemini-3.8-flash GEMINI_LOCATION=global GEMINI_TIMEOUT_MS=8000 \
  GEMINI_THINKING_LEVEL=model-default GOOGLE_CLOUD_PROJECT=ninth-airship-386815 >/dev/null || fail 'app_env_json rejected valid Gemini settings'
# Local-only and test-only settings never reach a revision.
for local_key in GEMINI_ACCESS_TOKEN GEMINI_ENDPOINT GCE_METADATA_HOST GOOGLE_APPLICATION_CREDENTIALS; do
  if sed -n '/^  gjd_app_env_keys=(/,/^  )/p' "$script" | rg -q -w "$local_key"; then fail "$local_key is in the app env allowlist"; fi
done
for app_key in BRAND_ORG BRAND_NAME BRAND_TAGLINE BRAND_KIND BRAND_DISCLAIMER API_MODE API_BASE_URL API_TIMEOUT_MS API_ROUTE_HEALTH API_ROUTE_CATALOGUE API_ROUTE_PIPELINE API_MODE_CATALOGUE API_MODE_LINEAGE \
  CATALOGUE_INTERPRETER GEMINI_MODEL GEMINI_LOCATION GEMINI_TIMEOUT_MS GEMINI_THINKING_LEVEL GOOGLE_CLOUD_PROJECT; do
  contains "$app_key" "$script"
  contains "#${app_key}=" "$env_example"
done
contains 'gjd_app_env_args=(--env-vars-file="$gjd_app_env_file")' "$script"
contains 'gjd_app_env_args=(--clear-env-vars)' "$script"
contains '"${gjd_app_env_args[@]}"' "$script"
contains 'sys.argv[18]' "$script"
contains 'sys.argv[20]' "$script"
if rg -n -- '--(update|set)-env-vars' "$script"; then fail 'deploy must pass app settings through the validated env-vars file'; fi

contains '--no-allow-unauthenticated' "$script"
contains 'service_private' "$script"
contains 'candidate_zero_traffic' "$script"
contains 'expected_bootstrap' "$script"
contains 'gjd_deploy_traffic_args=(--no-allow-unauthenticated)' "$script"
contains 'gjd_deploy_traffic_args=(--no-traffic)' "$script"
contains 'Validated private bootstrap revision' "$script"
# IAM is service-wide: a candidate deploy onto an existing service must not touch it in either direction.
[[ "$(rg -c -- '--no-allow-unauthenticated' "$script")" == 1 ]] || fail 'IAM flag must appear only on the bootstrap deploy'
if rg -n -- '(^|[^-])--allow-unauthenticated|add-iam-policy-binding|set-iam-policy' "$script"; then fail 'deploy workflow changes service IAM'; fi
contains 'live traffic changed during candidate deploy' "$script"
contains 'CATALOGUE_INTERPRETER=gemini is presenter-only, but' "$script"
contains 'service IAM changed during candidate deploy' "$script"
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
{"metadata":{"name":"graphs-and-jev"},"status":{"url":"https://graphs-and-jev.example.run.app","traffic":[{"tag":"candidate-abc123","revisionName":"graphs-and-jev-abc123","percent":0,"url":"https://candidate-abc123---graphs-and-jev.example.run.app"}]}}
JSON
cat >"$validator_dir/revision.json" <<'JSON'
{"metadata":{"name":"graphs-and-jev-abc123","annotations":{"autoscaling.knative.dev/minScale":"0","autoscaling.knative.dev/maxScale":"2"}},"spec":{"serviceAccountName":"graphs-and-jev-runner@ninth-airship-386815.iam.gserviceaccount.com","containerConcurrency":20,"timeoutSeconds":30,"containers":[{"image":"australia-southeast1-docker.pkg.dev/ninth-airship-386815/graphs-and-jev/app@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","resources":{"limits":{"cpu":"1","memory":"512Mi"}},"startupProbe":{"httpGet":{"path":"/api/ready"}},"livenessProbe":{"httpGet":{"path":"/api/health"}}}]}}
JSON
cat >"$validator_dir/iam.json" <<'JSON'
{"bindings":[]}
JSON
python3 -c "$validator" \
  "$validator_dir/service.json" "$validator_dir/revision.json" "$validator_dir/iam.json" \
  'australia-southeast1-docker.pkg.dev/ninth-airship-386815/graphs-and-jev/app@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa' \
  'graphs-and-jev-runner@ninth-airship-386815.iam.gserviceaccount.com' 'graphs-and-jev-abc123' \
  0 2 20 1 512Mi 30 /api/ready /api/health candidate-abc123 false "$validator_dir/effective.json" '{}' false '{}'
python3 -c 'import json,sys; data=json.load(open(sys.argv[1])); assert data["revision"] == "graphs-and-jev-abc123"; assert data["image_digest"].endswith("a" * 64); assert data["service_private"] is True; assert data["candidate_zero_traffic"] is True; assert data["public_invocation"] is False; assert data["candidate_tag"] == "candidate-abc123"; assert data["candidate_url"].startswith("https://candidate-abc123---")' "$validator_dir/effective.json"
cat >"$validator_dir/service-bootstrap.json" <<'JSON'
{"metadata":{"name":"graphs-and-jev"},"status":{"url":"https://candidate.example.run.app","traffic":[{"tag":"candidate-abc123","revisionName":"graphs-and-jev-abc123","percent":100,"url":"https://candidate-abc123---graphs-and-jev.example.run.app"}]}}
JSON
cat >"$validator_dir/iam-private.json" <<'JSON'
{"bindings":[]}
JSON
python3 -c "$validator" \
  "$validator_dir/service-bootstrap.json" "$validator_dir/revision.json" "$validator_dir/iam-private.json" \
  'australia-southeast1-docker.pkg.dev/ninth-airship-386815/graphs-and-jev/app@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa' \
  'graphs-and-jev-runner@ninth-airship-386815.iam.gserviceaccount.com' 'graphs-and-jev-abc123' \
  0 2 20 1 512Mi 30 /api/ready /api/health candidate-abc123 true "$validator_dir/effective-bootstrap.json" '{}' false '{}'
python3 -c 'import json,sys; data=json.load(open(sys.argv[1])); assert data["service_private"] is True; assert data["candidate_zero_traffic"] is False; assert data["public_invocation"] is False; assert data["traffic"][0]["percent"] == 100; assert data["service_url"] == "https://candidate.example.run.app"' "$validator_dir/effective-bootstrap.json"
python3 - "$validator_dir/revision.json" "$validator_dir/revision-env.json" <<'PYENV'
import json, sys
revision = json.load(open(sys.argv[1]))
revision["spec"]["containers"][0]["env"] = [{"name": "API_MODE", "value": "live"}, {"name": "BRAND_NAME", "value": "Shed \"test\""}]
json.dump(revision, open(sys.argv[2], "w"))
PYENV
validator_args=(
  'australia-southeast1-docker.pkg.dev/ninth-airship-386815/graphs-and-jev/app@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'
  'graphs-and-jev-runner@ninth-airship-386815.iam.gserviceaccount.com' 'graphs-and-jev-abc123'
  0 2 20 1 512Mi 30 /api/ready /api/health candidate-abc123 false "$validator_dir/effective-env.json"
)
python3 -c "$validator" "$validator_dir/service.json" "$validator_dir/revision-env.json" "$validator_dir/iam.json" \
  "${validator_args[@]}" '{"API_MODE": "live", "BRAND_NAME": "Shed \"test\""}' false '{}'
python3 -c 'import json,sys; data=json.load(open(sys.argv[1])); assert data["app_env"] == {"API_MODE": "live", "BRAND_NAME": "Shed \"test\""}, data' "$validator_dir/effective-env.json"
for stale_env in '{}' '{"API_MODE": "fixtures", "BRAND_NAME": "Shed \"test\""}'; do
  if python3 -c "$validator" "$validator_dir/service.json" "$validator_dir/revision-env.json" "$validator_dir/iam.json" \
    "${validator_args[@]}" "$stale_env" false '{}' >/dev/null 2>&1; then fail "validator accepted mismatched app env: $stale_env"; fi
done
python3 - "$validator_dir/revision.json" "$validator_dir/revision-secret.json" <<'PYSECRET'
import json,sys
revision=json.load(open(sys.argv[1])); revision["spec"]["containers"][0]["env"]=[
 {"name":"DECISION_BACKEND","value":"jev"}, {"name":"SERVICE_AUTH_MODE","value":"cloud-run-iam"}, {"name":"JEV_MODEL","value":"jev-1.2.3"},
 {"name":"TYPESAFE_API_KEY","valueFrom":{"secretKeyRef":{"name":"typesafe-api-key","key":"7"}}}]
json.dump(revision,open(sys.argv[2],"w"))
PYSECRET
python3 -c "$validator" "$validator_dir/service.json" "$validator_dir/revision-secret.json" "$validator_dir/iam.json" \
  "${validator_args[@]}" '{"DECISION_BACKEND":"jev","JEV_MODEL":"jev-1.2.3","SERVICE_AUTH_MODE":"cloud-run-iam"}' false '{}' typesafe-api-key 7
for wrong in 'wrong 7' 'typesafe-api-key 8'; do
  read -r secret version <<<"$wrong"
  if python3 -c "$validator" "$validator_dir/service.json" "$validator_dir/revision-secret.json" "$validator_dir/iam.json" \
    "${validator_args[@]}" '{"DECISION_BACKEND":"jev","JEV_MODEL":"jev-1.2.3","SERVICE_AUTH_MODE":"cloud-run-iam"}' false '{}' "$secret" "$version" >/dev/null 2>&1; then fail "validator accepted wrong secret reference: $wrong"; fi
done
# Existing public service: the candidate deploy must leave allUsers access and live traffic exactly as they were.
cat >"$validator_dir/service-live.json" <<'JSON'
{"metadata":{"name":"graphs-and-jev"},"status":{"url":"https://graphs-and-jev.example.run.app","traffic":[{"revisionName":"graphs-and-jev-old","percent":100},{"tag":"candidate-abc123","revisionName":"graphs-and-jev-abc123","percent":0,"url":"https://candidate-abc123---graphs-and-jev.example.run.app"}]}}
JSON
cat >"$validator_dir/iam-public.json" <<'JSON'
{"bindings":[{"role":"roles/run.invoker","members":["allUsers"]}]}
JSON
python3 -c "$validator" "$validator_dir/service-live.json" "$validator_dir/revision.json" "$validator_dir/iam-public.json" \
  "${validator_args[@]}" '{}' true '{"graphs-and-jev-old": 100}' || fail 'validator rejected an untouched public service'
for case in "iam.json|true|{\"graphs-and-jev-old\": 100}" "iam-public.json|true|{\"graphs-and-jev-older\": 100}" "iam-public.json|false|{\"graphs-and-jev-old\": 100}"; do
  IFS='|' read -r iam_file public serving <<<"$case"
  if python3 -c "$validator" "$validator_dir/service-live.json" "$validator_dir/revision.json" "$validator_dir/$iam_file" \
    "${validator_args[@]}" '{}' "$public" "$serving" >/dev/null 2>&1; then fail "validator accepted changed access or traffic: $case"; fi
done
# Disabling the invoker IAM check makes a service public even without an allUsers binding.
python3 - "$validator_dir/service.json" "$validator_dir/service-disabled.json" <<'PYDISABLED'
import json, sys
service=json.load(open(sys.argv[1]))
service.setdefault("metadata", {}).setdefault("annotations", {})["run.googleapis.com/invoker-iam-disabled"]="true"
json.dump(service, open(sys.argv[2], "w"))
PYDISABLED
if python3 -c "$validator" "$validator_dir/service-disabled.json" "$validator_dir/revision.json" "$validator_dir/iam.json" \
  "${validator_args[@]}" '{}' false '{}' >/dev/null 2>&1; then fail 'validator treated disabled invoker IAM check as private'; fi

private_promote="$root/deploy/promote-private.sh"
[[ -x "$private_promote" ]] || fail "$private_promote must exist and be executable"
bash -n "$private_promote"
if rg -n 'add-iam-policy-binding|set-iam-policy' "$private_promote"; then fail 'private promotion can modify IAM'; fi
contains 'GEMINI_VERIFY=1' "$private_promote"
contains 'VERIFY_EXPECTED_MODEL' "$private_promote"
if rg -n '\b(gjd_|GJ_)[A-Za-z_]*' <<<"$validator"; then fail 'shell namespace leaked into embedded Python'; fi
trap - EXIT
rm -rf "$validator_dir"

promote="$root/deploy/promote.sh"
[[ -x "$promote" ]] || fail "$promote must exist and be executable"
bash -n "$promote"
python3 -c 'import ast, sys; ast.parse(open(sys.argv[1]).read())' "$root/deploy/verify.py"
python3 - "$root/deploy/verify.py" <<'PYREDIRECT'
import http.server, os, subprocess, sys, threading
verify = sys.argv[1]
seen = []
class Target(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        seen.append(self.headers.get("Authorization"))
        self.send_response(200); self.end_headers(); self.wfile.write(b'{"ok":true}')
    def log_message(self, *_): pass
class Redirect(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(302); self.send_header("Location", target); self.end_headers()
    def log_message(self, *_): pass
def server(handler):
    value=http.server.ThreadingHTTPServer(("127.0.0.1",0),handler)
    threading.Thread(target=value.serve_forever,daemon=True).start(); return value
sink=server(Target); target=f"http://127.0.0.1:{sink.server_port}/stolen"
source=server(Redirect)
env={**os.environ,"VERIFY_BEARER":"secret-identity-token","VERIFY_ATTEMPTS":"1"}
run=subprocess.run([sys.executable,verify,f"http://127.0.0.1:{source.server_port}"],env=env,text=True,capture_output=True,timeout=10)
source.shutdown(); sink.shutdown()
assert run.returncode != 0, run.stdout
assert not seen, f"bearer reached redirect target: {seen}"
assert "HTTP 302" in run.stderr and target in run.stderr, run.stderr
PYREDIRECT
python3 - "$root/deploy/verify.py" <<'PYRETIRED'
import http.server, json, os, subprocess, sys, threading
verify=sys.argv[1]; seen=[]
locations={"/dcceew/":"/environmental-agency/","/parkinsons/":None}
hi_message="Hello! Friendly catalogue help."
class App(http.server.BaseHTTPRequestHandler):
    def send(self, code, body=b"", location=None, content="application/json"):
        self.send_response(code)
        if location: self.send_header("Location", location)
        self.send_header("Content-Type", content); self.end_headers(); self.wfile.write(body)
    def do_GET(self):
        seen.append((self.path, self.headers.get("Authorization")))
        if self.path == "/api/health": return self.send(200,b'{"ok":true}')
        if self.path == "/api/ready": return self.send(200,b'{"ok":true}')
        if self.path == "/api/catalogue": return self.send(200,b'{"sources":[1]}')
        if self.path in ("/","/technical-guide/","/environmental-agency/","/lineage-migration/","/little-bunnies/"):
            return self.send(200,b'<html data-brand="name">',content="text/html")
        if self.path in locations: return self.send(308,location=locations[self.path] or f"http://127.0.0.1:{self.server.server_port}/lineage-migration/")
        if self.path == "/config.js": return self.send(200,b'window.SITE_CONFIG = {};\n',content="text/javascript")
        return self.send(404)
    def do_POST(self):
        size=int(self.headers.get("Content-Length","0")); body=json.loads(self.rfile.read(size))
        if body.get("domain") == "lineage": return self.send(501,b'{}')
        outcomes={"annual CO2 for Australia in 2024":"answer","emissions":"clarify","hi":"clarify","write me a poem about the sea":"reject"}
        response={"outcome":outcomes[body["request"]]}
        if body["request"] == "hi": response["message"]=hi_message
        return self.send(200,json.dumps(response).encode())
    def log_message(self,*_): pass
server=http.server.ThreadingHTTPServer(("127.0.0.1",0),App)
threading.Thread(target=server.serve_forever,daemon=True).start()
base=f"http://127.0.0.1:{server.server_port}"
env={**os.environ,"VERIFY_BEARER":"retired-route-token","VERIFY_ATTEMPTS":"1"}
run=subprocess.run([sys.executable,verify,base],env=env,text=True,capture_output=True,timeout=10)
assert run.returncode == 0, run.stderr
for old in ("/dcceew/","/parkinsons/"):
    assert (old,"Bearer retired-route-token") in seen, (old,seen)
assert "verify passed" in run.stdout
hi_message="A generic clarification without the friendly marker."
run=subprocess.run([sys.executable,verify,base],env=env,text=True,capture_output=True,timeout=10)
assert run.returncode != 0 and "friendly Hello marker" in run.stderr, run.stderr
hi_message="Hello! Friendly catalogue help."
for unsafe in (
    "https://127.0.0.1/environmental-agency/",
    "http://example.invalid/environmental-agency/",
    "http://127.0.0.1:1/environmental-agency/",
    "/wrong-path/",
    "/environmental-agency/?query=unsafe",
):
    locations["/dcceew/"]=unsafe
    run=subprocess.run([sys.executable,verify,base],env=env,text=True,capture_output=True,timeout=10)
    assert run.returncode != 0 and "expected HTTP 308 to same-origin /environmental-agency/" in run.stderr, (unsafe,run.stderr)
server.shutdown(); server.server_close()
PYRETIRED
python3 - "$root/deploy/verify.py" <<'PYJEVVERIFY'
import http.server,json,os,subprocess,sys,threading
verify=sys.argv[1]; mode="valid"
def pipeline():
 response={"outcome":"review","backend":"jev","backend_label":"TypeSafe hosted Jev","model":"jev-1.2.3","residency":"hosted","policy":{"backend":"jev","model":"jev-1.2.3","residency":"hosted"},"stages":[{"stage":"gate","title":"Gate","answer":{"type":"choice","probabilities":{"answer":0.46,"clarify":0.27,"reject":0.27},"top":"answer","choice":"answer","confidence":0.19}}],"totals":{"decisions":1,"calls":1,"latency_ms":3.0,"input_tokens":12,"cost_usd":0.01,"usage_complete":True,"backend_errors":0},"interpretation":{"by":"gemini","model":"gemini-3.8-flash","request":"PM2.5 by country and year from 2019 to 2024","note":None}}
 if mode=="catalogue": response.update(backend="catalogue",model="catalogue-rules-v1",residency="local")
 if mode=="bad-contract": response["stages"]=[]; response["totals"]["calls"]=0
 if mode=="backend-error": response["totals"]["backend_errors"]=1
 if mode=="no-policy": response.pop("policy")
 if mode=="tie": response["stages"][0]["answer"].update(probabilities={"answer":0.5,"clarify":0.5},top="clarify",choice="clarify",confidence=0.0)
 if mode=="rounded": response["stages"][0]["answer"].update(probabilities={"answer":0.07,"clarify":0.75,"reject":0.18},top="clarify",choice="clarify",confidence=0.62)
 return response
class App(http.server.BaseHTTPRequestHandler):
 def send(self,code,body=b"",location=None,content="application/json"):
  self.send_response(code)
  if location:self.send_header("Location",location)
  self.send_header("Content-Type",content);self.end_headers();self.wfile.write(body)
 def do_GET(self):
  if self.path=="/api/health":return self.send(200,b'{"ok":true}')
  if self.path=="/api/ready":return self.send(200,b'{"ok":true}')
  if self.path=="/api/catalogue":return self.send(200,b'{"sources":[1]}')
  if self.path in ("/","/technical-guide/","/environmental-agency/","/lineage-migration/","/little-bunnies/"):return self.send(200,b'<html data-brand="name">',content="text/html")
  if self.path=="/dcceew/":return self.send(308,location="/environmental-agency/")
  if self.path=="/parkinsons/":return self.send(308,location="/lineage-migration/")
  if self.path=="/config.js":return self.send(200,b'window.SITE_CONFIG = {};\n',content="text/javascript")
  return self.send(404)
 def do_POST(self):
  size=int(self.headers.get("Content-Length","0")); body=json.loads(self.rfile.read(size))
  if body.get("domain")=="lineage":return self.send(501,b'{}')
  if body["request"]=="could I see fine particles for each nation, yearly, 2019 through 2024?": response=pipeline()
  elif body["request"]=="hi": response={**pipeline(),"outcome":"clarify","message":"Hello!"} if mode=="hosted-hi" else {"outcome":"clarify","message":"Hello!","backend":"catalogue",**({"interpretation":{"by":"gemini","model":"gemini-3.8-flash","request":"hi","note":None}} if mode=="gemini-hi" else {})}
  else:
   response=pipeline()
   if mode=="public":
    outcomes={"annual CO2 for Australia in 2024":"answer","emissions":"clarify","write me a poem about the sea":"reject"};response={"outcome":outcomes[body["request"]]}
  return self.send(200,json.dumps(response).encode())
 def log_message(self,*_):pass
server=http.server.ThreadingHTTPServer(("127.0.0.1",0),App);threading.Thread(target=server.serve_forever,daemon=True).start();base=f"http://127.0.0.1:{server.server_port}"
env={**os.environ,"VERIFY_ATTEMPTS":"1","JEV_VERIFY":"1","JEV_VERIFY_EXPECTED_MODEL":"jev-1.2.3"}
run=subprocess.run([sys.executable,verify,base],env=env,text=True,capture_output=True);assert run.returncode==0,run.stderr
for mode in ("catalogue","bad-contract","backend-error","no-policy"):
 run=subprocess.run([sys.executable,verify,base],env=env,text=True,capture_output=True);assert run.returncode!=0,(mode,run.stdout,run.stderr)
mode="hosted-hi"
run=subprocess.run([sys.executable,verify,base],env=env,text=True,capture_output=True);assert run.returncode!=0 and "hi spent a hosted decision call" in run.stderr,(mode,run.stdout,run.stderr)
mode="gemini-hi"
run=subprocess.run([sys.executable,verify,base],env=env,text=True,capture_output=True);assert run.returncode!=0 and "hi spent a Gemini call" in run.stderr,(mode,run.stdout,run.stderr)
mode="tie"
run=subprocess.run([sys.executable,verify,base],env=env,text=True,capture_output=True);assert run.returncode==0,run.stderr
mode="rounded"
run=subprocess.run([sys.executable,verify,base],env=env,text=True,capture_output=True);assert run.returncode==0,run.stderr
mode="valid";env.update(GEMINI_VERIFY="1",VERIFY_EXPECTED_MODEL="gemini-3.8-flash")
run=subprocess.run([sys.executable,verify,base],env=env,text=True,capture_output=True);assert run.returncode==0,run.stderr
mode="public"; public_env={**os.environ,"VERIFY_ATTEMPTS":"1"}
run=subprocess.run([sys.executable,verify,base],env=public_env,text=True,capture_output=True);assert run.returncode==0,run.stderr
server.shutdown();server.server_close()
PYJEVVERIFY
promote_dir="$(mktemp -d)"
mkdir -p "$promote_dir/deploy" "$promote_dir/bin"
cp "$promote" "$promote_dir/deploy/promote.sh"
cat >"$promote_dir/deploy/verify.py" <<'FAKE'
import os, sys
open(os.environ["MOCK_LOG"], "a").write(f"verify {sys.argv[1]} bearer={'yes' if os.environ.get('VERIFY_BEARER') else 'no'}\n")
sys.exit(1 if sys.argv[1] == os.environ.get("FAIL_VERIFY_URL") else 0)
FAKE
cat >"$promote_dir/bin/gcloud" <<'MOCK'
printf 'gcloud %s\n' "$*" >>"$MOCK_LOG"
case "$*" in
  "auth list"*) printf '%s\n' anthony.lui@archegon.com ;;
  "auth print-identity-token"*) printf '%s\n' fake-token ;;
  "config get-value project"*) printf '%s\n' ninth-airship-386815 ;;
  "run services describe"*)
    if [[ -e "$MOCK_STATE/promoted" ]]; then
      printf '%s\n' '{"status":{"url":"https://live.example.run.app","traffic":[{"revisionName":"graphs-and-jev-new","percent":100,"tag":"candidate-new","url":"https://candidate-new---live.example.run.app"}]}}'
    else
      printf '%s\n' '{"status":{"url":"https://live.example.run.app","traffic":[{"revisionName":"graphs-and-jev-old","percent":100},{"revisionName":"graphs-and-jev-new","percent":0,"tag":"candidate-new","url":"https://candidate-new---live.example.run.app"}]}}'
    fi ;;
  "run services get-iam-policy"*) printf '%s\n' '{"bindings":[{"role":"roles/run.invoker","members":["allUsers"]}],"etag":"BwX"}' ;;
  "run revisions describe"*"--format=json"*) printf '{"spec":{"containers":[{"env":[{"name":"CATALOGUE_INTERPRETER","value":"%s"},{"name":"DECISION_BACKEND","value":"%s"},{"name":"SERVICE_AUTH_MODE","value":"%s"}]}]}}\n' "${MOCK_INTERPRETER:-rules}" "${MOCK_BACKEND:-rules}" "${MOCK_AUTH:-public}" ;;
  "run revisions describe"*) printf '%s\n' True ;;
  "run services update-traffic"*"graphs-and-jev-new=100"*) : >"$MOCK_STATE/promoted" ;;
  "run services update-traffic"*|"run services add-iam-policy-binding"*|"run services set-iam-policy"*) ;;
  *) printf 'unexpected gcloud: %s\n' "$*" >&2; exit 72 ;;
esac
MOCK
chmod +x "$promote_dir/bin/gcloud" "$promote_dir/deploy/promote.sh"
run_promote() {
  rm -rf "$promote_dir/state" && mkdir "$promote_dir/state"
  : >"$promote_dir/log"
  PATH="$promote_dir/bin:$PATH" MOCK_LOG="$promote_dir/log" MOCK_STATE="$promote_dir/state" FAIL_VERIFY_URL="$1" \
    "$promote_dir/deploy/promote.sh" graphs-and-jev-new >"$promote_dir/out" 2>&1
}
run_promote '' || fail "promotion failed: $(cat "$promote_dir/out")"
log="$(cat "$promote_dir/log")"
[[ "$log" == *'verify https://candidate-new---live.example.run.app bearer=yes'* ]] || fail "candidate not verified with a token: $log"
[[ "$log" == *'--to-revisions=graphs-and-jev-new=100'* && "$log" == *'add-iam-policy-binding graphs-and-jev'*'--member=allUsers --role=roles/run.invoker'* ]] || fail "promotion did not switch traffic and restore public access: $log"
[[ "$log" == *'verify https://live.example.run.app bearer=no'* ]] || fail "canonical URL not verified anonymously: $log"
[[ "$log" != *'set-iam-policy'* && "$log" != *'graphs-and-jev-old=100'* ]] || fail "successful promotion rolled back: $log"
[[ "$log" != *fake-token* && "$(cat "$promote_dir/out")" != *fake-token* ]] || fail 'identity token leaked into argv or output'
if run_promote 'https://candidate-new---live.example.run.app'; then fail 'promotion passed with a failing candidate'; fi
log="$(cat "$promote_dir/log")"
[[ "$log" != *update-traffic* && "$log" != *iam-policy-binding* && "$log" != *set-iam-policy* ]] || fail "failed candidate still changed the service: $log"
if run_promote 'https://live.example.run.app'; then fail 'promotion passed with a failing canonical URL'; fi
log="$(cat "$promote_dir/log")"
[[ "$log" == *'--to-revisions=graphs-and-jev-old=100'* && "$log" == *'set-iam-policy graphs-and-jev'* ]] || fail "failed canonical check did not roll back traffic and IAM: $log"
[[ "$(cat "$promote_dir/out")" == *'rolling back'* ]] || fail 'rollback was not reported'
if MOCK_INTERPRETER=gemini run_promote ''; then fail 'promotion made a Gemini presenter revision public'; fi
log="$(cat "$promote_dir/log")"
[[ "$log" != *update-traffic* && "$log" != *iam-policy-binding* && "$log" != *set-iam-policy* && "$log" != *'verify '* ]] || fail "Gemini revision still changed the service: $log"
[[ "$(cat "$promote_dir/out")" == *'presenter-only and is never made public'* ]] || fail 'Gemini refusal was not reported'
if MOCK_BACKEND=jev run_promote ''; then fail 'promotion made a Jev revision public'; fi
[[ "$(cat "$promote_dir/out")" == *'private-only and is never made public'* ]] || fail 'Jev refusal was not reported'
if MOCK_AUTH=cloud-run-iam run_promote ''; then fail 'promotion made a Cloud Run IAM revision public'; fi
[[ "$(cat "$promote_dir/out")" == *'private-only and is never made public'* ]] || fail 'private auth refusal was not reported'
rm -rf "$promote_dir"

private_dir="$(mktemp -d)"
mkdir -p "$private_dir/deploy" "$private_dir/bin" "$private_dir/state"
cp "$private_promote" "$private_dir/deploy/promote-private.sh"
cat >"$private_dir/deploy/verify.py" <<'FAKE'
import os, sys
open(os.environ["MOCK_LOG"], "a").write(f"verify {sys.argv[1]} bearer={'yes' if os.environ.get('VERIFY_BEARER') else 'no'} gemini={os.environ.get('GEMINI_VERIFY')} model={os.environ.get('VERIFY_EXPECTED_MODEL')} jev={os.environ.get('JEV_VERIFY')} jev_model={os.environ.get('JEV_VERIFY_EXPECTED_MODEL')}\n")
raise SystemExit(1 if sys.argv[1] == os.environ.get("FAIL_VERIFY_URL") else 0)
FAKE
cat >"$private_dir/bin/gcloud" <<'MOCK'
#!/usr/bin/env bash
printf 'gcloud %s\n' "$*" >>"$MOCK_LOG"
case "$*" in
  "auth list"*) printf '%s\n' anthony.lui@archegon.com ;;
  "auth print-identity-token --audiences="*)
    [[ "${MOCK_AUDIENCE_FAILURE:-}" == other ]] && { printf '%s\n' 'network unavailable' >&2; exit 1; }
    [[ "${MOCK_AUDIENCE_FAILURE:-}" == account ]] && { printf '%s\n' 'Invalid account type for --audiences' >&2; exit 1; }
    printf '%s\n' private-token ;;
  "auth print-identity-token"*) printf '%s\n' private-token ;;
  "config get-value project"*) printf '%s\n' ninth-airship-386815 ;;
  "run services get-iam-policy"*) printf '%s\n' '{"bindings":[{"role":"roles/run.invoker","members":["user:presenter@example.com"]}]}' ;;
  "run revisions describe"*)
    case "${MOCK_PRIVATE_MODE:-combined}" in
      gemini) printf '%s\n' '{"spec":{"containers":[{"env":[{"name":"CATALOGUE_INTERPRETER","value":"gemini"},{"name":"GEMINI_MODEL","value":"gemini-3.8-flash"},{"name":"DECISION_BACKEND","value":"catalogue"},{"name":"SERVICE_AUTH_MODE","value":"cloud-run-iam"}]}]}}' ;;
      jev) printf '%s\n' '{"spec":{"containers":[{"env":[{"name":"CATALOGUE_INTERPRETER","value":"rules"},{"name":"DECISION_BACKEND","value":"jev"},{"name":"SERVICE_AUTH_MODE","value":"cloud-run-iam"},{"name":"JEV_MODEL","value":"jev-1.2.3"},{"name":"TYPESAFE_API_KEY","valueFrom":{"secretKeyRef":{"name":"typesafe-api-key","key":"1"}}}]}]}}' ;;
      wrong-name) printf '%s\n' '{"spec":{"containers":[{"env":[{"name":"DECISION_BACKEND","value":"jev"},{"name":"SERVICE_AUTH_MODE","value":"cloud-run-iam"},{"name":"JEV_MODEL","value":"jev-1.2.3"},{"name":"TYPESAFE_API_KEY","valueFrom":{"secretKeyRef":{"name":"wrong","key":"1"}}}]}]}}' ;;
      wrong-version) printf '%s\n' '{"spec":{"containers":[{"env":[{"name":"DECISION_BACKEND","value":"jev"},{"name":"SERVICE_AUTH_MODE","value":"cloud-run-iam"},{"name":"JEV_MODEL","value":"jev-1.2.3"},{"name":"TYPESAFE_API_KEY","valueFrom":{"secretKeyRef":{"name":"typesafe-api-key","key":"2"}}}]}]}}' ;;
      combined) printf '%s\n' '{"spec":{"containers":[{"env":[{"name":"CATALOGUE_INTERPRETER","value":"gemini"},{"name":"GEMINI_MODEL","value":"gemini-3.8-flash"},{"name":"DECISION_BACKEND","value":"jev"},{"name":"SERVICE_AUTH_MODE","value":"cloud-run-iam"},{"name":"JEV_MODEL","value":"jev-1.2.3"},{"name":"TYPESAFE_API_KEY","valueFrom":{"secretKeyRef":{"name":"typesafe-api-key","key":"1"}}}]}]}}' ;;
    esac ;;
  "run services describe"*)
    if [[ -e "$MOCK_STATE/promoted" ]]; then
      printf '%s\n' '{"metadata":{"annotations":{}},"status":{"url":"https://private.example.run.app","traffic":[{"revisionName":"graphs-and-jev-presenter-new","percent":100}]}}'
    else
      printf '%s\n' '{"metadata":{"annotations":{}},"status":{"url":"https://private.example.run.app","traffic":[{"revisionName":"graphs-and-jev-presenter-old","percent":100},{"revisionName":"graphs-and-jev-presenter-new","percent":0,"tag":"candidate-new","url":"https://candidate-new---private.example.run.app"}]}}'
    fi ;;
  "run services update-traffic"*"graphs-and-jev-presenter-new=100"*) : >"$MOCK_STATE/promoted" ;;
  "run services update-traffic"*"graphs-and-jev-presenter-old=100"*) rm -f "$MOCK_STATE/promoted" ;;
  *) printf 'unexpected gcloud: %s\n' "$*" >&2; exit 72 ;;
esac
MOCK
chmod +x "$private_dir/bin/gcloud" "$private_dir/deploy/promote-private.sh"
cat >"$private_dir/private.env" <<'ENV'
GEMINI_MODEL=gemini-3.8-flash
JEV_MODEL=jev-1.2.3
TYPESAFE_SECRET_RESOURCE=projects/ninth-airship-386815/secrets/typesafe-api-key/versions/1
ENV
run_private_promote() {
  rm -f "$private_dir/state/promoted"; : >"$private_dir/log"
  PATH="$private_dir/bin:$PATH" MOCK_LOG="$private_dir/log" MOCK_STATE="$private_dir/state" MOCK_PRIVATE_MODE="${2:-combined}" FAIL_VERIFY_URL="$1" \
    "$private_dir/deploy/promote-private.sh" graphs-and-jev-presenter-new "$private_dir/private.env" >"$private_dir/out" 2>&1
}
run_private_promote '' || fail "private promotion failed: $(cat "$private_dir/out")"
log="$(cat "$private_dir/log")"
[[ "$log" == *'auth print-identity-token --audiences=https://private.example.run.app'* && "$log" == *'verify https://candidate-new---private.example.run.app bearer=yes gemini=1 model=gemini-3.8-flash jev=1 jev_model=jev-1.2.3'* ]] || fail "private candidate was not Gemini-verified with auth: $log"
[[ "$(rg -c -F 'auth print-identity-token --audiences=https://private.example.run.app' <<<"$log")" == 2 && "$log" == *'verify https://private.example.run.app bearer=yes gemini=1 model=gemini-3.8-flash jev=1 jev_model=jev-1.2.3'* ]] || fail "private canonical URL was not Gemini-verified with auth: $log"
MOCK_AUDIENCE_FAILURE=account run_private_promote '' || fail "human-account private promotion failed: $(cat "$private_dir/out")"
log="$(cat "$private_dir/log")"
[[ "$(rg -c -F 'auth print-identity-token --audiences=https://private.example.run.app' <<<"$log")" == 2 && "$(rg -c -x -F 'gcloud auth print-identity-token' <<<"$log")" == 2 ]] || fail "human-account token fallback missing: $log"
if MOCK_AUDIENCE_FAILURE=other run_private_promote ''; then fail 'private promotion masked unrelated audience-token failure'; fi
log="$(cat "$private_dir/log")"
[[ "$log" != *$'gcloud auth print-identity-token\n'* ]] || fail "unrelated token failure used fallback: $log"
[[ "$log" != *'add-iam-policy-binding'* && "$log" != *'set-iam-policy'* ]] || fail "private promotion changed IAM: $log"
run_private_promote '' gemini || fail "Gemini-only private promotion failed: $(cat "$private_dir/out")"
log="$(cat "$private_dir/log")"; [[ "$log" == *'gemini=1 model=gemini-3.8-flash jev=None jev_model=None'* ]] || fail "Gemini-only verification modes wrong: $log"
run_private_promote '' jev || fail "Jev-only private promotion failed: $(cat "$private_dir/out")"
log="$(cat "$private_dir/log")"; [[ "$log" == *'gemini=None model=None jev=1 jev_model=jev-1.2.3'* ]] || fail "Jev-only verification modes wrong: $log"
for wrong in wrong-name wrong-version; do
  if run_private_promote '' "$wrong"; then fail "private promotion accepted $wrong secret reference"; fi
  [[ "$(cat "$private_dir/out")" == *'secret reference does not match'* ]] || fail "$wrong refusal missing: $(cat "$private_dir/out")"
done
if run_private_promote 'https://private.example.run.app'; then fail 'private promotion passed failed canonical verification'; fi
log="$(cat "$private_dir/log")"
[[ "$log" == *'--to-revisions=graphs-and-jev-presenter-old=100'* ]] || fail "private promotion did not roll back traffic: $log"
[[ "$log" != *'add-iam-policy-binding'* && "$log" != *'set-iam-policy'* ]] || fail "private rollback changed IAM: $log"
rm -rf "$private_dir"

printf 'deploy static checks passed\n'
