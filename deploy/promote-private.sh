#!/usr/bin/env bash
# Promote a private Gemini candidate without changing service IAM or IAP settings.
main() (
  set -euo pipefail
  local root project region service account revision state candidate_url service_url token rollback_args model
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
  model="$(gcloud run revisions describe "$revision" --project="$project" --region="$region" --format=json | python3 -c 'import json,sys; e={x["name"]:x.get("value","") for x in json.load(sys.stdin)["spec"]["containers"][0].get("env",[])}; assert e.get("CATALOGUE_INTERPRETER")=="gemini"; print(e.get("GEMINI_MODEL") or "gemini-3.8-flash")')" || fail 'candidate is not a Gemini revision'
  [[ "$candidate_url" == https://* ]] || fail 'candidate tag URL missing'
  token="$(gcloud auth print-identity-token --audiences="$service_url")"
  VERIFY_BEARER="$token" GEMINI_VERIFY=1 VERIFY_EXPECTED_MODEL="$model" python3 "$root/deploy/verify.py" "$candidate_url" || fail 'candidate Gemini checks failed'
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
  VERIFY_BEARER="$token" GEMINI_VERIFY=1 VERIFY_EXPECTED_MODEL="$model" python3 "$root/deploy/verify.py" "$service_url" || fail 'canonical Gemini checks failed'
  printf 'Privately promoted %s; IAM unchanged; authenticated canonical checks passed\n' "$revision"
)
[[ "${BASH_SOURCE[0]}" != "$0" ]] || main "$@"
