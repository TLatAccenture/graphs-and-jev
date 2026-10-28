#!/usr/bin/env bash
# Deploys the public build of graphs-and-jev (Gemini interpretation, no Jev) to Cloud Run in the
# Accenture frictionless sandbox. Builds and deploys as dataandaidemoday by impersonation, because
# personal accounts there can't create builds or services. Runs in Git Bash on Windows or Cloud Shell.
#
#   bash deploy/sandbox/deploy.sh                     build the image for HEAD and deploy it
#   DRY_RUN=1 bash deploy/sandbox/deploy.sh           print the settings and stop
#   INTERPRETER=rules bash deploy/sandbox/deploy.sh   read typed requests with the rules only
#
# The sandbox only allows internal and load-balancer ingress, so browsers reach the service through
# deploy/sandbox/public_access.sh. Override any setting with an environment variable.
set -euo pipefail
cd "$(dirname "$0")/../.."

PROJECT_ID="${PROJECT_ID:-acn-gcp-frictionless-spoke-13}"
REGION="${REGION:-australia-southeast1}"
SERVICE="${SERVICE:-graphs-and-jev}"
REPOSITORY="${REPOSITORY:-graphs-and-jev}"          # Artifact Registry repository, created if missing
# The platform strips project roles from accounts it didn't set up within 30 minutes, so the service
# runs as dataandaidemoday, whose Vertex AI roles the platform keeps, as sdm-demo and the presenter do.
RUNTIME_SA="${RUNTIME_SA:-dataandaidemoday@${PROJECT_ID}.iam.gserviceaccount.com}"
DEPLOY_AS="${DEPLOY_AS-dataandaidemoday@${PROJECT_ID}.iam.gserviceaccount.com}"   # empty = your own account
INGRESS="${INGRESS:-internal-and-cloud-load-balancing}"   # the widest the organisation policy allows
# The project's policy requires VPC egress set to private-ranges-only; these match the other services.
VPC_NETWORK="${VPC_NETWORK-default}"
VPC_SUBNET="${VPC_SUBNET:-subnet-australia-southeast1}"
MIN_INSTANCES="${MIN_INSTANCES:-0}"                 # set 1 before a live presentation to avoid cold starts
MAX_INSTANCES="${MAX_INSTANCES:-2}"
MAKE_PUBLIC="${MAKE_PUBLIC:-yes}"                   # yes = let anyone invoke the service
# gemini = Gemini on Vertex AI maps typed catalogue requests to terms and the rules still decide; any
# Gemini failure falls back to the rules with a visible label. rules = no model calls at all.
INTERPRETER="${INTERPRETER:-gemini}"
GEMINI_MODEL="${GEMINI_MODEL:-gemini-3.8-flash}"
# Gcloud wrapper that works around gcloud.cmd mangling arguments on Windows; shared with sdm-demo.
GCLOUD_ENV="${GCLOUD_ENV:-$HOME/Documents/GitHub/TestCode/sdm-demo/gcloud_env.sh}"

TAG="$(git rev-parse HEAD)"
IMAGE="${REGION}-docker.pkg.dev/${PROJECT_ID}/${REPOSITORY}/app:${TAG}"
say() { printf '\n== %s\n' "$*"; }

[ -z "$(git status --porcelain --untracked-files=no)" ] || { echo "Tracked files differ from HEAD; commit or stash them first."; exit 1; }
if [ -n "${DRY_RUN:-}" ]; then
  printf '%s\n' "project $PROJECT_ID" "region $REGION" "service $SERVICE" "image $IMAGE" \
    "runs as $RUNTIME_SA" "deploys as ${DEPLOY_AS:-your account}" "ingress $INGRESS" "public $MAKE_PUBLIC" "interpreter $INTERPRETER"
  exit 0
fi

source "$GCLOUD_ENV"
deployer() { if [ -n "$DEPLOY_AS" ]; then gcloud --impersonate-service-account "$DEPLOY_AS" "$@"; else gcloud "$@"; fi; }

say "Project $PROJECT_ID, region $REGION${DEPLOY_AS:+, building and deploying as $DEPLOY_AS}"

say "Image repository $REPOSITORY"
if ! deployer artifacts repositories describe "$REPOSITORY" --location "$REGION" >/dev/null 2>&1; then
  deployer artifacts repositories create "$REPOSITORY" --repository-format docker --location "$REGION" \
    --description "graphs-and-jev images"
fi

say "Image for commit ${TAG:0:12}"
# .gcloudignore limits the upload to the Dockerfile, Cargo files, catalogue.json, src and static.
# A rerun for the same commit skips the build.
if deployer artifacts docker images describe "$IMAGE" >/dev/null 2>&1; then
  echo "Already built."
else
  deployer builds submit . --region "$REGION" --tag "$IMAGE" --quiet
fi

say "Deploying $SERVICE"
VPC_ARGS=()
[ -z "$VPC_NETWORK" ] || VPC_ARGS=(--network "$VPC_NETWORK" --subnet "$VPC_SUBNET" --vpc-egress private-ranges-only)
ENV_ARGS=(--clear-env-vars)
if [ "$INTERPRETER" = gemini ]; then
  ENV_ARGS=(--set-env-vars "CATALOGUE_INTERPRETER=gemini,GEMINI_MODEL=$GEMINI_MODEL,GEMINI_LOCATION=global,GEMINI_TIMEOUT_MS=8000,GEMINI_THINKING_LEVEL=low,GOOGLE_CLOUD_PROJECT=$PROJECT_ID")
fi
# The time in the revision name lets a settings-only change redeploy the same commit's image.
deployer run deploy "$SERVICE" --region "$REGION" --image "$IMAGE" \
  --revision-suffix "${TAG:0:7}-$(date -u +%H%M%S)" --service-account "$RUNTIME_SA" --ingress "$INGRESS" "${VPC_ARGS[@]}" \
  --min-instances "$MIN_INSTANCES" --max-instances "$MAX_INSTANCES" --concurrency 20 --cpu 1 --memory 512Mi \
  --timeout 30s --port 8080 "${ENV_ARGS[@]}" --no-allow-unauthenticated \
  --startup-probe "timeoutSeconds=2,periodSeconds=2,failureThreshold=15,httpGet.port=8080,httpGet.path=/api/ready" \
  --liveness-probe "initialDelaySeconds=10,timeoutSeconds=2,periodSeconds=10,failureThreshold=3,httpGet.port=8080,httpGet.path=/api/health" \
  --quiet

say "Access"
INVOKERS="$(gcloud run services get-iam-policy "$SERVICE" --region "$REGION" --format 'value(bindings.members)')"
if [[ "$INVOKERS" == *allUsers* ]]; then
  echo "Anyone may invoke $SERVICE."
elif [ "$MAKE_PUBLIC" = "yes" ] && gcloud run services add-iam-policy-binding "$SERVICE" --region "$REGION" \
       --member allUsers --role roles/run.invoker >/dev/null 2>&1; then
  echo "Anyone may invoke $SERVICE."
elif [ "$MAKE_PUBLIC" = "yes" ]; then
  echo "Not public yet: letting everyone invoke $SERVICE needs Cloud Run Admin, which you don't have"
  echo "here. As a project IAM admin you can grant it to yourself briefly, rerun, then remove it."
fi

say "Done"
echo "Revision: $(gcloud run services describe "$SERVICE" --region "$REGION" --format 'value(status.latestReadyRevisionName)')"
echo "Browsers reach it only through the load balancer: bash deploy/sandbox/public_access.sh"
