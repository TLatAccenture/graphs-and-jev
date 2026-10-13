# Google Cloud Migration and Modernisation Acceleration Handover

**Status:** Implemented on branch `gcp-acceleration-story`, uncommitted; awaiting independent review of the final state. Tests and code were partly written together, so the pull request records the passing gates rather than a strict red-then-green history.
**Repository:** `TLatAccenture/graphs-and-jev`
**Worktree:** `/Users/anthonylui/.config/superpowers/worktrees/graphs-and-jev/gcp-acceleration-story`
**Branch:** `gcp-acceleration-story`
**Baseline commit:** `42c8a8f` (merge of PR #5; `4ca75ee` is its feature commit)
**Scope:** Documentation and visual redesign of the existing public publication. This work does not provision or wire new Google Cloud services.

**Precedence:** For the publication name, landing story, Google Cloud product roles and icons, this handover wins over `AGENTS.md`, `docs/plans/2026-10-12-mcp-shed-cursor-handover.md` and the 2026-10-13 presentation-readiness plan. Every other rule in those documents still applies. Update `AGENTS.md` in the same change: its opening description, the precedence line, rule 8 (sources may also come from Section 5 of this handover) and rule 12 (branch `gcp-acceleration-story`, not `mcp-shed-wsN`).

## 1. Purpose and user intent

Redesign the existing publication so Google Cloud is substantially more prominent across the landing page and the relevant technical-guide content. Keep the public Cloud Run demo, current runtime, demonstrations and existing content. Add context and clarify the story rather than replacing the publication wholesale.

The top-level publication and hero concept must move away from **The MCP Tool Shed**. The implementing agent must choose concise exact title copy that preserves the meaning **Stack migration and modernisation acceleration with Google Cloud**. Validate the chosen title in the desktop and mobile header, especially at 390 px. The MCP Tool Shed remains the connectivity pillar. It is not the name of the whole publication.

The redesign must:

- preserve the existing same-origin Rust and static Cloud Run publication;
- preserve all current demonstrations, use cases and the technical-guide link;
- distinguish what runs today from a planned Google Cloud reference architecture;
- keep the five-pillar argument exactly five pillars;
- make Jev and the System 1 layer central to the business-value story: it filters noise and gates routine decisions so Gemini can spend System 2 reasoning on higher abstractions;
- name Google Agent Development Kit (ADK) explicitly in the logic and orchestration story;
- introduce Google Cloud product roles and official marks without implying that those products are live in this demo;
- retain the plain-language **Why now** argument, but move parenthetical source links into compact numbered source notes below that section; and
- use official primary sources for claims and official Google Cloud assets for marks.

This publication remains an Accenture thought piece. It is not a product or service offer from Accenture or Google Cloud. Jev is not a Google Cloud product.

## 2. Factual boundary: live system versus reference architecture

This boundary is decided. Do not reopen it as an architecture question.

### 2.1 Current live implementation

The public service currently consists of one Rust process serving static pages and the same-origin API from one Cloud Run service. The catalogue API and demonstrations use deterministic rules and browser fixtures as configured. The runtime does not currently host or invoke real MCP servers, AlloyDB AI, Google Cloud RAG Engine, ADK, Gemini, or DiffusionGemma.

Use a compact status ribbon near the top of the landing page with two clearly differentiated states:

- **Live today:** Rust and static publication on Cloud Run, deterministic rules and the existing demonstrations.
- **Google Cloud reference architecture:** planned product roles for a production migration and modernisation system. Not connected to this demonstration.

The catalogue source changes with deployment mode: event builds set `API_MODE_CATALOGUE=fixtures`, so the catalogue then comes from browser fixtures rather than the API. Ribbon wording must be true in both modes. Either keep it mode-neutral, as above, or fill the catalogue line from `SITE_CONFIG` at runtime with a static mode-neutral fallback.

Do not let a product logo, architecture rail or nearby heading weaken this distinction. Each reference-architecture section, diagram caption and technical-guide expansion must visibly say **planned**, **reference architecture**, or **not live in this demo**.

### 2.2 Target Google Cloud reference architecture

Present the following as a planned reference architecture, not as deployed state:

- **MCP servers:** governed tool endpoints may run on Compute Engine where persistent processes, specialised runtimes or network placement require it. MCP is the protocol and tool boundary, not a parser.
- **Google Cloud RAG Engine:** the managed retrieval layer, including document parsing and retrieval where suitable.
- **AlloyDB AI:** operational PostgreSQL state. AlloyDB AI calls TypeSafe's hosted Jev API through AI functions and Model Endpoint Management. Jev is not embedded in AlloyDB and is not hosted inside AlloyDB.
- **DiffusionGemma:** a model that can be deployed on Compute Engine or Cloud Run behind the same discriminative-model API contract that Jev serves.
- **ADK with Gemini:** System 2 reasoning, agent orchestration and tool use. ADK must be named explicitly rather than folded into a generic agent-platform label.
- **Cloud Storage:** raw documents and source objects.
- **BigQuery:** normalised and analytical data, eval records and telemetry.
- **Spanner Graph:** lineage, dependency and semantic context.
- **Looker:** reporting and operational visibility.
- **Cloud Run:** the current publication and API, and a possible host for stateless agent or UI services in the reference architecture.

The architecture must assign counting and validation to deterministic software. Code and SQL count rows, validate schema shapes and enforce exact invariants. Model layers interpret intent, route work, classify or gate bounded decisions. Do not describe MCP, RAG or a model as doing deterministic parsing or counting that belongs in code.

## 3. Recommended page hierarchy

Keep the landing page's existing story and demonstrations, with the following additions and clarifications.

### 3.1 Header and hero

Replace the publication-level brand name in the shared header, document title, description and hero with a concise form of **Stack migration and modernisation acceleration with Google Cloud**. The exact title is an implementation decision, but it must retain all three ideas: stack migration, modernisation acceleration and Google Cloud.

Keep **MCP Tool Shed** as the title of pillar 1, Connectivity. Do not remove it from the publication or dilute its role as the governed connection boundary.

The hero should lead with the migration and modernisation outcome, then explain that Google Cloud supplies the reference platform across connectivity, retrieval, operational data, reasoning, lineage and reporting. It must not imply that the reference stack powers the live demo.

### 3.2 Status ribbon

Place a compact, accessible ribbon immediately after the hero introduction or directly below the hero. It must compare **Live today** with **Google Cloud reference architecture** in words, not by colour alone. It should remain readable in print and at 390 px.

### 3.3 Why now and source notes

Keep the current plain-language argument. Remove citation links from prose parentheses and use numbered markers linked to a compact source-note list immediately below the section. Preserve the existing primary sources for Google long context, Lost in the Middle, Anthropic's MCP code execution and context engineering articles, the MCP specification and Stripe. Retain the VirtusLab link only with a visible **secondary source** label.

Every number and third-party capability claim must link to a source. Do not leave numerical markers without a corresponding note, and do not number unsourced editorial statements.

### 3.4 Google Cloud architecture rail

Add a highly visual architecture rail or flow that works left to right on wide screens and in a single logical reading order on mobile and print. Each node needs:

- a readable product or role name;
- a concise responsibility;
- a state label such as **Live today** or **Planned reference**;
- an official local icon when policy permits; and
- visible text that does not depend on recognising the icon.

A suitable flow is:

1. source systems and raw documents;
2. governed MCP connectivity and managed retrieval;
3. operational state and discriminative gate;
4. ADK and Gemini System 2 orchestration;
5. lineage, analytics and reporting.

This rail explains product roles. It does not create a sixth pillar and must not replace the five-pillar section.

### 3.5 Five pillars, exactly five

Keep exactly five top-level pillars in the current page order, 00 to 04. Integrate Google Cloud roles into their existing argument:

- **00 · Existing stack:** keep the name. Add Cloud Storage as the planned raw-document and source-object landing zone, without inventing another pillar.
- **01 · Connectivity, MCP Tool Shed:** governed MCP servers and scoped tools. Compute Engine is one possible planned host, not part of MCP's definition.
- **02 · Reflexes, System 1 and Jev:** bounded discriminative decisions and routing. AlloyDB AI may call TypeSafe's hosted Jev API in the reference architecture, and AlloyDB holds operational state. DiffusionGemma may implement the same discriminative API separately. This layer filters routine noise so Gemini can reason about higher-level migration decisions.
- **03 · Logic and context:** rename the title from "Gemini Enterprise Agent Platform" to "ADK with Gemini". ADK with Gemini provides System 2 orchestration. RAG Engine provides managed retrieval. Spanner Graph provides lineage and relational context. Keep responsibilities distinct. Keep the Gemini Enterprise Agent Platform link as the place ADK agents are built and deployed, worded as planned.
- **04 · Visibility and cost:** keep "Modernisation Springboard" as the title. BigQuery stores analytical data, eval records and telemetry; Looker reports outcomes, quality and cost.

Qualify the hero sentence "Agents on Gemini Enterprise Agent Platform do the work" as the planned reference architecture, or reword it around ADK with Gemini, so it no longer reads as a description of the live demo.

The current page presents **Existing Stack** as layer 00 and the four capability layers after it. Preserve exactly five rendered `.stack` entries and the current accessible disclosure behaviour. Copy can clarify product roles inside those five entries.

### 3.6 Existing examples and technical guide

Keep the guided decision example, Environmental Agency catalogue discovery, lineage-guarded migration, Little Bunnies and the technical-guide route. Do not rewrite their mechanics to simulate the reference architecture. Add short state labels where a reader could otherwise mistake fixture or deterministic behaviour for a live model or managed service.

Extend the technical guide narrowly. Add or amend sections that explain the Google Cloud reference flow, ADK orchestration, RAG Engine retrieval and AlloyDB-to-hosted-Jev call boundary. Preserve existing labs, graph-engine grades, Spanner Graph evidence and current demonstration contracts.

## 4. Product wording and claims

### 4.1 Required accurate wording

Use wording of this kind:

- **MCP servers may be hosted on Compute Engine in the planned reference architecture.**
- **The current demo does not run MCP servers.**
- **AlloyDB AI can call TypeSafe's hosted Jev API through AI functions and Model Endpoint Management in the planned reference architecture.**
- **Jev remains a hosted TypeSafe service; it is not embedded in AlloyDB.**
- **Google Cloud RAG Engine is the planned managed retrieval layer. It is not connected to this demo.**
- **ADK with Gemini is the planned System 2 orchestration layer.**
- **DiffusionGemma can be deployed behind the discriminative-model API on Compute Engine or Cloud Run in the reference architecture.**
- **Cloud Run hosts the live publication and same-origin API today.**
- **MCP exposes tools and resources. Deterministic code or SQL parses exact formats, counts rows and validates schemas.**

### 4.2 Prohibited unqualified claims

Do not publish these phrases as standalone facts:

- **MCP runs on GCE.**
- **Jev runs on AlloyDB.**
- **RAG is live.**
- **ADK powers this demo.**
- **Gemini decides the current demonstrations.**
- **DiffusionGemma is deployed here.**

A test should reject those exact unqualified claims. Qualified planned-reference wording remains permitted.

Avoid language that collapses product boundaries. RAG Engine retrieves context; it does not replace MCP. MCP exposes tools and resources; it is not a document parser. AlloyDB stores operational PostgreSQL state and invokes an external model endpoint; it does not contain Jev. ADK orchestrates agents; Gemini supplies System 2 model reasoning. Spanner Graph stores and queries graph context; it is not the analytical telemetry warehouse.

## 5. Primary source register

Use primary sources wherever available. Link claims at the point of use or through the numbered source notes directly associated with the section.

### 5.1 New Google Cloud and agent sources

1. [Build an ADK agent in Gemini Enterprise Agent Platform](https://docs.cloud.google.com/gemini-enterprise-agent-platform/build/adk)
2. [ADK and MCP integration](https://google.github.io/adk-docs/mcp/)
3. [ADK AG-UI integration](https://google.github.io/adk-docs/integrations/ag-ui/), optional: cite it only if the page makes a claim about agent-to-UI streaming, otherwise leave it out
4. [RAG Engine layout parser integration](https://docs.cloud.google.com/gemini-enterprise-agent-platform/build/rag-engine/layout-parser-integration)
5. [Single-agent AI system with ADK and Cloud Run](https://docs.cloud.google.com/architecture/single-agent-ai-system-adk-cloud-run)
6. [AlloyDB AI and Jev codelab](https://codelabs.developers.google.com/alloydb-ai-jev?hl=en#0)
7. [Discriminative models codelab](https://codelabs.developers.google.com/codelabs/discriminative-models#0)
8. [Google Cloud official icon library](https://cloud.google.com/icons)

### 5.2 Existing Why now sources to preserve

9. [Google Gemini long context](https://ai.google.dev/gemini-api/docs/long-context)
10. [Lost in the Middle](https://aclanthology.org/2024.tacl-1.9/)
11. [Anthropic: code execution with MCP](https://www.anthropic.com/engineering/code-execution-with-mcp)
12. [Anthropic: effective context engineering for AI agents](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents)
13. [Model Context Protocol server specification](https://modelcontextprotocol.io/specification/2026-07-28/server)
14. [Stripe Minions](https://stripe.dev/blog/minions-stripes-one-shot-end-to-end-coding-agents)
15. [VirtusLab report on Stripe's toolshed](https://visdom-maturity-matrix.virtuslab.com/guides/infrastructure/toolshed-model-400-tools-behind-one-mcp-stripe), secondary source

Keep existing primary product sources already used by the technical guide where their claims remain, including BigQuery, Looker and Spanner Graph documentation. Recheck claim wording against the linked page before publishing.

## 6. Official icon and asset policy

Use only assets from the [official Google Cloud icon library](https://cloud.google.com/icons). Copy approved SVG files into the repository. The published site must make no runtime request for icons, fonts, scripts, images or styles from Google or another third party.

The core icon pack (checked 2026-10-13) has unique marks for Compute Engine, AlloyDB, Cloud Run, Cloud Storage, BigQuery, Cloud Spanner and Looker. Use those. Reuse the existing `static/assets/engine-logos/google-cloud-spanner.svg` for Spanner Graph rather than adding a second copy.

The pack has no unique mark for Gemini, ADK or RAG Engine. Owner decision (13 October 2026): use the official ADK logo from the ADK documentation for **Agent Development Kit (ADK)**, and the Gemini product logo that Google's [Gemini Enterprise Agent Platform documentation](https://docs.cloud.google.com/gemini-enterprise-agent-platform) displays, stored locally as Google's published PNG. Never use a category icon in its place. Keep **Google Cloud RAG Engine** text only. Do not draw or infer logos, and do not use the Vertex AI mark as a stand-in.

For every added SVG:

- preserve title and licence metadata supplied with the source where applicable;
- retain the downloaded source file without adding scripts, event handlers, external references or remote CSS;
- ensure it is inert under the existing `engine_marks_are_inert` test in `tests/static_frontend.rs`, extended to cover any new asset directory;
- use useful alt text when the mark conveys content, or empty alt text when adjacent text already names the product;
- avoid any composition that implies endorsement, partnership or that Jev is a Google Cloud product; and
- add a provenance row to `docs/asset-provenance.md`.

This repository has no tracked provenance file. The engine-logo README exists only in the separate graphragapp repository. Create `docs/asset-provenance.md`, outside `static/` so it is not served publicly. Record the eight existing engine logos from the graphragapp README as well as the new Google Cloud icons.

Each provenance record should include the local filename, product/category name, official source URL, retrieval date, title or licence metadata retained, and any transformation. Optimisation must not strip legally or factually important metadata.

## 7. Likely implementation files

Keep the change narrow. The expected files are:

- `static/index.html`: publication name, hero, status ribbon, Why now source notes, Google Cloud architecture rail, pillar clarification and state labels.
- the `<title>`, meta description and fallback `data-brand="name"` text in `static/environmental-agency/index.html`, `static/lineage-migration/index.html`, `static/little-bunnies/index.html` and `static/technical-guide/index.html`. Visitors without JavaScript, and print, see these fallbacks.
- `AGENTS.md`, `package.json` (description), `deploy/cloud-run.env.example`, `docs/api-contract.md`, `tests/api_boundary.rs` and `tests/e2e/helpers.ts`, wherever they name the whole publication.
- `docs/asset-provenance.md`: new provenance record (see Section 6).
- `static/styles/landing.css`: responsive rail, ribbon, source notes, product marks, print treatment and reduced-motion-safe presentation.
- `static/config.js`: static-hosting default brand name and tagline.
- `src/site.rs`: Rust-rendered default brand name and tagline for `/config.js`.
- `README.md`: publication description and honest current/reference boundary.
- `docs/design.md`: current implementation and planned Google Cloud reference architecture.
- `static/technical-guide/index.html` and, only as needed, `static/styles/tech.css`: reference architecture and product-role explanation.
- `static/assets/...`: official local Google Cloud SVG assets.
- the canonical asset provenance README.
- `tests/static_frontend.rs`: static contracts and factual guardrails.
- `tests/e2e/landing.spec.ts`: interaction, responsive, accessibility and visual-state contracts.
- possibly `tests/e2e/links.spec.ts` and refreshed screenshots after browser verification.

`static/config.js` and `src/site.rs` define the same public brand through two serving paths. The first is used by plain static hosting; the second renders `/config.js` from environment variables in the Rust service. If the default brand changes in only one file, local static and Cloud Run builds will show different names and tests that override `SITE_CONFIG.brand.name` may hide the drift. Update and test both defaults together.

Do not change API routes, fixture contracts or runtime architecture merely to support this visual and documentation redesign.

## 8. Test-driven implementation sequence

Follow this order. Preserve a clear red-to-green record in the PR description.

### Task 1: Add factual and structure tests first

In `tests/static_frontend.rs`, add failing tests for:

- the new default publication brand in both `static/config.js` and the Rust site's rendered default;
- the exact **Live today** and **Google Cloud reference architecture** labels;
- an explicit statement that the reference architecture is planned and not live in this demo;
- required primary-source links from Section 5;
- exactly five rendered pillars;
- explicit ADK, Gemini, RAG Engine, AlloyDB AI, hosted Jev API, Cloud Run, Compute Engine, Cloud Storage, BigQuery, Spanner Graph and Looker role text;
- local icon paths only, files present, provenance documented and SVGs inert;
- no forbidden unqualified claims from Section 4.2;
- no remote script, image, style or font runtime assets; and
- no GCP project IDs, secrets, analytics identifiers or raw personal data in browser assets.

Prefer assertions on durable state labels, hrefs, data attributes and section IDs over brittle full-paragraph snapshots. Where an exact factual disclaimer prevents ambiguity, assert the exact sentence.

Run the narrow Rust test target and record the expected failures before changing production files:

```bash
cargo test --locked --test static_frontend
```

### Task 2: Add Playwright contract tests

Before implementation, add failing landing tests for:

- the status ribbon content;
- five, and only five, accessible pillar controls;
- architecture-rail reading order and text labels;
- every product mark having adjacent readable text;
- Why now numbered markers linking to source notes;
- no horizontal overflow at desktop and 390 px;
- keyboard operation and visible focus;
- no console errors or failed local asset requests;
- print visibility of status labels, architecture roles and source notes; and
- stable presentation under `prefers-reduced-motion: reduce`.

### Task 3: Implement landing copy, layout and assets

Update the shared brand defaults, landing content and narrow CSS. Add official local icons and provenance. Preserve the current demonstrations and JavaScript behaviour. Do not add a framework, package or build step.

Use semantic HTML for the architecture rail. If connectors are decorative, hide them from assistive technology. Do not put essential wording inside an SVG. Use the existing responsive and print design system before adding new rules.

### Task 4: Align the technical guide and repository docs

Add only the technical-guide material needed to explain the planned reference architecture and the current/live boundary. Align `README.md` and `docs/design.md`. Keep the technical labs and existing evidence intact.

Check every occurrence of the old publication-level name with `rg -n "The MCP Tool Shed"`. Preserve **MCP Tool Shed** where it names pillar 1 or the shed illustration, while replacing it where it names the whole publication. Historical plans under `docs/plans/` stay unchanged.

### Task 5: Complete browser and visual verification

Run Playwright for desktop and mobile. Verify click and keyboard interaction, all links, print, no overflow, no console errors and reduced motion. Test at minimum 1280 px wide and 390 px wide. Use the existing presentation sizes where practical.

Update screenshots only after the browser checks pass. Screenshots are evidence of the implemented state, not a substitute for tests.

## 9. Acceptance criteria

### 9.1 Factual integrity

- The publication clearly distinguishes the current live Cloud Run service from the planned Google Cloud reference architecture.
- Every reference-architecture section visibly says planned or reference and not live in this demo.
- No wording implies that MCP servers, AlloyDB AI, RAG Engine, ADK, Gemini or DiffusionGemma are connected to the current demonstrations.
- AlloyDB-to-Jev wording preserves the external hosted API boundary.
- MCP is described as a protocol and tool boundary, not a parser.
- Deterministic code and SQL own exact counting and schema validation; models interpret, route or gate.
- Product responsibilities do not overlap incorrectly.
- Every number and third-party capability claim has a linked source. VirtusLab is visibly secondary.

### 9.2 Content and information architecture

- The publication-level name is a concise form of the required Google Cloud migration and modernisation acceleration concept.
- MCP Tool Shed remains pillar 1, not the publication name.
- There are exactly five pillars.
- Jev/System 1 remains central and explicitly frees Gemini/System 2 for higher-level reasoning.
- ADK appears explicitly in logic and orchestration.
- Existing demonstrations, use cases and technical-guide link remain available and functional.
- Why now source links appear as compact numbered notes below the prose.

### 9.3 Accessibility and responsive behaviour

- Landmark, heading and reading order remain logical without CSS.
- State is conveyed in text, not colour alone.
- Product icons have accessible treatment and never replace product names.
- All interactive controls work by keyboard and retain visible focus.
- Touch targets remain at least 44 px where the existing design requires them.
- No horizontal overflow occurs at 390 px.
- The mobile header accommodates the chosen title without obscuring navigation or content.
- Reduced motion is respected.
- The page remains understandable and legible when printed with background graphics disabled.

### 9.4 Security and static assets

- All runtime scripts, styles, fonts and images are local.
- Added SVGs are inert and documented with official provenance.
- Browser assets contain no secrets, access tokens, GCP credentials, project IDs, analytics identifiers or raw personal data.
- Existing content security and same-origin boundaries are preserved.
- No GitHub Pages workflow, base path or deployment is restored.

### 9.5 Scope and delivery

- No GCP service is provisioned or integrated by this redesign.
- No new framework or dependency is introduced.
- The current Rust runtime, API routes and demonstrations are not replaced.
- Implementation does not run deployment or provisioning commands unless separately authorised.

## 10. Verification commands

Run from the repository root. Use locked dependency resolution where supported.

```bash
cargo fmt --check
cargo test --locked
cargo audit
bash tests/static-checks.sh
bash tests/test-deploy.sh
npm run test:e2e
npm run test:e2e:subpath
git diff --check
```

Also run the local live Rust suite and container smoke where proportionate to the changed paths. Use the repository's established commands and environment rather than inventing a second harness. At minimum, verify the Rust-served `/config.js`, landing page and affected technical-guide route. If the container or live suite is skipped, state the reason in the PR.

Do not weaken or delete an existing gate to make the redesign pass. Update an assertion only when the intended public contract genuinely changed.

## 11. Review gates

The change is not merge-ready until all findings are handled point by point and any fixes are re-reviewed.

1. **Independent factual and content review:** compare every product role and capability claim with the primary sources in Section 5. Check the current/live boundary and all planned-reference labels.
2. **Accessibility review:** semantics, keyboard, focus, contrast, text alternatives, state labels, reduced motion and print reading order.
3. **Security review:** local asset provenance, inert SVGs, content security policy implications, remote-request checks and the live/reference data boundary.
4. **Visual and browser review:** desktop at 1280 px, mobile at 390 px and print. Check header length, source notes, architecture flow, no clipping or overflow, and no console errors.

Record each finding, disposition, evidence and re-review result in the PR or linked review record. Do not dismiss a finding solely because the page is illustrative.

## 12. Git and release procedure

Implementation uses the repository's configured `TLatAccenture` identity only. Do not add a Cursor, agent or other co-author trailer.

1. Implement and verify on the feature branch.
2. Open a pull request with the red-to-green test record, factual boundary summary, asset provenance and screenshots.
3. Complete all review gates and fix/re-review every accepted finding.
4. Require all four hosted CI jobs to be green before merge.
5. Merge only after approval. Do not restore GitHub Pages.
6. From a clean, merged `main`, the authorised lead runs `deploy/deploy.sh ~/.config/graphs-and-jev/cloud-run.env` to create the zero-traffic candidate. Before running it, check whether that env file still carries the event lines (`API_MODE_CATALOGUE=fixtures`, `MIN_INSTANCES=1`), and confirm with the owner whether this release is an event build.
7. Verify the candidate page, API boundary, configuration, browser checks and revision state.
8. Promote with `deploy/promote.sh <revision>`.
9. Rewrite `docs/deployment/cloud-run.md` from observed live state after promotion, including revision, digest, traffic, settings and verification evidence.

The owner performs any required `gcloud` reauthentication interactively. Do not work around an expired login. Apart from the guarded deploy and promote scripts run by the authorised lead, GCP-mutating commands require separate approval.

## 13. Explicit non-goals

This redesign does not include:

- an actual MCP server on Compute Engine;
- AlloyDB or AlloyDB AI provisioning or integration;
- RAG Engine provisioning or integration;
- ADK or Gemini integration;
- DiffusionGemma deployment or integration;
- Cloud Storage, BigQuery or Spanner provisioning;
- Soul Machines content or integration;
- a new front-end framework, dependency or build system;
- replacement of the Rust runtime, APIs, fixtures or current demos;
- restoration of GitHub Pages; or
- unsupported claims that the planned products are live or validated here.

If implementation uncovers a real integration need, record it as follow-up work. Do not expand this redesign into platform provisioning.

## 14. Minimal implementation decisions

The architecture framing is decided and requires no question. The implementing agent may decide only:

- the exact concise publication title, while preserving the required meaning; and
- the narrow visual composition of the status ribbon and architecture rail within the existing design system.

Validate the selected title in the 390 px mobile header before treating it as final. Prefer a shorter visible header label with the full phrase in the hero if that is necessary for navigation and accessibility. Do not abbreviate Google Cloud to an unexplained initialism.

## 15. Implementation checklist

- [ ] Read `AGENTS.md` and the existing handover and presentation-readiness plans.
- [ ] Confirm the baseline and preserve unrelated work.
- [ ] Add failing Rust/static factual and structure tests.
- [ ] Add failing Playwright accessibility and responsive tests.
- [ ] Record the expected red test results.
- [ ] Choose and validate the concise publication title at 1280 px and 390 px.
- [ ] Update both static and Rust-rendered brand defaults.
- [ ] Add the live/reference status ribbon.
- [ ] Convert Why now citations to numbered source notes.
- [ ] Add the planned Google Cloud architecture rail without adding a sixth pillar.
- [ ] Preserve all current demos, use cases and routes.
- [ ] Add only official local Google Cloud icons and provenance.
- [ ] Align the technical guide, README and design document narrowly.
- [ ] Run formatting, Rust, audit, static, deployment and Playwright gates.
- [ ] Verify desktop, mobile, reduced motion, print, links and console output.
- [ ] Update screenshots only after browser verification.
- [ ] Obtain independent factual, accessibility, security and visual reviews.
- [ ] Resolve every finding point by point and re-review fixes.
- [ ] Open the PR under `TLatAccenture`; require all four hosted CI jobs green.
- [ ] Merge, deploy a candidate, verify, promote and rewrite the deployment record from observed state only when separately authorised.

Use the official sources in this handover and preserve British English throughout. Use no em dash. Keep the tone plain and factual. Link every number. Address the implementing agent directly and do not assume a particular model or vendor.
