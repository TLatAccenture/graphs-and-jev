#!/usr/bin/env bash
# Promote a private Gemini and/or hosted-Jev candidate without changing service IAM or IAP settings.
main() (
  set -euo pipefail
  local root project region service account revision state candidate_url service_url token rollback_args
  local gemini_model jev_model secret_name secret_version expected_secret_name expected_secret_version verify_error
  local -a verify_env
  root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
  [[ $# -ge 1 && $# -le 2 ]] || { printf 'usage: %s REVISION [environment-file]\n' "$0" >&2; exit 64; }
  revision="$1"; [[ $# == 1 ]] || source "$2"
  project="${PROJECT_ID:-ninth-airship-386815}"; region="${REGION:-australia-southeast1}"
  service="${SERVICE:-graphs-and-jev-presenter}"; account="${EXPECTED_ACCOUNT:-anthony.lui@archegon.com}"
  fail() { printf 'private promote failed: %s\n' "$*" >&2; exit 1; }
  [[ "$revision" =~ ^${service}-[a-z0-9-]+$ ]] || fail "revision must belong to $service"
  [[ "$(gcloud auth list --filter=status:ACTIVE --format='value(account)')" == "$account" ]] || fail "active account must be $account"
  [[ "$(gcloud config get-value project 2>/dev/null)" == "$project" ]] || fail "active project must be $project"
  state="$(mktemp -d)"; trap 'rm -rf "$state"' EXIT
  gcloud run services describe "$service" --project="$project" --region="$region" --format=json >"$state/before.json"
  gcloud run services get-iam-policy "$service" --project="$project" --region="$region" --format=json >"$state/iam-before.json"
  python3 - "$state/before.json" "$state/iam-before.json" <<'PY'
import json,sys
s=json.load(open(sys.argv[1])); iam=json.load(open(sys.argv[2])); a=s.get("metadata",{}).get("annotations",{}).get("run.googleapis.com/invoker-iam-disabled")
if a not in (None,"false") or any(b.get("role")=="roles/run.invoker" and "allUsers" in b.get("members",[]) for b in iam.get("bindings",[])): raise SystemExit("service is not verifiably private")
PY
  rollback_args="$(python3 -c 'import json,sys; t=[x for x in json.load(open(sys.argv[1]))["status"]["traffic"] if x.get("percent")]; print("--to-revisions="+",".join("%s=%s" % (x["revisionName"], x["percent"]) for x in t))' "$state/before.json")" || fail 'cannot record traffic'
  candidate_url="$(python3 -c 'import json,sys; x=[t for t in json.load(open(sys.argv[1]))["status"]["traffic"] if t.get("revisionName")==sys.argv[2] and t.get("tag")]; print(x[0].get("url","") if len(x)==1 else "")' "$state/before.json" "$revision")"
  service_url="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["status"]["url"])' "$state/before.json")"
  gcloud run revisions describe "$revision" --project="$project" --region="$region" --format=json >"$state/revision.json"
  read -r gemini_model jev_model secret_name secret_version < <(python3 - "$state/revision.json" <<'PY'
import json,sys
c=json.load(open(sys.argv[1]))["spec"]["containers"][0]; env=c.get("env",[])
values={x["name"]:x["value"] for x in env if "value" in x}; refs={x["name"]:x["valueFrom"] for x in env if "valueFrom" in x}
gemini=values.get("CATALOGUE_INTERPRETER")=="gemini"; jev=values.get("DECISION_BACKEND")=="jev"
assert gemini or jev
assert values.get("SERVICE_AUTH_MODE")=="cloud-run-iam"
if gemini: assert values.get("GEMINI_MODEL")
ref=refs.get("TYPESAFE_API_KEY",{}).get("secretKeyRef",{}) if jev else {}
if jev: assert values.get("JEV_MODEL") and ref.get("name") and str(ref.get("key","")).isdigit() and "TYPESAFE_API_KEY" not in values
print(values.get("GEMINI_MODEL","-"), values.get("JEV_MODEL","-"), ref.get("name","-"), ref.get("key","-"))
PY
  ) || fail 'candidate is not a valid private Gemini and/or Jev revision'
  verify_env=()
  if [[ "$gemini_model" != - ]]; then
    [[ -z "${GEMINI_MODEL:-}" || "$gemini_model" == "$GEMINI_MODEL" ]] || fail 'candidate Gemini model does not match expected model'
    verify_env+=(GEMINI_VERIFY=1 VERIFY_EXPECTED_MODEL="$gemini_model")
  fi
  if [[ "$jev_model" != - ]]; then
    [[ -n "${TYPESAFE_SECRET_RESOURCE:-}" && "$TYPESAFE_SECRET_RESOURCE" =~ ^projects/([^/]+)/secrets/([^/]+)/versions/([1-9][0-9]*)$ ]] || fail 'Jev promotion requires a numeric TYPESAFE_SECRET_RESOURCE'
    [[ "${BASH_REMATCH[1]}" == "$project" ]] || fail 'TYPESAFE_SECRET_RESOURCE must be in the deployment project'
    expected_secret_name="${BASH_REMATCH[2]}"; expected_secret_version="${BASH_REMATCH[3]}"
    [[ "$secret_name" == "$expected_secret_name" && "$secret_version" == "$expected_secret_version" ]] || fail 'candidate TypeSafe secret reference does not match the environment file'
    [[ -z "${JEV_MODEL:-}" || "$jev_model" == "$JEV_MODEL" ]] || fail 'candidate Jev model does not match expected model'
    verify_env+=(JEV_VERIFY=1 JEV_VERIFY_EXPECTED_MODEL="$jev_model")
  fi
  [[ "$candidate_url" == https://* ]] || fail 'candidate tag URL missing'
  token="$(gcloud auth print-identity-token --audiences="$service_url")"
  env VERIFY_BEARER="$token" "${verify_env[@]}" python3 "$root/deploy/verify.py" "$candidate_url" || fail 'candidate private-model checks failed'
  rollback() { printf 'private promote: rolling back traffic\n' >&2; gcloud run services update-traffic "$service" --project="$project" --region="$region" "$rollback_args" --quiet || printf 'private promote: TRAFFIC ROLLBACK FAILED\n' >&2; }
  trap 'rc=$?; [[ $rc == 0 ]] || rollback; rm -rf "$state"; exit $rc' EXIT
  gcloud run services update-traffic "$service" --project="$project" --region="$region" --to-revisions="${revision}=100" --quiet
  gcloud run services get-iam-policy "$service" --project="$project" --region="$region" --format=json >"$state/iam-after.json"
  cmp -s "$state/iam-before.json" "$state/iam-after.json" || fail 'service IAM changed'
  gcloud run services describe "$service" --project="$project" --region="$region" --format=json >"$state/after.json"
  python3 - "$state/after.json" "$revision" <<'PY'
import json,sys
s=json.load(open(sys.argv[1])); a=s.get("metadata",{}).get("annotations",{}).get("run.googleapis.com/invoker-iam-disabled")
t={x["revisionName"]:x["percent"] for x in s["status"]["traffic"] if x.get("percent")}
if a not in (None,"false") or t != {sys.argv[2]:100}: raise SystemExit("privacy or traffic mismatch")
PY
  token="$(gcloud auth print-identity-token --audiences="$service_url")"
  env VERIFY_BEARER="$token" "${verify_env[@]}" python3 "$root/deploy/verify.py" "$service_url" || fail 'canonical private-model checks failed'
  printf 'Privately promoted %s; IAM unchanged; authenticated canonical checks passed\n' "$revision"
)
[[ "${BASH_SOURCE[0]}" != "$0" ]] || main "$@"
