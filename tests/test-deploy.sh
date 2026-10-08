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

contains 'set -euo pipefail' "$script"
contains 'ninth-airship-386815' "$env_example"
contains 'australia-southeast1' "$env_example"
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
contains '--service-account="$build_sa_resource"' "$script"
contains 'gcloud artifacts docker images describe' "$script"
# shellcheck disable=SC2016
contains '${IMAGE}@${digest}' "$script"
contains 'gcloud beta run deploy' "$script"
contains '--no-traffic' "$script"
contains '--allow-unauthenticated' "$script"
contains '--min-instances=0' "$script"
contains '--max-instances=2' "$script"
contains '--concurrency=20' "$script"
contains '--cpu=1' "$script"
contains '--memory=512Mi' "$script"
contains '--timeout=30s' "$script"
contains 'httpGet.path=/api/ready' "$script"
contains 'httpGet.path=/api/health' "$script"
# shellcheck disable=SC2016
contains '--service-account="$RUNTIME_SERVICE_ACCOUNT"' "$script"

if rg -n '(:latest|--tag=latest)' "$script"; then
  fail 'deploy script must not use latest'
fi

printf 'deploy static checks passed\n'
