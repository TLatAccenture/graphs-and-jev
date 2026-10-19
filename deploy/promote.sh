#!/usr/bin/env bash
# Promotes a candidate from deploy/deploy.sh: verify it, move all traffic to it, make the service
# public, verify the canonical URL. Any failure after the first change restores the prior traffic
# split and IAM policy.
#
# usage: deploy/promote.sh REVISION [environment-file]

promote_traffic_args() {
  # Prints --to-revisions=REV=PCT,... for the revisions that served traffic before promotion.
  python3 -c '
import json, sys
traffic = [t for t in json.load(open(sys.argv[1]))["status"].get("traffic", []) if t.get("percent")]
if not traffic or any("revisionName" not in t for t in traffic):
    raise SystemExit("prior traffic must name revisions")
print("--to-revisions=" + ",".join("%s=%s" % (t["revisionName"], t["percent"]) for t in traffic))
' "$1"
}

main() (
  set -euo pipefail
  local gjp_REPO_ROOT gjp_PROJECT_ID gjp_REGION gjp_SERVICE gjp_EXPECTED_ACCOUNT gjp_revision
  local gjp_state gjp_service_json gjp_iam_json gjp_rollback_args gjp_candidate_url gjp_service_url gjp_ready gjp_interpreter

  gjp_REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
  if [[ $# -lt 1 || $# -gt 2 ]]; then
    printf 'usage: %s REVISION [environment-file]\n' "$0" >&2
    exit 64
  fi
  gjp_revision="$1"
  if [[ $# -eq 2 ]]; then
    # shellcheck disable=SC1090
    source "$2"
  fi
  gjp_PROJECT_ID="${PROJECT_ID:-ninth-airship-386815}"
  gjp_REGION="${REGION:-australia-southeast1}"
  gjp_SERVICE="${SERVICE:-graphs-and-jev}"
  gjp_EXPECTED_ACCOUNT="${EXPECTED_ACCOUNT:-anthony.lui@archegon.com}"
  local -a gjp_where=(--project="$gjp_PROJECT_ID" --region="$gjp_REGION")

  fail() { printf 'promote failed: %s\n' "$*" >&2; exit 1; }
  [[ "$gjp_revision" =~ ^${gjp_SERVICE}-[a-z0-9-]+$ ]] || fail "revision must be a ${gjp_SERVICE}-* revision name"
  [[ "$(gcloud auth list --filter=status:ACTIVE --format='value(account)')" == "$gjp_EXPECTED_ACCOUNT" ]] || fail "active account must be $gjp_EXPECTED_ACCOUNT"
  [[ "$(gcloud config get-value project 2>/dev/null)" == "$gjp_PROJECT_ID" ]] || fail "active project must be $gjp_PROJECT_ID"

  gjp_state="$(mktemp -d)"
  trap 'rm -rf "$gjp_state"' EXIT
  gjp_service_json="$gjp_state/service.json"
  gjp_iam_json="$gjp_state/iam.json"
  gcloud run services describe "$gjp_SERVICE" "${gjp_where[@]}" --format=json >"$gjp_service_json"
  gcloud run services get-iam-policy "$gjp_SERVICE" "${gjp_where[@]}" --format=json \
    | python3 -c 'import json,sys; policy=json.load(sys.stdin); policy.pop("etag", None); json.dump(policy, sys.stdout)' >"$gjp_iam_json"
  gjp_rollback_args="$(promote_traffic_args "$gjp_service_json")" || fail 'could not record prior traffic'
  gjp_service_url="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["status"]["url"])' "$gjp_service_json")"
  gjp_candidate_url="$(python3 -c '
import json, sys
tagged = [t for t in json.load(open(sys.argv[1]))["status"].get("traffic", []) if t.get("revisionName") == sys.argv[2] and t.get("tag")]
print(tagged[0]["url"] if tagged else "")
' "$gjp_service_json" "$gjp_revision")"
  [[ "$gjp_candidate_url" == https://* ]] || fail "$gjp_revision has no candidate tag URL"
  gjp_ready="$(gcloud run revisions describe "$gjp_revision" "${gjp_where[@]}" --format='value(status.conditions[0].status)')"
  [[ "$gjp_ready" == True ]] || fail "$gjp_revision is not ready"
  # A revision where Gemini reads typed requests is presenter-only and is never made public.
  gjp_interpreter="$(gcloud run revisions describe "$gjp_revision" "${gjp_where[@]}" --format=json \
    | python3 -c 'import json,sys; env={e["name"]: e.get("value", "") for e in json.load(sys.stdin)["spec"]["containers"][0].get("env", [])}; print(env.get("CATALOGUE_INTERPRETER") or "rules")')" \
    || fail "could not read the settings of $gjp_revision"
  [[ "$gjp_interpreter" != gemini ]] || fail "$gjp_revision has CATALOGUE_INTERPRETER=gemini; it is presenter-only and is never made public"

  VERIFY_BEARER="$(gcloud auth print-identity-token)" python3 "$gjp_REPO_ROOT/deploy/verify.py" "$gjp_candidate_url" || fail 'candidate checks failed'

  rollback() {
    printf 'promote: rolling back traffic and IAM\n' >&2
    gcloud run services update-traffic "$gjp_SERVICE" "${gjp_where[@]}" "$gjp_rollback_args" --quiet || printf 'promote: TRAFFIC ROLLBACK FAILED: run update-traffic %s\n' "$gjp_rollback_args" >&2
    gcloud run services set-iam-policy "$gjp_SERVICE" "$gjp_iam_json" "${gjp_where[@]}" --quiet >/dev/null || printf 'promote: IAM ROLLBACK FAILED\n' >&2
  }
  trap 'status=$?; [[ $status == 0 ]] || rollback; rm -rf "$gjp_state"; exit $status' EXIT

  gcloud run services update-traffic "$gjp_SERVICE" "${gjp_where[@]}" --to-revisions="${gjp_revision}=100" --quiet
  gcloud run services add-iam-policy-binding "$gjp_SERVICE" "${gjp_where[@]}" \
    --member=allUsers --role=roles/run.invoker --quiet >/dev/null
  gcloud run services describe "$gjp_SERVICE" "${gjp_where[@]}" --format=json >"$gjp_state/after.json"
  python3 -c '
import json, sys
serving = {t["revisionName"]: t["percent"] for t in json.load(open(sys.argv[1]))["status"]["traffic"] if t.get("percent")}
if serving != {sys.argv[2]: 100}:
    raise SystemExit(f"traffic is {serving}, expected all on {sys.argv[2]}")
' "$gjp_state/after.json" "$gjp_revision" || fail 'traffic did not move to the candidate'
  VERIFY_BEARER='' python3 "$gjp_REPO_ROOT/deploy/verify.py" "$gjp_service_url" || fail 'canonical URL checks failed'
  printf 'Promoted %s: 100%% traffic, public, canonical URL verified at %s\n' "$gjp_revision" "$gjp_service_url"
)

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  main "$@"
fi
