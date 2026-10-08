<div align="center">

# Graphs + Jev

### Turn connected evidence into bounded decisions.

[**Open the interactive field guide →**](https://tlataccenture.github.io/graphs-and-jev/)

<img src="assets/evidence-path.svg" width="560" alt="A question moves through connected evidence to a bounded Jev decision, with the evidence path returned alongside the answer.">

</div>

Graphs preserve the relationships between facts. Jev decides, from that evidence, whether an agent may answer, must ask, must refuse, or must hand the question to a person.

## Explore the publication

- [How graphs and Jev work together](https://tlataccenture.github.io/graphs-and-jev/#how-it-works)
- [DCCEEW catalogue discovery](https://tlataccenture.github.io/graphs-and-jev/dcceew/)
- [Parkinson’s research evidence](https://tlataccenture.github.io/graphs-and-jev/parkinsons/)
- [Little Bunnies: why two hops are usually enough](https://tlataccenture.github.io/graphs-and-jev/little-bunnies/)
- [Graph-engine guide and browser labs](https://tlataccenture.github.io/graphs-and-jev/#technical-guide)

The demonstrations run entirely in the browser with deterministic representative data. No backend, hosted model, analytics, cookies, credentials, or live research search is required.

## About

This is a personal open-source research and demonstration project by TLatAccenture. It is not an official Accenture or Google Cloud product. Product names and marks belong to their respective owners.

## Run locally

Because the site uses the GitHub Pages project path, serve its parent directory:

```bash
cd ..
python3 -m http.server 8000
```

Open <http://localhost:8000/graphs-and-jev/>.

### Run the Rust container

Build the same Linux architecture used by Cloud Run, then smoke-test it on a random local port:

```bash
docker build --platform linux/amd64 -t graphs-and-jev:test .
./scripts/smoke-container.sh
```

To run it directly:

```bash
docker run --rm -p 8080:8080 -e PORT=8080 graphs-and-jev:test
```

The scratch image runs as non-root UID 65532 and contains only the static service binary, catalogue, and field-guide assets. The smoke script verifies the static x86-64 ELF, exact scratch rootfs allowlist, image UID, and every API route, including a non-clinical live LRRK2 research request. `CATALOGUE_PATH` and `STATIC_DIR` default to the packaged catalogue and field guide; override them only when mounting replacements. Cloud Run should configure external startup and liveness probes for `/api/ready` and `/api/health`; the image deliberately contains no HTTP client, cloud SDK, or credentials.

## Reuse

No reuse licence is granted yet. All rights are reserved unless a licence is added later.
