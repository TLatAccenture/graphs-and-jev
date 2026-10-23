#!/usr/bin/env bash
# Serves graphs-and-jev publicly through the sdm-demo HTTPS load balancer in the Accenture
# frictionless sandbox, on its own hostname. Reusing that load balancer's IP, proxy and forwarding
# rule avoids a second ~US$18 a month forwarding rule. Adds a Cloud Armor policy that rate-limits
# each visitor's API requests. Safe to re-run. Run after deploy/sandbox/deploy.sh.
#
#   bash deploy/sandbox/public_access.sh
#   DOMAIN=jev.example.com bash deploy/sandbox/public_access.sh   use your own domain, pointed at the IP
set -euo pipefail
cd "$(dirname "$0")"

PROJECT_ID="${PROJECT_ID:-acn-gcp-frictionless-spoke-13}"
REGION="${REGION:-australia-southeast1}"
SERVICE="${SERVICE:-graphs-and-jev}"
LB="${LB:-sdm-demo}"                                 # prefix of the existing load balancer's resources
DEPLOY_AS="${DEPLOY_AS-dataandaidemoday@${PROJECT_ID}.iam.gserviceaccount.com}"
API_LIMIT_PER_MINUTE="${API_LIMIT_PER_MINUTE:-120}"  # per client IP, enforced by Cloud Armor at the edge
PIPELINE_LIMIT_PER_MINUTE="${PIPELINE_LIMIT_PER_MINUTE:-20}"  # /api/pipeline can call Gemini, so it gets less
GCLOUD_ENV="${GCLOUD_ENV:-$HOME/Documents/GitHub/TestCode/sdm-demo/gcloud_env.sh}"
N="$SERVICE"

source "$GCLOUD_ENV"
deployer() { if [ -n "$DEPLOY_AS" ]; then gcloud --impersonate-service-account "$DEPLOY_AS" "$@"; else gcloud "$@"; fi; }
exists() { deployer "$@" >/dev/null 2>&1; }   # takes a describe command
say() { printf '\n== %s\n' "$*"; }

IP="$(deployer compute addresses describe "$LB-ip" --global --format 'value(address)')"
DOMAIN="${DOMAIN:-${SERVICE}-${IP//./-}.sslip.io}"
say "Serving $SERVICE at https://$DOMAIN ($IP, shared with $LB)"

say "Serverless endpoint group for $SERVICE"
exists compute network-endpoint-groups describe "$N-neg" --region "$REGION" ||
  deployer compute network-endpoint-groups create "$N-neg" --region "$REGION" \
    --network-endpoint-type serverless --cloud-run-service "$SERVICE"

say "Cloud Armor: at most $API_LIMIT_PER_MINUTE API requests a minute per visitor"
exists compute security-policies describe "$N-armor" ||
  deployer compute security-policies create "$N-armor" --description "Rate limits for graphs-and-jev"
exists compute security-policies rules describe 1000 --security-policy "$N-armor" ||
  deployer compute security-policies rules create 1000 --security-policy "$N-armor" \
    --expression "request.path.startsWith('/api/')" --action throttle \
    --rate-limit-threshold-count "$API_LIMIT_PER_MINUTE" --rate-limit-threshold-interval-sec 60 \
    --conform-action allow --exceed-action deny-429 --enforce-on-key IP

say "Cloud Armor: at most $PIPELINE_LIMIT_PER_MINUTE pipeline requests a minute per visitor"
exists compute security-policies rules describe 900 --security-policy "$N-armor" ||
  deployer compute security-policies rules create 900 --security-policy "$N-armor" \
    --expression "request.path.startsWith('/api/pipeline')" --action throttle \
    --rate-limit-threshold-count "$PIPELINE_LIMIT_PER_MINUTE" --rate-limit-threshold-interval-sec 60 \
    --conform-action allow --exceed-action deny-429 --enforce-on-key IP

say "Backend service, with request logging"
exists compute backend-services describe "$N-backend" --global ||
  deployer compute backend-services create "$N-backend" --global --load-balancing-scheme EXTERNAL_MANAGED \
    --enable-logging --logging-sample-rate 1
BACKENDS="$(deployer compute backend-services describe "$N-backend" --global --format 'value(backends[].group)')"
if [[ "$BACKENDS" != *"/$N-neg"* ]]; then
  deployer compute backend-services add-backend "$N-backend" --global \
    --network-endpoint-group "$N-neg" --network-endpoint-group-region "$REGION"
fi
deployer compute backend-services update "$N-backend" --global --security-policy "$N-armor" --quiet

say "Host rule on $LB-lb"
MATCHERS="$(deployer compute url-maps describe "$LB-lb" --global --format 'value(pathMatchers[].name)')"
if [[ ";$MATCHERS;" != *";$N;"* ]]; then
  deployer compute url-maps add-path-matcher "$LB-lb" --global --path-matcher-name "$N" \
    --default-service "$N-backend" --new-hosts "$DOMAIN"
fi

say "Certificate for $DOMAIN on $LB-https-proxy"
CERT="$N-cert-$(tr '.' '-' <<<"$DOMAIN" | cut -c1-40)"
exists compute ssl-certificates describe "$CERT" --global ||
  deployer compute ssl-certificates create "$CERT" --global --domains "$DOMAIN"
CERTS="$(deployer compute target-https-proxies describe "$LB-https-proxy" --global --format 'value(sslCertificates.basename())')"
[[ ";$CERTS;" == *";$CERT;"* ]] ||
  deployer compute target-https-proxies update "$LB-https-proxy" --global --ssl-certificates "${CERTS//;/,},$CERT"

say "Done"
echo "Site: https://$DOMAIN"
printf 'Certificate: '
deployer compute ssl-certificates describe "$CERT" --global --format 'value(managed.status)'
echo "A new certificate takes 15-60 minutes to become ACTIVE; until then browsers show a security error."
