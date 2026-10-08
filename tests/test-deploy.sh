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
contains 'IMAGE=app' "$env_example"
contains '--default-buckets-behavior=regional-user-owned-bucket' "$script"
contains 'gcloud beta billing projects describe' "$script"
contains 'gcloud services list' "$script"
contains 'gcloud artifacts repositories describe' "$script"
contains 'gcloud iam service-accounts describe' "$script"
contains 'troubleshoot-policy iam' "$script"
contains 'iam.serviceAccounts.actAs' "$script"
contains 'logging.logEntries.create' "$script"
contains 'serviceusage.services.use' "$script"
contains 'artifactregistry.repositories.uploadArtifacts' "$script"
contains "value(overallAccessState)" "$script"
contains 'cloudbilling.googleapis.com' "$script"
contains 'logging.googleapis.com' "$script"
contains 'gcloud run revisions describe' "$script"
contains 'gcloud run services get-iam-policy' "$script"
contains 'EFFECTIVE_SETTINGS_JSON' "$script"
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
contains '--service-account="$build_sa_resource"' "$script"
contains 'gcloud artifacts docker images describe' "$script"
# shellcheck disable=SC2016
contains '${IMAGE}@${digest}' "$script"
contains 'assert_effective_deployment' "$script"
contains 'gcloud beta run deploy' "$script"
contains '--no-traffic' "$script"
contains '--allow-unauthenticated' "$script"
# shellcheck disable=SC2016
contains 'MIN_INSTANCES="${MIN_INSTANCES:-0}"' "$script"
# shellcheck disable=SC2016
contains 'MAX_INSTANCES="${MAX_INSTANCES:-2}"' "$script"
# shellcheck disable=SC2016
contains 'CONCURRENCY="${CONCURRENCY:-20}"' "$script"
# shellcheck disable=SC2016
contains 'CPU="${CPU:-1}"' "$script"
# shellcheck disable=SC2016
contains 'MEMORY="${MEMORY:-512Mi}"' "$script"
# shellcheck disable=SC2016
contains 'TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-30}"' "$script"
# shellcheck disable=SC2016
contains 'STARTUP_PATH="${STARTUP_PATH:-/api/ready}"' "$script"
# shellcheck disable=SC2016
contains 'LIVENESS_PATH="${LIVENESS_PATH:-/api/health}"' "$script"
# shellcheck disable=SC2016
contains '--service-account="$RUNTIME_SERVICE_ACCOUNT"' "$script"

if rg -n '(:latest|--tag=latest)' "$script"; then
  fail 'deploy script must not use latest'
fi

printf 'deploy static checks passed\n'
