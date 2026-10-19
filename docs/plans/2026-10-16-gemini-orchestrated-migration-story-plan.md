# Gemini-orchestrated migration story implementation plan

**Status:** Plan only
**Repository:** `TLatAccenture/graphs-and-jev`
**Branch:** `gcp-acceleration-story`
**Baseline:** `295bed5`
**Scope:** Front-end-first story and documentation change. No service integration or deployment.

## 1. Outcome and diagnosis

Refactor the landing story so a reader immediately understands that a planned **Gemini Migration Agent**, built with Agent Development Kit (ADK), orchestrates migration work across Google's official **Assess, Plan, Deploy, Optimize** framework. The page must also make representative source-to-destination paths explicit, while preserving the exact boundary between the live deterministic demonstration and the planned reference architecture.

The current implementation has two structural problems:

- The `#google-cloud-architecture` rail is a five-stage product flow. ADK with Gemini appears only at stage 4, after source, connection and operational-state stages. This makes Gemini read as one product component rather than the orchestrator spanning the migration lifecycle.
- The source-estate list names PySpark, Hadoop, SAP, Databricks, Salesforce, S3, Azure Blob and Microsoft 365, but it does not consistently state the data or workload type, movement method, Google Cloud destination and validation evidence.

Do not weaken the existing factual boundary:

- **Live today:** one Rust process serves the static publication and same-origin API on Cloud Run. The current demonstrations use deterministic rules and browser or API fixtures according to deployment configuration.
- **Planned reference architecture:** Gemini, ADK, MCP servers, RAG Engine, AlloyDB AI and the other migration products described below are not wired into the demonstration.
- Preserve the exact sentence: **“The Google Cloud reference architecture is planned and not live in this demo.”**
- Keep every existing demonstration illustrative and deterministic. Do not imply live Gemini, ADK, MCP, RAG Engine, AlloyDB AI or migration-service execution.

Primary framing sources:

- [Google Cloud migration hub](https://docs.cloud.google.com/docs/migration)
- [Migration to Google Cloud: getting started](https://docs.cloud.google.com/architecture/migration-to-gcp-getting-started)
- [Assessing and discovering workloads](https://docs.cloud.google.com/architecture/migration-to-gcp-assessing-and-discovering-your-workloads)
- [Transferring large datasets](https://docs.cloud.google.com/architecture/migration-to-google-cloud-transferring-your-large-datasets)

## 2. Recommended story hierarchy

Use this hierarchy on the landing page:

1. **Hero:** lead with “Gemini Migration Agent” or “Gemini Agent orchestrates the migration”. Exact copy may be refined for clarity and line length, but the visible hero text and visual must both name the Gemini Agent. State that it is built with ADK and belongs to the planned reference architecture.
2. **Live/reference ribbon:** retain it immediately after the hero. Keep both state labels and the exact planned/not-live disclaimer.
3. **Agent-centred migration control plane:** replace or substantially refactor the current `#google-cloud-architecture` product rail. Show one persistent Gemini Migration Agent orchestration band across Assess, Plan, Deploy and Optimize.
4. **Representative migration lanes:** make source, artefact or workload, movement service, destination and validation explicit. Keep these lanes within the refactored architecture section rather than adding a second giant architecture section.
5. **Five pillars:** retain exactly five. Reframe them as the capabilities the Gemini Migration Agent uses, not as the page's primary narrative.
6. **Guided example and use cases:** retain existing routes and mechanics. Add compact orchestration context without pretending the deterministic demo calls the planned services.
7. **Where to start and technical guide:** align terminology with the four official phases and define the source-to-destination paths narrowly.

Use this responsibility split throughout:

- The Gemini Migration Agent coordinates work, reasons over evidence, proposes plans and invokes governed tools.
- The agent does not copy bytes, parse exact formats, count rows or bypass approvals.
- The MCP Tool Shed provides governed tools and resources. MCP is a protocol and tool boundary, not a parser.
- Jev/System 1 makes bounded classifications and gates routine decisions at every phase. Low-confidence or ambiguous outcomes go to a person.
- RAG Engine retrieves document context where suitable.
- Spanner Graph provides lineage, dependency and semantic context.
- Deterministic migration services and code or SQL perform movement, exact parsing, row counts, schema validation and invariant checks.
- BigQuery and Looker hold and report analytical telemetry and evaluations in the planned reference architecture.

## 3. Four-phase migration lifecycle

Build one semantic lifecycle component using the official phase order from [Migration to Google Cloud: getting started](https://docs.cloud.google.com/architecture/migration-to-gcp-getting-started). Each phase card must expose the same labelled fields: **Inputs**, **Agent action**, **Jev gate**, **Output**, and **State**. Every card must say **Planned reference** or an equivalently explicit state label.

### Assess

**Copy intent:** establish what exists, what depends on what and what evidence is missing before a destination is selected.

- **Inputs:** workload and data inventory, source types, owners, dependencies, security constraints, current service levels, cost and utilisation evidence.
- **Agent action:** Gemini summarises discovered evidence and gaps. Governed MCP tools inspect authorised sources. Mention [Migration Center](https://cloud.google.com/migration-center/docs/migration-center-overview) only as a sourced discovery and assessment capability, not as a universal inventory source.
- **Jev gate:** classify, route and escalate inventory findings. Ambiguous ownership, sensitivity or dependency evidence cannot silently proceed.
- **Graph role:** Spanner Graph records dependencies and lineage in the planned reference design.
- **Output:** assessed inventory, dependency map, baseline and TCO inputs, and an evidence-gap list.

### Plan

**Copy intent:** turn assessed evidence into an approved migration plan for each workload or data set.

- **Inputs:** assessed inventory, dependency graph, baseline, constraints and acceptance requirements.
- **Agent action:** Gemini proposes the migration strategy, target destination, sequence, security, network and IAM requirements, cutover, rollback and acceptance criteria.
- **Jev gate:** classify bounded choices, flag ambiguity and stop low-confidence recommendations from becoming actions.
- **Human checkpoint:** show an explicit approval between Plan and Deploy. No visual path may bypass it.
- **Output:** approved source-to-destination plan, selected deterministic services, run order, rollback plan and acceptance tests.

### Deploy

**Copy intent:** distinguish orchestration from execution.

- **Inputs:** approved plan, authorised service configuration, runbook, rollback conditions and validation criteria.
- **Agent action:** Gemini coordinates jobs, tool calls, status and evidence. It does not move bytes.
- **Execution:** deterministic migration services move data or workloads. Deterministic code and SQL parse exact formats, count rows, validate schemas and compare invariants.
- **Jev gate:** gate routine state transitions and escalate policy failures, evidence gaps and low-confidence classifications.
- **Output:** migration job evidence, validation report and either a cutover-ready state or rollback instruction. Show **validate before cutover** and **rollback if validation fails**.

### Optimize

**Copy intent:** compare the migrated outcome with the baseline and decide what to improve next.

- **Inputs:** baseline, migration evidence, service quality, cost and performance telemetry, evaluation outcomes and user feedback.
- **Agent action:** Gemini interprets the evidence, identifies exceptions and proposes further modernisation.
- **Jev gate:** classify routine outcomes, route anomalies and escalate uncertain recommendations.
- **Observability role:** BigQuery holds analytical telemetry and eval records; Looker reports quality, cost and performance in the planned reference architecture.
- **Output:** outcome comparison, optimisation backlog, accepted recommendations and evidence for the next cycle.

## 4. Representative source-to-destination migration lanes

Within the refactored architecture section, add four clearly labelled **representative reference paths**. They are examples, not automatic recommendations. On desktop, use readable text and directional connectors. On mobile and print, preserve semantic order by stacking each complete lane before the next.

Every lane must visibly provide **Source**, **What moves**, **Movement/service**, **Destination**, and **Validation/evidence**.

Add a prominent note: **Destination is chosen by workload. Not all data belongs in BigQuery.**

### Lane 1: Unstructured documents

- **Source:** Microsoft 365, Amazon S3 and Azure Blob Storage.
- **What moves:** PDFs, Word documents, PowerPoint presentations and images.
- **Movement/service:** use an appropriately qualified object or file transfer path, including [Storage Transfer Service](https://cloud.google.com/storage-transfer/docs/overview) where its documented source support fits. Do not imply MCP copies the objects.
- **Destination:** raw source objects land in Cloud Storage. Document AI or RAG Engine may parse and retrieve document content where suitable. Extracted analytical outputs may optionally go to BigQuery.
- **Validation/evidence:** deterministic checks verify object counts, checksums or service manifests and required metadata. Gemini interprets the report.
- **Required distinction:** label the raw landing zone, retrieval corpus and optional analytical output separately. Do not merge them into one destination box.

### Lane 2: Spreadsheets and tabular files

- **Source:** Excel and CSV files in Microsoft 365, Amazon S3 and Azure Blob Storage.
- **What moves:** files, sheets, tables, schemas and controlled metadata.
- **Movement/service:** transfer source files to a Cloud Storage landing zone using a documented transfer path; deterministic loaders and SQL create validated tables.
- **Destination:** Cloud Storage landing, followed by validated BigQuery tables where analytics is the selected target.
- **Validation/evidence:** deterministic code or SQL counts rows, validates schemas, checks rejects and reconciles totals. Gemini interprets the generated report. RAG may help with document-like content but is not the row counter.

### Lane 3: Operational relational data

- **Source:** PostgreSQL, Oracle, SQL Server and SAP-backed operational data.
- **What moves:** operational tables, change streams and required schema or application state, according to the selected migration strategy.
- **Movement/service:** use [Database Migration Service](https://cloud.google.com/database-migration/docs) or [Datastream](https://cloud.google.com/datastream/docs/overview) only where their primary documentation supports the precise source and target. A partner connector may be appropriate for a specific SAP path. Do not claim one universal SAP migration route.
- **Destination:** AlloyDB for PostgreSQL for compatible operational relational state; BigQuery for analytical use cases. The destination depends on workload needs.
- **Validation/evidence:** deterministic schema compatibility, row reconciliation, change-lag, integrity and application acceptance checks.
- **Boundary:** AlloyDB AI may call TypeSafe's hosted Jev API through the already sourced external endpoint pattern. Jev is not stored or hosted in AlloyDB.

### Lane 4: Applications and jobs

- **Source:** virtual machines, Spark and Hadoop workloads, and containerisable applications.
- **What moves:** machine workloads, batch or data-processing jobs, application binaries, configuration and deployment definitions. Keep this lane visibly distinct from data movement.
- **Movement/service:** cite [Migrate to Virtual Machines](https://cloud.google.com/migrate/virtual-machines/docs/5.0/discover/overview) and [Migrate to Containers](https://cloud.google.com/migrate/containers/docs) only for their documented workload paths. Use workload-specific deployment and migration tooling for Spark or Hadoop jobs.
- **Destination:** Compute Engine for suitable virtual-machine workloads; GKE or Dataproc for suitable container or data-processing workloads; Cloud Run for suitable stateless containerised applications.
- **Validation/evidence:** deterministic health, behaviour, data, performance and rollback checks against acceptance criteria. Gemini coordinates and interprets evidence but does not migrate the machine or job itself.

Before implementation, verify every supported-source or target statement against the linked primary documentation. If a product page does not establish a claimed source path, narrow the copy rather than filling the gap with inference.

## 5. Orchestrator visual specification

Replace the linear peer-product composition with a control-plane composition:

- Use four phase columns in semantic order: Assess, Plan, Deploy, Optimize.
- Place one persistent horizontal orchestration spine or band across all four columns, labelled **Gemini Migration Agent** with **Built with ADK** adjacent.
- Use the existing official local Gemini mark. Keep readable text beside it. Do not make recognition depend on the mark.
- Show governed tool calls descending from the orchestrator into phase activities and migration lanes. Show evidence and status returning upward.
- Show the MCP Tool Shed, RAG Engine, Spanner Graph and deterministic services under the phase or lane where they are used. Do not portray all product logos as peers flowing through one linear pipeline.
- Repeat Jev gate markers in all four phases. Jev must not appear as a one-off stage.
- Put a labelled human approval checkpoint between Plan and Deploy. Also show human review as the destination for low-confidence Jev outcomes.
- Keep compute, data, retrieval and observability products under the phase or lane that uses them.
- Use status text in addition to colour. Decorative connectors must be hidden from assistive technology.
- Preserve DOM reading order without CSS. Ensure the composition remains understandable in print with background graphics disabled.
- Prefer HTML and CSS for essential content. If `static/assets/hero-migration.svg` is revised or an additional orchestration illustration is added, keep all essential wording in adjacent HTML or provide an equivalent complete text alternative.

## 6. Changes by current page section

### Header and brand

Keep **Migration Acceleration with Google Cloud**. No default brand, server or configuration change is expected.

### Hero

Replace the current outcome-only heading and source-to-product illustration with a Gemini-orchestrator-led message. The business outcome remains first, but **Gemini Migration Agent** must be the most visually prominent system role and **built with ADK** must be visible. Qualify the role as planned and preserve the live deterministic statement.

### Live/reference ribbon

Retain the existing structure, labels and exact disclaimer. Do not move it below a large architecture section.

### Why now

Retain the existing argument and compact numbered notes. Add the Google migration framework source only where the section makes a lifecycle claim. Do not overload this section with the source-to-destination register.

### `#google-cloud-architecture`

Substantially refactor the current five-stage product rail into the four-phase agent control plane and representative migration lanes described above. Preserve the existing section ID if practical to avoid breaking inbound links. Do not add a duplicate full-width architecture section.

### Five pillars

Keep exactly these five controls and their order:

1. Existing Stack
2. MCP Tool Shed
3. System 1 (Jev)
4. ADK with Gemini
5. Modernisation Springboard

Change the heading and introductory copy so these are **what the Gemini Migration Agent uses**. Preserve disclosure behaviour and technical-guide links. Explain repeated Jev gating without creating a sixth pillar.

### Guided example

Add a compact, static **orchestrator view** above the existing result. It should expose phase, source, target, selected tools, Jev outcome and human checkpoint for the selected illustrative scenario. Populate it from the existing scenario data or a minimal adjacent mapping. Do not change outcome mechanics, call a model, add network requests or imply live Gemini.

### Use cases

For each existing use-case card, add concise labelled facts for **Source**, **What moves**, **Destination**, **Gemini Agent role**, and **Jev gate**. Preserve links, fictional labels, routes and demonstration mechanics. Mark the Gemini role as planned reference and the existing demo as illustrative or deterministic.

### Where to start

Prefer four concise steps matching Assess, Plan, Deploy and Optimize. If retaining three visual steps, map them unambiguously to Assess, Plan/Deploy and Optimize. The four-step version is clearer and should be attempted first within the current layout.

### Technical guide

Make a narrow update only. Add an **Agent orchestration** subsection and definitions for source, what moves, movement service, destination and validation evidence. Reuse existing primary product sources and add the migration framework sources. State that this is a planned reference architecture and makes no runtime implementation claim. Preserve labs, engine grades and Spanner Graph evidence.

### Repository docs

Align `README.md` and `docs/design.md` with the new story hierarchy and exact live/reference boundary. Do not rewrite historical plans.

## 7. Source and factual rules

The implementation must:

- link the [migration hub](https://docs.cloud.google.com/docs/migration) and [four-phase guide](https://docs.cloud.google.com/architecture/migration-to-gcp-getting-started);
- use the [assessment and discovery guide](https://docs.cloud.google.com/architecture/migration-to-gcp-assessing-and-discovering-your-workloads) and [large-dataset transfer guide](https://docs.cloud.google.com/architecture/migration-to-google-cloud-transferring-your-large-datasets) for the relevant lifecycle and transfer claims;
- mention Migration Center only for sourced discovery and assessment capabilities;
- mention Storage Transfer Service only where its documented source and transfer support fits;
- mention Migrate to Virtual Machines, Migrate to Containers, Database Migration Service and Datastream only with primary links and precise supported-source wording;
- avoid universal SAP connector or migration-path claims;
- preserve the existing ADK, MCP, RAG Engine, AlloyDB/Jev, Cloud Run and icon source register;
- keep AlloyDB-to-Jev wording as an external call to TypeSafe's hosted Jev API, never “Jev runs on AlloyDB”;
- keep MCP as protocol and governed tool access, never exact parser or byte mover;
- assign exact counts, schema checks and validation to deterministic code, SQL or services;
- avoid any claim that the reference services are wired to the live demonstration; and
- link every numeric or third-party capability claim at the point of use or in a directly associated source list.

Use British English, sentence-case headings, plain factual prose and no em dash.

## 8. Expected implementation files

Keep the implementation narrow. Likely files are:

- `static/index.html`
- `static/styles/landing.css`
- `static/assets/hero-migration.svg`, or one replacement or additional local orchestration illustration
- `tests/static_frontend.rs`
- `tests/e2e/landing.spec.ts`
- `tests/e2e/links.spec.ts` only if source-link contracts need expansion
- `static/technical-guide/index.html`
- `static/styles/tech.css` only for the narrow guide addition
- `README.md`
- `docs/design.md`
- `docs/asset-provenance.md` only if a new official asset is added

No server, API, fixture, route or deployment configuration change should be needed. Do not change `src/site.rs` or `static/config.js` unless the default brand copy changes, which is not expected.

## 9. Test-driven task plan

Implement the work in the following order. Record the intended red result before production changes and the green result after each implementation slice.

### Task 1: Add failing static contracts

**Files:** `tests/static_frontend.rs` only.

Add focused assertions for:

- visible hero text naming **Gemini Migration Agent** and ADK;
- one orchestrator region spanning exactly four phases in the order Assess, Plan, Deploy, Optimize;
- each phase containing Inputs, Agent action, Jev gate, Output and Planned reference state text;
- Jev represented in all four phases;
- a human approval checkpoint between Plan and Deploy;
- exactly four representative migration lanes;
- each lane containing Source, What moves, Movement/service, Destination and Validation/evidence;
- required source and destination terms for documents, tabular files, relational data, and applications/jobs;
- the “destination is chosen by workload” warning;
- exactly five `.stack-layer` pillar controls and their existing names;
- the migration hub, lifecycle, assessment/discovery and large-dataset-transfer links;
- precise product source links used by the selected lane copy;
- preserved exact live/reference disclaimer;
- wording that Gemini coordinates but does not move bytes;
- deterministic counting and validation ownership;
- no unqualified live-service claims, no universal SAP route and no “all data goes to BigQuery” claim.

Prefer stable IDs, `data-*` attributes and short factual text fragments over full-section snapshots.

**Command:**

```bash
cargo test --locked --test static_frontend
```

**Expected red:** the new hero, lifecycle, lane and source assertions fail against `295bed5`; existing tests remain green.
**Expected green:** all static contracts pass after Tasks 3 to 5.

### Task 2: Add failing Playwright contracts

**Files:** `tests/e2e/landing.spec.ts`; `tests/e2e/links.spec.ts` only if link validation belongs there.

Add browser assertions for:

- semantic phase order Assess, Plan, Deploy, Optimize;
- one persistent visible **Gemini Migration Agent** label and adjacent ADK label;
- source and target text visible for every representative lane;
- lane semantic order unchanged at 1280 px, 390 px and print;
- the human checkpoint positioned between Plan and Deploy in content order;
- low-confidence Jev outcomes identifying human review;
- no horizontal overflow in the lifecycle, lanes, hero, guided view or page;
- keyboard access and visible focus for links and any existing disclosure controls;
- state and status understandable without colour;
- lifecycle, lane fields and live/reference labels visible in print;
- stable content under `prefers-reduced-motion: reduce`;
- no failed local assets, console errors or unexpected network requests;
- existing guided-example outcomes and routes unchanged.

Do not create interactive controls for a purely explanatory diagram. Test semantic text and reading order instead.

**Commands:**

```bash
npm run test:e2e -- tests/e2e/landing.spec.ts
npm run test:e2e:subpath -- tests/e2e/landing.spec.ts
```

**Expected red:** selectors for the orchestrator, phases and lanes are absent or retain the old five-stage flow.
**Expected green:** the landing tests pass at root and subpath after Tasks 3 and 4.

### Task 3: Implement the hero and lifecycle control plane

**Files:** `static/index.html`, `static/styles/landing.css`, and `static/assets/hero-migration.svg` only if the existing illustration cannot support the new hierarchy.

Implement the smallest semantic structure that satisfies Tasks 1 and 2:

- revise the hero so Gemini Migration Agent and ADK are explicit and planned;
- retain the live/reference ribbon unchanged in meaning;
- replace the current five product-stage rail with the four official phases;
- add one orchestration band, repeated Jev gates and the human checkpoint;
- place products under phases instead of presenting logos as linear peers;
- use text-first responsive HTML and decorative CSS connectors;
- reuse the existing local Gemini and ADK marks;
- avoid JavaScript unless the existing page behaviour requires a small data mapping for the guided view.

**Commands:**

```bash
cargo test --locked --test static_frontend
npm run test:e2e -- tests/e2e/landing.spec.ts
```

**Expected red during implementation:** lane and downstream-section assertions remain failing while hero and lifecycle assertions turn green.
**Expected green for this slice:** hero, phase order, orchestrator, gate, checkpoint, state, responsive and print lifecycle assertions pass.

### Task 4: Implement lanes and align landing sections

**Files:** `static/index.html`, `static/styles/landing.css`, and the existing inline guided-example script in `static/index.html` only if needed.

- add the four representative lanes with all five required fields;
- keep applications/jobs distinct from data movement;
- add the workload-based destination warning;
- reframe the five pillars as orchestrator capabilities without changing count or behaviour;
- add the compact guided-example orchestrator view without changing deterministic outcomes;
- add Source, What moves, Destination, Gemini Agent role and Jev gate to each use case;
- revise Where to start to the four official phases where it fits cleanly;
- preserve every route and current demo contract.

**Commands:**

```bash
cargo test --locked --test static_frontend
npm run test:e2e -- tests/e2e/landing.spec.ts
npm run test:e2e:subpath -- tests/e2e/landing.spec.ts
```

**Expected red during implementation:** technical-guide and documentation alignment assertions may remain failing.
**Expected green for this slice:** all landing structure, lane, pillar, guided-example, use-case, responsive and source-link contracts pass.

### Task 5: Align the technical guide and repository docs narrowly

**Files:** `static/technical-guide/index.html`, `static/styles/tech.css` only if needed, `README.md`, `docs/design.md`, and `docs/asset-provenance.md` only if Task 3 added an official asset.

- add the Agent orchestration subsection and source-to-destination field definitions;
- preserve the exact live/reference boundary and product responsibility split;
- align README and design prose with the Gemini-led hierarchy and four phases;
- preserve all labs, benchmark material, engine grades and existing evidence;
- record official provenance only for newly added assets.

**Commands:**

```bash
cargo test --locked --test static_frontend
bash tests/static-checks.sh
npm run test:e2e -- tests/e2e/links.spec.ts
```

**Expected red:** before the docs update, guide or alignment assertions added in Task 1 fail.
**Expected green:** all source, guide, static-asset and documentation contracts pass.

### Task 6: Complete visual and browser verification

**Files:** no production file unless verification exposes a defect. Store screenshots only in the repository's established evidence location if the project already tracks them; otherwise attach them to the pull request rather than adding a new directory.

Verify:

- desktop at 1280 px;
- mobile at 390 px;
- print with background graphics disabled;
- keyboard navigation and visible focus;
- reduced motion;
- root and subpath hosting;
- the local Rust-served page where relevant;
- no overflow, failed assets, console errors or remote runtime assets;
- first-screen comprehension: orchestrator, what moves, source, target, phase, gate and live state.

**Commands:**

```bash
npm run test:e2e
npm run test:e2e:subpath
npm run test:live
```

**Expected red:** any visual or runtime regression produces a specific failing assertion or recorded defect before correction.
**Expected green:** all browser suites pass and screenshots show the required hierarchy at 1280 px, 390 px and print.

## 10. Acceptance criteria

The implementation is complete only when:

- a reader can answer within one screen: Who orchestrates? What moves? From where? To where? Through what phase? Who gates it? What is live?;
- Gemini Migration Agent is the most visually prominent system role after the business outcome and is clearly built with ADK;
- Assess, Plan, Deploy and Optimize appear in the official order;
- at least four representative lanes explicitly show source, artefact or workload, movement service, destination and validation evidence;
- every lane is labelled as a representative reference path rather than an automatic recommendation;
- Jev appears as a gate in all four phases;
- human approval is visible between Plan and Deploy and for low-confidence outcomes;
- the agent is never depicted or described as moving bytes, performing exact row counts or bypassing approval;
- exactly five pillars remain;
- all current demos, routes and deterministic mechanics remain unchanged and truthfully labelled;
- the exact planned/not-live disclaimer remains;
- the page is usable at 1280 px and 390 px, in print, with reduced motion and by keyboard;
- no runtime image, script, style or font is remote;
- browser assets contain no secrets, tokens, GCP project IDs, analytics identifiers, raw PII or credentials; and
- every numeric or third-party capability claim has a primary source where available.

## 11. Non-goals

Do not include:

- ADK, Gemini, MCP, RAG Engine, AlloyDB, Jev or other Google Cloud service implementation;
- service provisioning, deployment or runtime wiring;
- a new front-end framework, dependency or build step;
- backend, API, route or fixture changes unless a brand-default change becomes unavoidable, which is not expected;
- a generic claim that all data goes to BigQuery;
- an exhaustive migration-product catalogue;
- a universal SAP migration path;
- Soul Machines;
- GitHub Pages restoration; or
- deployment in this task.

## 12. Verification and review gates

Run the complete repository gates from the root after the focused red-to-green sequence:

```bash
cargo fmt --check
cargo test --locked
cargo audit
bash tests/static-checks.sh
bash tests/test-deploy.sh
npm run test:e2e
npm run test:e2e:subpath
npm run test:live
git diff --check
```

Where the live suite needs a separately started local service or environment, use the repository's established harness and report any justified skip. Do not weaken a gate to make the change pass.

Before merge, require independent reviews for:

1. **Factual and content accuracy:** compare each lifecycle, product, supported-source and destination claim with the linked Google primary documentation. Recheck the live/reference boundary, MCP-not-parser wording, deterministic validation ownership and AlloyDB-to-hosted-Jev boundary.
2. **Accessibility:** landmark and heading order, semantic phase and lane order, keyboard and focus behaviour, text alternatives, non-colour state, reduced motion and print.
3. **Visual quality:** 1280 px, 390 px and print. Check hierarchy, first-screen comprehension, lane arrows, text wrapping, source notes and overflow.
4. **Security and static assets:** local-only runtime assets, inert SVGs, provenance, no secrets, identifiers, analytics or PII, and no unexpected requests.

Record every finding and disposition point by point. Fix accepted findings, rerun the relevant gates and obtain re-review. Both independent factual/content and implementation reviews must clear before merge.

## 13. Ordered implementation checklist

1. [ ] Re-read `AGENTS.md`, this plan and the 2026-10-13 handover; confirm baseline `295bed5` and preserve unrelated work.
2. [ ] Verify every proposed migration-service claim against its linked primary documentation and narrow unsupported wording.
3. [ ] Add failing static contracts for the hero, four phases, orchestrator, Jev gates, checkpoint, lanes, sources, five pillars and factual guardrails.
4. [ ] Run the focused Rust test and record the expected red result.
5. [ ] Add failing Playwright contracts for semantic order, responsive layout, print, keyboard, reduced motion and asset or console failures.
6. [ ] Run root and subpath focused browser tests and record the expected red result.
7. [ ] Implement the Gemini-led hero and four-phase orchestration control plane.
8. [ ] Implement the four representative source-to-destination lanes and workload-based destination note.
9. [ ] Reframe the five pillars, guided example, use cases and Where to start without changing demo mechanics or routes.
10. [ ] Align the technical guide, README and design document narrowly; update asset provenance only if an official asset was added.
11. [ ] Run focused static and browser tests until green.
12. [ ] Run the complete formatting, Rust, audit, static, deploy, root, subpath, live and diff gates.
13. [ ] Capture and review desktop, mobile and print evidence after tests pass.
14. [ ] Complete independent factual/content, accessibility, visual and security reviews.
15. [ ] Fix every accepted finding point by point, rerun affected gates and obtain re-review.
16. [ ] Prepare the change for review without deploying. Merge and release remain separate authorised work.
