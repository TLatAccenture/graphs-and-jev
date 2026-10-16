# AdminLTE worked example: one real migration through the Gemini-orchestrated story

**Date:** 16 October 2026
**Status:** Revision 2, after owner review on 16 October 2026. Draft. Not approved for implementation.
**Depends on:** the Gemini-Orchestrated Migration Story Plan, `docs/plans/2026-10-16-gemini-orchestrated-migration-story-plan.md`. That plan is not yet in the repository, so this revision has not been checked against it. See Section 2.
**Builds on:** `AGENTS.md`, `docs/plans/2026-10-13-gcp-migration-acceleration-handover.md` (the Google Cloud handover), `docs/plans/2026-10-12-mcp-shed-cursor-handover.md` (the earlier handover) and `docs/plans/2026-10-13-presentation-readiness.md`. Every rule in those documents still applies.
**Baseline:** `main` at `6a801c5` (merge of PR #6).
**Branch:** `gemini-admin-portal-migration`, cut from `main`. Revision 1 was commit `7137355` on `gcp-acceleration-story`; it is carried here as the first commit of this branch.

## 0. Changes from revision 1

| Owner finding | Change in this revision |
|---|---|
| A fourth use-case card does not fix the main story: Gemini still reads as stage 4 of a product pipeline. | AdminLTE becomes the worked example beneath the Gemini-orchestrated landing story (Section 1). The Gemini plan lands first. A dedicated page is optional (Stage B). |
| The Rust `portal` domain, `API_MODE_PORTAL` and the 501 response add server surface with no function. | All server, API, configuration and deployment work is removed (Section 8). The example is static HTML with ordinary page JavaScript, like Little Bunnies. No `siteApi.pipeline()` call. |
| Proceed 37, Ask 2, Escalate 22 presented a documentation lookup as a validated decision. | Evidence labels (Mapped, Choice required, Not covered) are separated from Jev outcomes (Section 6). The register is a draft for independent row-by-row review and is not published until reviewed. |
| Gemini appeared mainly in Deploy. | A persistent **Gemini Migration Agent, built with ADK**, has a defined role in all four phases. Jev gates every phase, and people approve uncertain decisions (Section 4). |
| Google sources were marked unchecked. | The owner checked them on 16 October 2026. Two newly found pages still need opening (Section 9). |
| The branch reused a merged branch. | New branch. `AGENTS.md` rule 12 change requested from the lead (Section 8). |

## 1. Purpose and position

The landing page must show one migration orchestrated end to end by a Gemini agent. This plan supplies the concrete case that proves that story: a fictional organisation's internal admin portal, built on [AdminLTE](https://github.com/ColorlibHQ/AdminLTE) 3.2.0, moves to AdminLTE 4 on Cloud Run through Google Cloud's four migration phases.

Sequence:

1. Implement the Gemini-Orchestrated Migration Story Plan. This plan does not start until it has merged.
2. **Stage A:** add AdminLTE as the first worked example carried through that architecture, on the landing page.
3. **Stage B, optional:** add a detail page with the full evidence register, only if the landing example would otherwise become too dense and the register has passed independent review.

Decisions made. Do not reopen them:

1. **AdminLTE is the migration subject, not the user interface.** No ColorlibHQ CSS, JavaScript, fonts, images or logos go into `static/`. Reasons: AdminLTE 4 needs Bootstrap 5, a framework (`AGENTS.md` rule 3); its demo dashboard page loads 11 files from `cdn.jsdelivr.net` (rule 16 and the Content Security Policy); its built `dist/css` folder is 7.1 MB on disk.
2. **AdminLTE 3.2.0 to AdminLTE 4** is the migration. Gentelella is out of scope.
3. **The example demonstrates the main story.** It is not a separate fourth demo beside it.
4. **No server work.** No Rust domain, no environment variable, no `siteApi` call.
5. **Evidence first, decisions second.** A guide lookup produces an evidence label. Only the Jev gate produces an outcome.
6. **A neutral fictional organisation.** The example uses "a fictional organisation" rather than the Environmental Agency, because the landing page's sample estate is not the Environmental Agency and new lore adds nothing.

## 2. Dependency on the Gemini-orchestrated story plan

That plan defines the landing story, names, section ids, colours and boundary wording. Where anything here differs from it, that plan wins. This revision assumes it provides:

- a landing section in which the **Gemini Migration Agent, built with ADK** orchestrates the whole migration across Assess, Plan, Deploy and Optimise;
- MCP tools, the Jev gate, human approval and lineage evidence as controls repeated in every phase; and
- the live-versus-planned wording for that story.

Before Stage A starts, the implementer checks these assumptions against the merged plan and updates this plan where they fail. If the Gemini story already has a phase lane, the worked example reuses it as a second track rather than adding another lane.

## 3. Factual boundary

| Kind | Covers | Label on the page |
|---|---|---|
| Real, sourced | AdminLTE's package files, migration guide and licence; Google Cloud's migration guidance | A link to the source |
| Fictional | The organisation, its portal, which templates use which plugin, and any test or hook result in the worked decisions | "fictional" or "illustrative" |
| Planned reference | The Gemini Migration Agent, ADK, Jev, MCP servers, Migration Center, Spanner Graph, BigQuery, Looker, and Cloud Run as the portal's target | "Planned reference" or "not live in this demo" |
| Live today | Cloud Run hosting this publication; the page's own JavaScript showing the example | "Live today" |

### 3.1 Required wording

- **The organisation and its portal are fictional. AdminLTE, its package data and its migration guide are real.**
- **This example runs on in-browser data. No agent, model or Google Cloud service is called.**
- **The Gemini Migration Agent plans, coordinates and explains. Deterministic jobs change code, run tests and deploy. It does not move bytes or change production on its own.**
- **Mapped means AdminLTE's guide documents a path. It does not mean the step is safe to automate.**
- **Jev, TypeSafe's System 1 decision model, gates each decision. People decide when the gate asks or escalates.**
- **Migration Center discovers infrastructure such as machines and databases. Code reads the package inventory.**
- **Cloud Run hosts the live publication and same-origin API today.** This publication is released as a zero-traffic Cloud Run candidate that is checked and then promoted (`deploy/deploy.sh`, `deploy/promote.sh`, `docs/deployment/cloud-run.md`).

### 3.2 Prohibited unqualified claims

A test rejects these exact phrases: **Migration Center assessed this portal**, **Gemini migrated the portal**, **Gemini rewrote the portal**, **ADK migrated the portal**, **The portal runs on Cloud Run**, **safe to automate** (except inside the required "does not mean the step is safe to automate" sentence), and any form of **ColorlibHQ endorses** or a partnership with ColorlibHQ or Google Cloud.

Counting and classification belong to code. No model is described as counting packages or parsing `package.json`.

## 4. Roles across the four phases

The Gemini Migration Agent persists across all four phases. The controls in the last row repeat in every phase.

| Phase | Google's guidance | Gemini Migration Agent, built with ADK | Deterministic tools and MCP | Jev gate | Person | Evidence produced |
|---|---|---|---|---|---|---|
| **Assess** | Inventory workloads and dependencies; Migration Center discovers infrastructure (G2, G3, G8) | Summarises the inventory and dependency evidence; names the gaps | MCP tools read the repository; code lists packages, templates, plugin calls, browser requirements and dependency edges, and counts them | Is the evidence complete enough to plan? | Confirms scope and browser support | Inventory, gap list, lineage edges |
| **Plan** | Build the foundation; group work into waves and move groups (G4, G9) | Proposes the target architecture, waves, acceptance criteria and rollback for each wave | Code looks up each component in AdminLTE's guide and assigns an evidence label; dependency edges constrain wave order | Decides each component: Proceed, Ask, Escalate, Refuse or No data | Approves the wave plan; answers Ask and Escalate items | Wave plan, decisions, approvals |
| **Deploy** | Run waves in sprints; test; roll back on failure; automate containerised deployment (G5, G10, G11) | Coordinates the jobs, proposes each widget swap for review, collects evidence | Codemods for class and attribute renames, package changes, tests, container build, zero-traffic Cloud Run candidate, promote or roll back; hooks block failing changes | May this candidate be promoted? | Approves each promotion | Diffs, test results, candidate revisions, eval records |
| **Optimise** | Iterate on cost, operations and automation; keep assessing (G6, G10) | Interprets validation, accessibility, bundle, performance and cost evidence; proposes the next improvement | Measures against the Assess baseline; BigQuery holds eval records and telemetry; Looker reports | Does the proposal enter the next loop? | Prioritises the next loop | Before-and-after comparison |
| **Every phase** | | | MCP tools reach systems; lineage evidence shows what depends on what | Jev gates | People approve uncertain decisions | Eval record per step |

Codemods are grounded in the source: AdminLTE's guide says a regex find-and-replace handles most of the Bootstrap renames and names a migration tool that automates the bulk (A4).

## 5. Stage A: worked example on the landing page

### 5.1 Placement and copy

- Placement: directly after the Gemini-orchestrated story section, as its worked example. Section id `#worked-example`, unless the Gemini plan names one.
- Eyebrow: "Worked example · AdminLTE 3 to 4". Heading (draft): "One real migration, start to finish."
- Intro (draft): "A fictional organisation runs an internal admin portal built on AdminLTE 3.2.0, an open-source dashboard template from ColorlibHQ. Its maintainers have since published AdminLTE 4 and a migration guide. Here the Gemini Migration Agent carries the portal through all four phases, Jev gates each decision, and people decide what the evidence cannot."
- State label: "Planned reference. This example runs on in-browser data. No agent, model or Google Cloud service is called."

### 5.2 Layout

Reuse the Gemini story's components and the existing design system. In reading order:

1. **Source and target.** Source: AdminLTE 3.2.0 portal, Bootstrap 4.6 and jQuery 3.6, 61 direct runtime dependencies (A2). Target: AdminLTE 4.10.1, Bootstrap 5.3 and no jQuery; it declares no runtime dependencies and one peer dependency, Bootstrap (A3, A4).
2. **Persistent orchestrator band**, spanning all four phases: "Gemini Migration Agent, built with ADK. Plans, coordinates and explains."
3. **Four phase blocks**, Assess, Plan, Deploy and Optimise, left to right at 1100 px and wider and stacked in reading order below that. Each block shows the Section 4 row in short lines: Gemini, tools, Jev gate, person.
4. **Repeated controls strip**: MCP tools, Jev gate, human approval, lineage evidence.
5. **Worked decisions** (5.3): six buttons and one result panel.

### 5.3 Worked decisions

Six decisions, at least one per phase, covering all five outcomes. Codes and labels are the existing ones (earlier handover Section 5.3). "Proceed" means the gate lets the step enter its wave; tests, hooks and the wave approval still apply.

| Phase | Request | Evidence | Evidence label | Jev outcome | What happens next |
|---|---|---|---|---|---|
| Assess | Start planning the portal migration. | Code read 61 direct runtime dependencies (A2). The portal's browser requirement is not recorded. AdminLTE's guide says to stay on AdminLTE 3 if a project relies on jQuery or Internet Explorer 11 (A4). | None | Ask | Gemini asks the portal owner which browsers the portal must support before drafting a plan. |
| Plan | Replace Select2 in the user admin forms. | The guide's replacement table names Tom Select (A4). Lineage evidence shows which templates use Select2 (illustrative). | Mapped | Proceed | The swap joins wave 2. Tests, hooks and the wave approval still apply. |
| Plan | Replace the Dropzone file upload. | The table lists two replacements: Dropzone 6 or FilePond (A4). | Choice required | Ask | A person chooses. Gemini records the choice and the reason. |
| Plan | Replace ion-rangeslider on the reports filter. | The table does not name ion-rangeslider. It names noUiSlider for bootstrap-slider, a different package (A4). | Not covered | Escalate | A person decides. The gap is recorded for the next assessment. |
| Deploy | Promote the wave 2 candidate to all traffic. | Illustrative: a hook found a Bootstrap 4 `data-toggle` attribute left in one template. The guide lists `data-toggle` to `data-bs-toggle` among the renames (A4). | None | Refuse | The candidate keeps zero traffic. Gemini reports the failing check and the codemod runs again. |
| Optimise | Show whether pages load faster after wave 2. | No performance baseline was recorded before the migration. | None | No data | Gemini proposes recording a baseline in the next loop. No figure is shown. |

The only counts in Stage A are 61, 0 and 1, each linked to A2 or A3. Version numbers link to the same sources.

### 5.4 Behaviour

- **No data file and no `siteApi` call.** The six decisions are written in the HTML, so they read correctly without JavaScript and in print. Ordinary page JavaScript turns them into a selector: buttons with `aria-pressed`, one result panel, an `aria-live` status line, and `siteUi.reveal` to bring the result on screen without moving focus (rule 17).
- Outcome shape plus text, never colour alone. Targets at least 44 px.
- Print and no-JavaScript: all four phases and all six decisions are visible.
- Reduced motion: no animation is added.

### 5.5 Density check

After Stage A, review the landing page at 1280 px and 390 px. Stage B is warranted only if the owner finds the example too dense, or wants the full register available as a detail view.

## 6. Evidence register (input to Stage B)

### 6.1 Two levels

- **Packages:** the 61 entries in the `dependencies` block of A2. The package name and declared version are facts.
- **Decision units:** packages a migration decides together. For example, `datatables.net` and its 14 Bootstrap 4 packages are one DataTables decision, not 15. The grouping is our judgement and is reviewed like every other row.

### 6.2 Evidence labels

| Label | Meaning | What it does not mean |
|---|---|---|
| **Mapped** | AdminLTE's guide documents a target or a removal path for this unit | That the step can proceed, or is safe to automate |
| **Choice required** | The guide documents more than one alternative | That either alternative suits this portal |
| **Not covered** | This source does not establish a path | That there is no path, or that the unit must escalate |

Each Mapped unit also records its **scope**: *full* where the guide maps the whole package, *capability only* where it maps one capability of a larger package (for example, jQuery UI sortable), and *removal* where the path is to remove the package rather than replace it (jQuery).

### 6.3 From evidence to a Jev outcome

The register holds evidence only. A Jev outcome needs the register plus the portal's own usage evidence from Assess. Typical outcomes, all illustrative in this publication:

| Evidence | Usage evidence from Assess | Typical Jev outcome |
|---|---|---|
| Mapped, full | Usage stays within what the guide maps; tests are defined | Proceed into the unit's wave |
| Mapped, capability only | Usage outside the mapped capability, or unknown | Escalate, or Ask if one fact is missing |
| Mapped, removal | Code counts no remaining calls to the package | Proceed |
| Mapped, removal | Calls remain | No decision yet: the unit stays in a later wave until the units that remove those calls are done |
| Choice required | Any | Ask |
| Not covered | Any | Escalate |
| Any | Conflicts with a stated hard constraint, such as Internet Explorer 11 support | Refuse |
| Not in the inventory | None | No data |

### 6.4 Independent review

The draft register in Appendix A is the plan author's reading. Before any row becomes fixture data, and before any count other than 61 is published:

1. A reviewer who did not write the draft checks every row against A2 at tag `v3.2.0` and A4 at tag `v4.10.1`.
2. For each row the reviewer records: package, declared version, decision unit, evidence label, scope, target, guide section, verdict (agree, change, unsure) and a note.
3. The owner resolves every "change" and "unsure" row.
4. The reviewed register is committed under `docs/` with the reviewer and date. Only then may Stage B build its data file from it.

## 7. Stage B, optional: detail page

Only after Section 5.5 calls for it and Section 6.4 is complete.

- Path: `static/adminlte-evidence/index.html`, linked from the worked example. Title: "AdminLTE evidence register".
- Data: `static/js/adminlte-evidence.js` sets `window.ADMINLTE_EVIDENCE` from the reviewed register, with sources. No `fetch`, no `siteApi`, no server change.
- Page JavaScript filters by evidence label, scope and decision unit, and counts what is shown, with an `aria-live` count line. Counts are computed, never typed into the HTML.
- The page shows evidence labels only. Jev outcomes appear only in the landing worked example, labelled illustrative.
- Footer: add the page to the footer link lists, as for the other detail pages.

## 8. Files and ownership

One workstream, **WS9 AdminLTE worked example**, owns these files for this change.

| Stage | Files |
|---|---|
| A | `static/index.html` (the `#worked-example` section only), `static/styles/landing.css`, `tests/static_frontend.rs`, `tests/e2e/landing.spec.ts`, `tests/static-checks.sh` (Section 10.3), `README.md` (one line), `docs/design.md`, `docs/screenshots/landing-1280.png` and `landing-390.png` |
| Review | `docs/adminlte-evidence-register.md` (new; the reviewed register) |
| B | `static/adminlte-evidence/index.html`, `static/js/adminlte-evidence.js`, `static/styles/adminlte-evidence.css`, the footer link list on every page, `tests/e2e/helpers.ts` (`PAGES`), `tests/e2e/adminlte-evidence.spec.ts` (new), its screenshots |

Unchanged by this plan: `src/**`, `deploy/**`, `static/config.js`, `static/js/api.js`, `static/fixtures/**`, `tests/api_boundary.rs`, `tests/test-deploy.sh`, `docs/api-contract.md`. No new Google Cloud icon and no AdminLTE or ColorlibHQ asset.

**Requests for the lead (`AGENTS.md`):**

1. Rule 12: replace the pinned branch with "Work on a fresh branch named for the change, cut from current `main`. Never reuse a branch whose pull request has merged."
2. Rule 8: also accept sources from Section 9 of this plan.

## 9. Sources

AdminLTE sources, checked first-hand by the plan author on 16 October 2026 at tag `v3.2.0` (commit `bd4d9c7`) and tag `v4.10.1` (commit `e13df78`). The owner confirmed the same facts on 16 October 2026.

| Id | Claim | Source |
|---|---|---|
| A1 | AdminLTE is MIT-licensed | https://github.com/ColorlibHQ/AdminLTE/blob/v4.10.1/LICENSE |
| A2 | AdminLTE 3.2.0 declares 61 direct runtime dependencies, including Bootstrap 4.6 and jQuery 3.6 | https://github.com/ColorlibHQ/AdminLTE/blob/v3.2.0/package.json |
| A3 | AdminLTE 4.10.1 declares no runtime dependencies and one peer dependency, Bootstrap `^5.3.8` | https://github.com/ColorlibHQ/AdminLTE/blob/v4.10.1/package.json |
| A4 | AdminLTE 4 migration guide: Bootstrap 5.3, no jQuery, class and attribute renames, plugin replacement table, Internet Explorer 11 guidance, suggested migration order | https://adminlte.io/themes/v4/docs/migration.html (source at https://github.com/ColorlibHQ/AdminLTE/blob/v4.10.1/src/html/components/docs/migration.mdx) |
| A5 | AdminLTE changelog | https://github.com/ColorlibHQ/AdminLTE/blob/v4.10.1/CHANGELOG.md |

Google Cloud sources. This cloud session cannot reach `docs.cloud.google.com`, so the plan author has not opened them. The owner opened them on 16 October 2026 and confirmed the four phases (Assess, Plan, Deploy, Optimize) and that G10 covers waves, move groups, sprints, testing, rollback and continued assessment. Recheck each sentence's wording against its source before publishing, as the Google Cloud handover requires.

| Id | Subject | Source | Checked |
|---|---|---|---|
| G1 | Migration documentation hub | https://docs.cloud.google.com/docs/migration | Owner, 16 Oct |
| G2 | Migrate to Google Cloud: Get started (the four phases) | https://docs.cloud.google.com/architecture/migration-to-gcp-getting-started | Owner, 16 Oct |
| G3 | Assess and discover your workloads | https://docs.cloud.google.com/architecture/migration-to-gcp-assessing-and-discovering-your-workloads | Owner, 16 Oct |
| G4 | Plan and build your foundation | https://docs.cloud.google.com/architecture/migration-to-google-cloud-building-your-foundation | **Found by search only; open before use** |
| G5 | Migrate from manual deployments to automated, containerised deployments | https://docs.cloud.google.com/architecture/migration-to-google-cloud-automated-containerized-deployments | Owner, 16 Oct |
| G6 | Optimise your environment | https://docs.cloud.google.com/architecture/migration-to-google-cloud-optimizing-your-environment | Owner, 16 Oct |
| G7 | Migration Center overview | https://docs.cloud.google.com/migration-center/docs/migration-center-overview | Owner, 16 Oct |
| G8 | Migration Center: discovery and assessment | https://docs.cloud.google.com/migration-center/docs/discovery-and-assessment-overview | Owner, 16 Oct |
| G9 | Migration Center: migration planning | https://docs.cloud.google.com/migration-center/docs/migration-planning-overview | Owner, 16 Oct |
| G10 | Migration Center: execute your migration | https://docs.cloud.google.com/migration-center/docs/migration-execution | Owner, 16 Oct |
| G11 | Deploy your workloads | https://docs.cloud.google.com/architecture/migration-to-gcp-deploying-your-workloads | **Found by search only; open before use** |

Page copy uses British spelling ("Optimise") and names Google's pages by subject in link text rather than quoting their titles. Keep using the Google Cloud handover's Section 5 sources for ADK, Gemini, Jev, Cloud Run, BigQuery, Looker and Spanner Graph.

## 10. Tests

Failing tests first, recorded, then implementation, as in the Google Cloud handover's Section 8.

### 10.1 Stage A

`tests/static_frontend.rs`:

- `#worked-example` exists with four `[data-phase]` blocks, `assess`, `plan`, `deploy` and `optimise`, in that order;
- each phase block names the Gemini Migration Agent (`[data-role="orchestrator"]`) and the Jev gate (`[data-role="gate"]`);
- the section links A2, A3, A4, G2, G9 and G10, and carries the state label and the required sentences in Section 3.1;
- the figure 61 sits beside the A2 link;
- the section calls neither `siteApi` nor `fetch`;
- no prohibited claim from Section 3.2 appears.

`tests/e2e/landing.spec.ts`, desktop 1280 px and mobile 390 px:

- the six decisions return Ask, Proceed, Ask, Escalate, Refuse and No data, each with its shape and text;
- choosing a decision brings the result on screen and leaves focus on the button;
- keyboard operation, visible focus, 44 px targets, no horizontal overflow;
- with JavaScript disabled, and in print, all four phases and all six decisions are readable;
- no console errors and no request to another host.

### 10.2 Stage B

The detail page joins the page lists in `tests/static_frontend.rs` and `tests/e2e/helpers.ts`. Its spec checks 61 package rows, that computed counts match the reviewed register, that each filter updates the rows and the `aria-live` count line, and that the page makes no API call.

### 10.3 Static check

`tests/static-checks.sh` currently rejects `optimiz` anywhere in an `.html` file, including inside G6's URL. Change the "US spelling" check so it ignores `href` and `src` attribute values and still checks all visible text. Show it still fails on prose: add "optimize" to a page locally, confirm the check fails, then remove it. Record both runs in the pull request.

## 11. Acceptance criteria

- The worked example sits beneath the Gemini-orchestrated story and uses its names and components.
- The Gemini Migration Agent, built with ADK, has a visible role in all four phases. Jev gates every phase. People approve uncertain decisions.
- Evidence labels and Jev outcomes are never shown as the same thing. No page presents a Mapped unit as safe to automate.
- No server, API, configuration or deployment file changes.
- The only published figures are sourced facts (61; 0 and 1). No evidence-label count is published before the register review.
- No AdminLTE or ColorlibHQ file, logo or screenshot is in `static/`. No request leaves the site.
- `AGENTS.md` rules hold: British English, no em dashes, relative paths, `fetch` only in `static/js/api.js`, accessibility patterns kept, results revealed without moving focus.

## 12. Open points

1. **The Gemini plan is not in the repository.** Push it, or commit it on this branch, so Section 2 can be checked against it before approval.
2. **Register reviewer.** Name who performs the Section 6.4 review. It blocks Stage B only.
3. **The old branch.** `origin/gcp-acceleration-story` now carries revision 1 (`7137355`) on top of PR #6's merge. Restoring it to `295bed5` needs a force push; the owner decides.

## Appendix A. Draft evidence register: unreviewed, not for publication

Source: the `dependencies` block of A2, read against A4. Draft by the plan author on 16 October 2026. Section 6.4 review pending. Tentative tallies: 37 packages in 21 Mapped units, 2 packages in 2 Choice required units, 22 packages in 22 Not covered units.

**Mapped (21 units, 37 packages)**

| Decision unit | Packages | Target in A4 | Scope | Review flag |
|---|---|---|---|---|
| Bootstrap | `bootstrap` | Bootstrap 5.3, with the class and `data-*` renames | full | The guide lists the renames "most likely to bite", not every Bootstrap 4 to 5 change |
| jQuery | `jquery` | Remove (jQuery removal section) | removal | Depends on every jQuery plugin unit |
| Icons | `@fortawesome/fontawesome-free` | Bootstrap Icons, the v4 standard; keeping FontAwesome is allowed | full | The guide says v3 demos used FontAwesome 6, but A2 declares `^5.15.4`. Possibly Choice required |
| Select2 | `select2`, `@ttskch/select2-bootstrap4-theme` | Tom Select | full | A compatibility stylesheet allows keeping Select2. Possibly Choice required |
| Dual list box | `bootstrap4-duallistbox` | Tom Select | full | |
| DataTables | `datatables.net`, `datatables.net-bs4` and 13 `datatables.net-*-bs4` extensions | Tabulator | full | The guide lists sorting, filtering, pagination, inline edit and export. Equivalence for each extension is not established |
| jsGrid | `jsgrid` | Tabulator | full | |
| Date and time pickers | `daterangepicker`, `tempusdominus-bootstrap-4` | Flatpickr | full | |
| Rich text | `summernote` | Quill | full | |
| Slider | `bootstrap-slider` | noUiSlider | full | |
| Checkboxes | `icheck-bootstrap` | Native `.form-check` | full | |
| Input masks | `inputmask` | IMask | full | |
| Knob | `jquery-knob-chif` | Chart.js 4 doughnut | full | The package is a fork; the guide links the original jQuery Knob |
| Vector map | `jquery-mapael` | jsVectorMap | full | |
| jQuery UI | `jquery-ui-dist` | SortableJS, for jQuery UI sortable | capability only | Other jQuery UI widgets are not covered |
| Calendar | `fullcalendar` | FullCalendar 7 | full | A2 declares `^5.10.1`, which the guide says is already jQuery-free. This is a version upgrade |
| Charts | `chart.js` | Chart.js 4 | full | |
| Lightbox | `ekko-lightbox` | GLightbox | full | |
| Scrollbars | `overlayscrollbars` | Native scrollbar CSS | full | Check the portal's usage. AdminLTE 4's own demo still loads OverlayScrollbars 2 from a CDN |
| Notifications | `toastr` | Bootstrap toasts | full | |
| Dialogs | `sweetalert2` | SweetAlert2, kept; jQuery-free since version 7 | full | |

**Choice required (2 units, 2 packages)**

| Decision unit | Package | Alternatives in A4 |
|---|---|---|
| File upload | `dropzone` | Dropzone 6 or FilePond |
| Colour picker | `bootstrap-colorpicker` | Pickr or the native colour input |

**Not covered (22 units, 22 packages)**

`@lgaitan/pace-progress`, `@sweetalert2/theme-bootstrap-4`, `bootstrap-switch`, `bs-custom-file-input`, `bs-stepper`, `codemirror`, `fastclick`, `filterizr`, `flag-icon-css`, `flot`, `fs-extra`, `ion-rangeslider`, `jquery-mousewheel`, `jquery-validation`, `jqvmap-novulnerability`, `jszip`, `moment`, `pdfmake`, `popper.js`, `raphael`, `sparklines`, `uplot`.

Review flags: `bootstrap-switch` has adjacent evidence (the guide's checkbox row says Bootstrap 5.3's own switches need no plugin) but is not named. `fs-extra` is a Node.js file-system library, so it may not belong in a browser inventory at all. Some Not covered packages may belong to a Mapped unit (for example, a dependency of a plugin being replaced); A4 does not establish that, so the reviewer decides with a separate source.
