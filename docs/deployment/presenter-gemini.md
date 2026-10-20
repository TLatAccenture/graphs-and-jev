# Presenter build with Gemini

This runbook is for the agent or person who deploys the presenter service. It turns on Gemini for the Environmental Agency demo on a separate, private Cloud Run service. The public service at `graphs-and-jev` does not change.

## What it does

- Gemini on Vertex AI reads each typed Environmental Agency request and maps it to catalogue terms through a response schema that lists only the catalogue's values. The service writes those terms back as a plain request, and the configured decision backend decides on it. Gemini never picks an outcome, a dataset or a number.
- On a Gemini-only presenter, the default `DECISION_BACKEND=catalogue` means deterministic catalogue rules decide. On a combined presenter, `DECISION_BACKEND=jev` means hosted Jev decides. If Gemini fails or finds something outside the catalogue, deterministic interpretation passes the reader's own words to that configured backend and the result says Gemini was unavailable.
- The page says on load that Gemini reads requests and that request text is sent to Vertex AI. The service does not log request text; each log line has an `interpreter` field instead.
- Settings and response shape: [docs/api-contract.md](../api-contract.md), sections "Gemini interpreter settings" and "Domain `catalogue`".

## Rules for this deployment

- Presenter only. `deploy/deploy.sh` refuses `CATALOGUE_INTERPRETER=gemini` on a service that is public, and `deploy/promote.sh` refuses to make such a revision public. Do not work around either.
- No keys in the service. Cloud Run calls Vertex AI as the service's runtime service account, through the metadata server ([Cloud Run container contract](https://cloud.google.com/run/docs/container-contract#metadata-server)). Never copy a JSON key file into the repository, the image, an env file or a chat. `deploy/deploy.sh` refuses `GEMINI_ACCESS_TOKEN`.
- Keep the deploy env file outside the repository.

## One-time setup

Run as the project owner, from the deploying machine. Names below are the defaults this runbook assumes.

```bash
PROJECT_ID=ninth-airship-386815
REGION=australia-southeast1
PRESENTER_SA=graphs-and-jev-presenter@${PROJECT_ID}.iam.gserviceaccount.com

gcloud services enable aiplatform.googleapis.com --project="$PROJECT_ID"
gcloud iam service-accounts create graphs-and-jev-presenter \
  --project="$PROJECT_ID" --display-name="graphs-and-jev presenter runtime"
gcloud projects add-iam-policy-binding "$PROJECT_ID" \
  --member="serviceAccount:${PRESENTER_SA}" --role=roles/aiplatform.user
```

`roles/aiplatform.user` is a broad, workable default and includes `aiplatform.endpoints.predict`, which `generateContent` needs ([Vertex AI access control](https://cloud.google.com/vertex-ai/docs/general/access-control)). `deploy/deploy.sh` checks that permission for the runtime service account before it builds. Give it only to the presenter service account, not to `graphs-and-jev-runner`. Where your organisation supports custom roles, prefer a reviewed minimal role containing only the required Vertex prediction permission(s).

The deploying account also needs `iam.serviceAccounts.actAs` on the new service account; `deploy/deploy.sh` checks it.

If you deploy with another approved identity rather than `anthony.lui@archegon.com`, set `EXPECTED_ACCOUNT` to that identity in the env file. Authenticate interactively or use an approved workload identity; do not create or use a service-account key file for this workflow.

### Check the model before the day

Confirm the model answers at the location you will use ([Vertex AI locations](https://cloud.google.com/vertex-ai/generative-ai/docs/learn/locations)). From the deploying machine:

```bash
curl -sS -X POST \
  -H "Authorization: Bearer $(gcloud auth print-access-token)" \
  -H "Content-Type: application/json" \
  "https://aiplatform.googleapis.com/v1/projects/${PROJECT_ID}/locations/global/publishers/google/models/gemini-3.8-flash:generateContent" \
  -d '{"contents":[{"role":"user","parts":[{"text":"Reply with OK."}]}]}'
```

A reply with `candidates` verifies the configured `gemini-3.8-flash` model at the `global` Vertex endpoint; the service sends thinking level `LOW` by default. Gemini Enterprise Agent Platform entitlement and the public Vertex AI model catalogue are distinct surfaces, so verify against the deployed project and endpoint rather than inferring availability from the other catalogue. A 404 means the model is not offered there: set `GEMINI_LOCATION` to a location that offers it (the service then calls `https://{location}-aiplatform.googleapis.com`), or set `GEMINI_MODEL` to the id that answers.

## Env file

Copy `deploy/cloud-run.env.example` to a private path, for example `~/private/presenter.env`, and uncomment the "Presenter build with Gemini" block:

```bash
SERVICE=graphs-and-jev-presenter
RUNTIME_SERVICE_ACCOUNT=graphs-and-jev-presenter@${PROJECT_ID}.iam.gserviceaccount.com
CATALOGUE_INTERPRETER=gemini
GEMINI_MODEL=gemini-3.8-flash
GEMINI_LOCATION=global
GEMINI_TIMEOUT_MS=8000
GEMINI_THINKING_LEVEL=low
GOOGLE_CLOUD_PROJECT=ninth-airship-386815
MIN_INSTANCES=1
```

Leave `API_MODE_CATALOGUE` unset (live). The "Event build" setting `API_MODE_CATALOGUE=fixtures` would run the demo in the browser, where Gemini is never called.

## Deploy

```bash
deploy/deploy.sh ~/private/presenter.env
```

- First deploy: the script creates `graphs-and-jev-presenter` private, with all traffic on the new revision.
- Later deploys: the new revision gets a tag URL and no traffic. Promote only with the private workflow, which verifies the tagged candidate with an identity token whose audience is the base Cloud Run service URL, requires a real Gemini interpretation and canonical request, preserves IAM/private settings, switches traffic, verifies the canonical URL with a newly minted token whose audience is the base Cloud Run service URL, and rolls back traffic on failure:

```bash
deploy/promote-private.sh REVISION ~/private/presenter.env
```

Never use raw `update-traffic` or the public `deploy/promote.sh` for this service.

## Open it

The service stays private. Open it through an authenticated local proxy ([gcloud run services proxy](https://cloud.google.com/sdk/gcloud/reference/run/services/proxy)):

```bash
gcloud run services proxy graphs-and-jev-presenter \
  --project=ninth-airship-386815 --region=australia-southeast1 --port=8080
```

Then open <http://localhost:8080/environmental-agency/>. The presenter's account needs `roles/run.invoker` on the service; project owners have it.

## Smoke test

With the proxy running:

```bash
curl -sS http://localhost:8080/api/pipeline \
  -H "Content-Type: application/json" \
  -d '{"domain":"catalogue","request":"could I see fine particles for each nation, yearly, 2019 through 2024?","preference":null}' \
  | python3 -c 'import json,sys; b=json.load(sys.stdin); print(b["outcome"], json.dumps(b.get("interpretation")))'
```

Expected: `answer` and `"by": "gemini"` with `"note": null`. `python3 deploy/verify.py http://localhost:8080` then runs the usual release checks through the proxy. On the page, the Scope note names Gemini and the result shows "Read by Gemini (gemini-3.8-flash): …" followed by the catalogue terms.

If `interpretation.by` is `rules`, the note says why:

| Note | Likely cause | Fix |
|---|---|---|
| no Google Cloud access token | Metadata server not reachable | Check the revision runs on Cloud Run with the presenter service account |
| no Google Cloud project | Project id not found | Set `GOOGLE_CLOUD_PROJECT` |
| Vertex AI returned HTTP 403 | API off or role missing | Repeat the one-time setup |
| Vertex AI returned HTTP 404 | Model not offered at the location | Change `GEMINI_LOCATION` or `GEMINI_MODEL` |
| Vertex AI returned HTTP 400 | A setting the model does not accept | Try `GEMINI_THINKING_LEVEL=model-default` |
| Gemini did not respond in time | Slow reply | Raise `GEMINI_TIMEOUT_MS`, staying below 15000 |
| Gemini returned terms outside the catalogue | The reply ignored the schema | Report it; the rules keep answering |

Cloud Run logs show the same per request in the `interpreter` field, for example `"interpreter":"rules_fallback: Vertex AI returned HTTP 404"`.

## On the day

- Keep `MIN_INSTANCES=1` so the first request does not wait for a cold start.
- Run the smoke test 30 minutes before presenting.
- If Gemini fails during the talk, the demo keeps working on the rules and says so on screen. Nothing needs restarting.

## After the day

- Scale down the presenter service by deploying again with `MIN_INSTANCES` removed. Deletion is a separate owner-approved operation.
- Remove `roles/aiplatform.user` (or the custom minimal role) from the presenter service account if nothing else uses it.

## Local dry run (optional)

Your own `gcloud` login is enough; the key file is not needed.

```bash
CATALOGUE_INTERPRETER=gemini \
GOOGLE_CLOUD_PROJECT=ninth-airship-386815 \
GEMINI_ACCESS_TOKEN="$(gcloud auth print-access-token)" \
cargo run
```

Open <http://127.0.0.1:8080/environmental-agency/>. The token expires; rerun the command to refresh it.


## Combined Gemini and hosted Jev presenter

This is a separate private Archegon service. Set `CATALOGUE_INTERPRETER=gemini`, the exact GA `GEMINI_MODEL`, `DECISION_BACKEND=jev`, `SERVICE_AUTH_MODE=cloud-run-iam`, and exact `JEV_MODEL`. Use a dedicated runtime service account with only Vertex prediction, Secret Manager secret-version access for the one TypeSafe secret, and normal logging permissions. Keep ingress private as configured, invoker IAM enabled, and grant `roles/run.invoker` only to approved presenters or IAP identities.

Set `TYPESAFE_SECRET_RESOURCE` to `projects/PROJECT/secrets/SECRET/versions/VERSION`. The project must match `PROJECT_ID`; the version must be an enabled positive integer. `latest` is refused so the release remains immutable. Set `TYPESAFE_EGRESS_ACKNOWLEDGED=yes` to record that requests leave Google Cloud for TypeSafe's hosted API. Never set `TYPESAFE_API_KEY` in the environment file. The deploy preflight reads only secret metadata and IAM policy; it never accesses the payload.

Do not bind the secret before consuming code exists. The hosted Jev backend now consumes it, so deployment may bind it as `TYPESAFE_API_KEY` through Cloud Run `secretKeyRef`. Deploy with `deploy/deploy.sh PRIVATE_ENV`, then promote only with `deploy/promote-private.sh REVISION PRIVATE_ENV`. Private promotion authenticates both candidate and canonical smoke tests, verifies whichever private model modes the revision enables (Gemini, Jev or both), preserves IAM, and restores prior traffic on failure. Public promotion refuses either hosted Jev or Cloud Run IAM service mode.
