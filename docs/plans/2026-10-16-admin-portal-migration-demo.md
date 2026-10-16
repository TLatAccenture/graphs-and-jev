# Admin portal migration demo: AdminLTE 3 to 4 through Google Cloud's migration phases

**Date:** 16 October 2026
**Status:** Draft for owner review. Nothing is implemented. No code, test or page has changed.
**Builds on:** `AGENTS.md`, `docs/plans/2026-10-13-gcp-migration-acceleration-handover.md` (the Google Cloud handover), `docs/plans/2026-10-12-mcp-shed-cursor-handover.md` (the earlier handover) and `docs/plans/2026-10-13-presentation-readiness.md`. Every rule in those documents still applies. This plan adds one use-case page. It does not change the publication name, the landing story, the five pillars or the Google Cloud product roles.
**Baseline:** `main` at `6a801c5` (merge of PR #6).
**Branch:** `gcp-acceleration-story`, restarted from `main` because PR #6 merged it. See open point 5 in Section 10.

## 1. Purpose and decisions made

Add a fourth use-case demonstration that walks one real, public stack migration through Google Cloud's four migration phases: Assess, Plan, Deploy and Optimise. The subject is an internal admin portal built on [AdminLTE](https://github.com/ColorlibHQ/AdminLTE), ColorlibHQ's MIT-licensed admin dashboard template, moving from AdminLTE 3.2.0 (Bootstrap 4.6 and jQuery) to AdminLTE 4 (Bootstrap 5.3, no jQuery) and onto Cloud Run.

Owner decisions, 16 October 2026. Do not reopen them:

1. **AdminLTE is the migration subject, not the user interface.** The page uses the site's own design system. No ColorlibHQ CSS, JavaScript, fonts or images are copied into `static/`.
2. **AdminLTE 3.2.0 to AdminLTE 4** is the migration. Gentelella is out of scope.
3. **Plan first.** Implementation starts only after the owner approves this plan.

Why the template is not embedded as the interface:

- AdminLTE 4 needs Bootstrap 5, which is a framework. `AGENTS.md` rule 3 forbids frameworks in `static/`.
- Its demo dashboard page loads 11 files from `cdn.jsdelivr.net` (fonts, Bootstrap, Popper, Chart.js, OverlayScrollbars, jsVectorMap, SortableJS). Rule 16 forbids any request leaving the site, and the Content Security Policy would block them.
- Its built `dist/css` folder is 7.1 MB on disk, and it would read as a second site inside the publication.

Why AdminLTE works as the subject:

- The migration is real and documented by its maintainers. The [AdminLTE 4 migration guide](https://adminlte.io/themes/v4/docs/migration.html) maps every AdminLTE 3 jQuery plugin to a vanilla-JavaScript replacement, lists the class and attribute renames, and gives a suggested migration order.
- Every number on the page can come from a tagged, public file. [AdminLTE 3.2.0's `package.json`](https://github.com/ColorlibHQ/AdminLTE/blob/v3.2.0/package.json) declares 61 runtime dependencies. [AdminLTE 4.10.1's `package.json`](https://github.com/ColorlibHQ/AdminLTE/blob/v4.10.1/package.json) declares none, with Bootstrap `^5.3.8` as its one peer dependency.
- The data maps onto the existing gate vocabulary without inventing anything: a plugin with one documented replacement can proceed, a plugin with two documented replacements needs a choice, and a dependency the guide does not name needs a person.

## 2. Factual boundary

Four kinds of statement appear on the page. Each must be visibly labelled as what it is.

| Kind | What it covers | Label on the page |
|---|---|---|
| **Real, sourced** | AdminLTE's package data, its migration guide, its licence, Google Cloud's published migration guidance | A link to the source |
| **Fictional** | The portal and its owner. Reuse the fictional **Environmental Agency** (open point 3) and say once on the page that it is fictional. | "fictional" |
| **Planned reference** | Every Google Cloud product role: Migration Center, Cloud Run as the portal's target, ADK with Gemini, Jev, Spanner Graph, BigQuery, Looker | "Planned reference" or "not live in this demo" |
| **Live today** | The in-browser rules that decide the demo, and Cloud Run hosting this publication | "Live today" |

### 2.1 Required wording

Use wording of this kind:

- **The portal and its owner are fictional. AdminLTE, its package data and its migration guide are real.**
- **This demo runs on in-browser rules. It calls no Google Cloud migration tool and no model.**
- **Migration Center discovers infrastructure such as machines, virtual machines and databases. In this plan it would discover the servers the portal runs on. Code reads the package inventory.**
- **Jev, TypeSafe's System 1 decision model, is the planned gate. Here the gate is a few in-browser word rules.**
- **ADK with Gemini is the planned System 2 layer that proposes each widget swap as a code change. Deterministic hooks check every change.**
- **Cloud Run hosts the live publication and same-origin API today.** This publication is itself released as a zero-traffic Cloud Run candidate and then promoted (`deploy/deploy.sh`, `deploy/promote.sh`, `docs/deployment/cloud-run.md`).

### 2.2 Prohibited unqualified claims

A test rejects these exact phrases (Section 8):

- **Migration Center assessed this portal.**
- **Gemini rewrote the portal.**
- **ADK migrated the portal.**
- **The portal runs on Cloud Run.**
- **ColorlibHQ endorses** (in any form), and any wording that implies a partnership with ColorlibHQ or Google Cloud.

Keep counting and classification in code. The page's script counts the inventory and applies the classification rule. No model is described as counting or parsing `package.json`.

## 3. Google Cloud's four phases, as the page uses them

**Not yet checked first-hand.** This session's network policy blocks `docs.cloud.google.com`, so the summaries below come from search results, not from reading the pages. Before any copy is written, the implementer opens every G source in Section 7, checks each summary against it, and corrects the plan if they differ.

| Phase | What Google's guidance asks (to recheck) | Sources |
|---|---|---|
| **Assess** | Build an inventory of workloads and their dependencies. Categorise them by business criticality, dependencies, allowable downtime and migration difficulty. Migration Center supports discovery and assessment of infrastructure. | G3, G7, G8 |
| **Plan** | Build the foundation (resource hierarchy, identity and access, network, billing, logging and monitoring). Group workloads into migration waves of applications with shared characteristics or dependencies; standalone applications suit a first wave. | G4, G9 |
| **Deploy** | Run the plan wave by wave. Prefer automated, containerised deployments to manual ones. Test, and roll back when tests fail. | G5, G10 |
| **Optimise** | Once workloads run, iterate on cost, operations and automation. Google describes this phase as a loop. | G6 |

Google's pages use US spelling ("Optimize"). Page copy uses British spelling ("Optimise") and names Google's pages by subject in link text rather than quoting their titles.

## 4. The demonstration

### 4.1 Page

- Path: `static/admin-portal-migration/index.html`. Page title: "Admin portal migration". `data-page-title="Admin portal migration"`.
- Eyebrow: "Use case · Follow Google Cloud's migration phases". H1: "Assess, plan, deploy, optimise."
- Lede (draft): "The Environmental Agency, a fictional organisation, runs an internal admin portal built on AdminLTE 3.2.0, an open-source dashboard template from ColorlibHQ. AdminLTE's maintainers have since rewritten it as AdminLTE 4: Bootstrap 5.3, no jQuery, and a documented replacement for each jQuery plugin. This demonstration follows Google Cloud's four migration phases to move the portal to AdminLTE 4 on Cloud Run, with a System 1 gate deciding each step."
- Banner, reusing the lineage page's change-safety banner: "**Illustrative, not a migration tool.** The package data and replacement table are AdminLTE's own. The portal and its owner are fictional. The demo runs in your browser and calls no Google Cloud service."
- Header, footer and `<head>` follow the shared pattern: `config.js`, `js/api.js`, `js/ui.js`, relative paths, `data-brand` fallbacks with the current brand name.

### 4.2 Sections in order

Fixed ids, because tests and the presentation run of show link to them.

1. **Hero and banner** (4.1).
2. **`#phases` phase rail.** An ordered list of four linked steps, Assess, Plan, Deploy and Optimise, left to right at 1100 px and wider and stacked in reading order below that. Each step shows its phase name, one line on what Google's guidance asks, and a state label ("Live today: in-browser rules" or "Planned reference"). Plain links to the four sections below. No scroll-spy and no motion.
3. **`#assess` Assess: know what the portal depends on.** A short paragraph with sources G3 and G8 on what the phase asks, then the inventory:
   - A table of all 61 runtime dependencies of AdminLTE 3.2.0, with columns Package, Declared version, Group, AdminLTE 4 replacement and Gate outcome. Caption names the source file and tag.
   - Group filter and outcome filter as toggle buttons (`aria-pressed`), with an `aria-live` count line such as "Showing 22 of 61 dependencies".
   - Counts by outcome, computed by the page's script from the fixture data, never typed into the HTML: Proceed 37, Ask 2, Escalate 22 (Appendix A). Beside them, the rule in one sentence: "Our classification: named in AdminLTE's replacement table with one replacement, proceed; with two, ask; not named, escalate."
   - Pillar roles, labelled "Planned reference": the MCP Tool Shed reaches the source repository through a governed MCP server; Spanner Graph records dependency edges such as `@ttskch/select2-bootstrap4-theme` depending on `select2`; Migration Center discovers the portal's servers, not its packages.
4. **`#plan` Plan: decide each component, then order the waves.** A short paragraph with sources G4 and G9, then:
   - **The gate demo** (4.3), on the lineage page's pattern: a request field, five example buttons, `aria-live` status, and a result panel brought on screen with `siteUi.reveal` without moving focus.
   - **The wave plan**, built from the fixture and following AdminLTE's own suggested migration order (A4): wave 1, framework (Bootstrap 4 to 5 class and `data-*` attribute renames, dark-mode attribute, icons); wave 2, the 37 Proceed items one widget at a time; wave 3, the Ask and Escalate items once a person has decided; wave 4, remove jQuery when no calls remain, then re-test breakpoints and print views.
   - The landing-zone foundation for the target (project, identity and access, network, logging), labelled "Planned reference".
5. **`#deploy` Deploy: ship each wave behind checks.** A short paragraph with sources G5 and G10, then the stages trail of the most recent gate result (gate, inventory, playbook, hook check, eval record), and the release pattern:
   - Planned reference: ADK with Gemini proposes each widget swap as a code change; deterministic hooks block it if an AdminLTE 3 class from the rename table remains, if a Bootstrap 4 `data-toggle` style attribute remains, if any jQuery call remains after wave 4, or if tests fail. Each wave becomes a container image on a zero-traffic Cloud Run revision, is tested, then promoted, with rollback on failure.
   - Live today: this publication is released the same way, as a zero-traffic Cloud Run candidate that is checked and then promoted.
6. **`#optimise` Optimise: measure, then go round again.** A short paragraph with source G6, then:
   - Before and after, from AdminLTE's own files: 61 runtime dependencies in 3.2.0 (A2); none in 4.10.1, with one peer dependency, Bootstrap 5.3 (A3). The page's script computes the before figure from the fixture; the after figure links to A3.
   - Planned reference: every step writes an eval record to BigQuery; Looker reports outcomes by wave, escalation rate and cost per migrated component. Describe the tiles in words. No cost or time figures.
7. **Scope and sources.** Scope note in the existing style. Source list with every A and G source used on the page, AdminLTE's MIT licence (A1), and the statement "AdminLTE is a ColorlibHQ project. Its name is used to identify it. This publication is not affiliated with or endorsed by ColorlibHQ."

### 4.3 Gate demo examples

Outcome codes and labels are the existing ones (earlier handover Section 5.3): `answer` Proceed, `clarify` Ask, `review` Escalate, `reject` Refuse, `no_data` No data.

| Example button text | Outcome | Message | Why lines (draft) |
|---|---|---|---|
| Replace Select2 in the user admin forms | answer (Proceed) | Proceed: replace Select2 with Tom Select. | AdminLTE's replacement table names Tom Select for Select2. The Select2 Bootstrap 4 theme goes with it. A compatibility stylesheet exists if Select2 must stay. |
| Replace the Dropzone file upload | clarify (Ask) | Two replacements are documented: Dropzone 6 or FilePond. Which should the portal use? | The table lists both. Choosing one would be a guess, so the agent asks. |
| Replace ion-rangeslider on the reports filter | review (Escalate) | ion-rangeslider is not in AdminLTE's replacement table. A person decides. | The dependency is known; its successor is not documented. The table names noUiSlider for a different slider, bootstrap-slider, so matching it here would be a guess. |
| Move to AdminLTE 4 and keep Internet Explorer 11 support | reject (Refuse) | Refused. AdminLTE 4 does not support Internet Explorer 11. | AdminLTE's guide says to stay on AdminLTE 3 if a project relies on jQuery or Internet Explorer 11. The gate stops the request before any tool is called. |
| Replace the audit log widget | no_data (No data) | Nothing in the inventory matches that component. Discover it before planning its migration. | The inventory holds AdminLTE 3.2.0's declared dependencies only. A custom widget would have to be found first. |

Rules, first match wins:

1. Refuse when the lower-cased request mentions Internet Explorer (`internet explorer`, `ie 11`, `ie11`).
2. Look up inventory names: every package name in Appendix A plus the library names AdminLTE's table uses (for example "select2", "dropzone", "datatables", "summernote"). Longest match first. Return that component's outcome from Appendix A.
3. No data when the request contains "replace", "upgrade", "migrate" or "remove" and no inventory name matched.
4. Otherwise Ask: "Name the component and the change, for example \"replace Select2\"."

Any of the 61 package names typed into the field returns its Appendix A outcome. Free text "hello" returns Ask, as on the lineage page.

## 5. Data contract

Follow the lineage page exactly: the page calls `siteApi.pipeline({domain: "portal", request, preference})`, and `static/fixtures/portal.js` registers `SITE_FIXTURES.portal`.

**This needs a server change, not just a fixture.** The Rust service defaults to `API_MODE=live`, so without a `portal` entry in `api.domains` the page would call the live API, get "Unknown domain" and show "Service unavailable". Mirror the lineage domain:

- `static/config.js`: add `portal: "fixtures"` to `api.domains`.
- `src/site.rs`: render `api.domains.portal` from `API_MODE_PORTAL`, default `fixtures`.
- `src/lib.rs`: `domain: "portal"` returns 501 with `{"detail": "Portal domain not implemented on this service."}`; the unknown-domain message lists `catalogue, lineage, portal`.
- `deploy/deploy.sh`: add `API_MODE_PORTAL` to the allowlist and to the `live`/`fixtures` validation. `deploy/cloud-run.env.example` documents it.

Response shape: the common fields from the earlier handover Section 9.2, plus:

```json
{
  "component": {"package": "select2", "declared": "^4.0.13", "group": "Form widget"},
  "replacement": {"name": "Tom Select", "alternatives": [], "source": "https://adminlte.io/themes/v4/docs/migration.html#third-party-plugin-replacements"},
  "wave": 2,
  "playbook": {"skill": "adminlte-3-to-4", "steps": [{"name": "string", "status": "done | skipped | blocked"}]},
  "eval": {"run_id": "string", "decision": "string", "confidence": 0.0, "tools_called": ["string"], "tokens_in": 0, "tokens_out": 0, "latency_ms": 0, "cost_usd": 0.0}
}
```

Stage ids: `gate`, `inventory`, `playbook`, `hook`, `eval`. Every fixture response sets `backend_label: "In-browser portal rules"`, `model: "portal-fixture-v1"` and `residency: "browser"`. Confidence and eval values are illustrative and the page says so, as the lineage page does. The fixture also exposes the 61-row inventory for the `#assess` table, so the table and the gate share one data source.

## 6. Files and ownership

One workstream, **WS8 Admin portal**, owns these files for this change. The lead agent owns `AGENTS.md` and this plan.

| Area | Files |
|---|---|
| New page | `static/admin-portal-migration/index.html`, `static/styles/portal.css`, `static/fixtures/portal.js`, optional original art `static/assets/admin-portal-phases.svg` (no vendor marks) |
| Landing and shared | `static/index.html` (a fourth `#use-cases` card and its footer link), the footer link list on every other page, `static/config.js` (`api.domains.portal`) |
| Server and release | `src/lib.rs`, `src/site.rs`, `deploy/deploy.sh`, `deploy/cloud-run.env.example`, `tests/test-deploy.sh`, `docs/api-contract.md` |
| Tests | `tests/static_frontend.rs`, `tests/api_boundary.rs`, `tests/e2e/admin-portal-migration.spec.ts` (new), `tests/e2e/helpers.ts`, `tests/e2e/presentation.spec.ts`, possibly `tests/static-checks.sh` (open point 2) |
| Docs | `README.md` (Pages list), `docs/design.md`, `docs/screenshots/admin-portal-migration-1280.png` and `-390.png` |
| Lead | `AGENTS.md`: rule 8 also accepts sources from Section 7 of this plan; rule 12 names the branch for this work |

No new Google Cloud icon is required. Use the existing local marks only where a product is already shown on the landing page. Migration Center is text only unless the official icon library has a unique mark for it; if one is added, it gets a row in `docs/asset-provenance.md`. Do not add any AdminLTE or ColorlibHQ logo or screenshot.

Do not change the five pillars, the architecture rail, the guided example or the other demos.

## 7. Source register additions

AdminLTE sources, checked first-hand on 16 October 2026 from the repository at tag `v4.10.1` (commit `e13df78`) and tag `v3.2.0` (commit `bd4d9c7`):

| Id | Claim | Source |
|---|---|---|
| A1 | AdminLTE is MIT-licensed | https://github.com/ColorlibHQ/AdminLTE/blob/v4.10.1/LICENSE |
| A2 | AdminLTE 3.2.0 declares 61 runtime dependencies, including Bootstrap 4.6 and jQuery 3.6 | https://github.com/ColorlibHQ/AdminLTE/blob/v3.2.0/package.json |
| A3 | AdminLTE 4.10.1 declares no runtime dependencies and one peer dependency, Bootstrap `^5.3.8` | https://github.com/ColorlibHQ/AdminLTE/blob/v4.10.1/package.json |
| A4 | AdminLTE 4 migration guide: Bootstrap 5.3, no jQuery, class renames, plugin replacement table, Internet Explorer 11 guidance, suggested migration order | https://adminlte.io/themes/v4/docs/migration.html (source: https://github.com/ColorlibHQ/AdminLTE/blob/v4.10.1/src/html/components/docs/migration.mdx) |
| A5 | AdminLTE changelog | https://github.com/ColorlibHQ/AdminLTE/blob/v4.10.1/CHANGELOG.md |

Google Cloud sources. **None is checked first-hand yet** (Section 3). G2 and G4 URLs are unconfirmed: find the current URLs from G1 before use.

| Id | Claim | Source |
|---|---|---|
| G1 | Google Cloud migration documentation hub | https://docs.cloud.google.com/docs/migration |
| G2 | Migrate to Google Cloud: Get started (the four phases; migration types) | https://docs.cloud.google.com/architecture/migration-to-gcp-getting-started (unconfirmed) |
| G3 | Assess and discover your workloads | https://docs.cloud.google.com/architecture/migration-to-gcp-assessing-and-discovering-your-workloads |
| G4 | Plan and build your foundation | URL to confirm from G1 |
| G5 | Migrate from manual deployments to automated, containerised deployments | https://docs.cloud.google.com/architecture/migration-to-google-cloud-automated-containerized-deployments |
| G6 | Optimise your environment | https://docs.cloud.google.com/architecture/migration-to-google-cloud-optimizing-your-environment |
| G7 | Migration Center overview | https://docs.cloud.google.com/migration-center/docs/migration-center-overview |
| G8 | Migration Center: discovery and assessment | https://docs.cloud.google.com/migration-center/docs/discovery-and-assessment-overview |
| G9 | Migration Center: migration planning and waves | https://docs.cloud.google.com/migration-center/docs/migration-planning-overview |
| G10 | Migration Center: execute your migration | https://docs.cloud.google.com/migration-center/docs/migration-execution |

Keep using the Google Cloud handover's existing sources (Section 5) for ADK, Gemini, Jev, Cloud Run, BigQuery, Looker and Spanner Graph.

## 8. Test-driven implementation sequence

Follow the Google Cloud handover's order (its Section 8): failing tests first, recorded, then implementation.

### Task 1: Rust and static tests

`tests/static_frontend.rs`:

- the new page is in the page lists, is served with 200 and uses `siteApi.pipeline(`;
- it carries the banner text, the fictional statement, "Planned reference" and "Live today" labels, and section ids `phases`, `assess`, `plan`, `deploy`, `optimise`;
- it links A1 to A4 and every G source it uses;
- `static/config.js` has `portal: "fixtures"`;
- no forbidden claim from Section 2.2;
- the landing page has four `#use-cases` cards and links the new page.

`tests/api_boundary.rs`: `domain: "portal"` returns 501 with the exact detail; the unknown-domain message lists all three domains; `/config.js` renders `domains.portal` as `fixtures` by default and honours `API_MODE_PORTAL=live`.

`tests/test-deploy.sh`: `API_MODE_PORTAL` is allowlisted and rejected unless `live` or `fixtures`.

### Task 2: Playwright

New `tests/e2e/admin-portal-migration.spec.ts`, at `/`, `/sub/path/` and on the Rust service, desktop and mobile:

- the five examples return Proceed, Ask, Escalate, Refuse and No data; "hello" returns Ask; typing `select2` returns Proceed;
- the inventory shows 61 rows and the counts Proceed 37, Ask 2, Escalate 22; each filter changes the rows and the `aria-live` count line;
- the result is brought on screen without moving focus;
- on the Rust service with default settings, the page runs from fixtures and makes no `/api/pipeline` call; with `domains.portal = "live"` and the route returning 500, it shows "Service unavailable" and no result;
- no horizontal overflow at 390 px; keyboard operation and visible focus; no console errors or failed requests;
- print shows all four phases, the counts and the source list.

Add the page to `PAGES` in `tests/e2e/helpers.ts` and to the reveal checks in `tests/e2e/presentation.spec.ts`. The link crawl in `links.spec.ts` then covers it.

### Task 3: Implement the page, fixture, server domain and landing card

### Task 4: Docs, screenshots and verification

Run the Google Cloud handover's Section 10 commands. Attach screenshots at 1280 px and 390 px. Complete its four review gates (factual, accessibility, security, visual) before merge.

## 9. Acceptance criteria

- The page follows the four phases in order, each with its Google source and a visible state label.
- All 61 AdminLTE 3.2.0 runtime dependencies appear, with counts computed by code and matching Appendix A.
- The five examples return their Section 4.3 outcomes in fixtures mode at `/`, at a sub-path and on the Rust service.
- No wording says or implies that Migration Center, ADK, Gemini, Jev or any Google Cloud migration tool processed this portal.
- No AdminLTE or ColorlibHQ file, logo or screenshot is in `static/`. The page makes no request to another host.
- Every number links to an A source or is computed in the page from data that does, or is labelled "illustrative".
- `AGENTS.md` rules hold: British English, no em dashes, relative paths, `fetch` only in `static/js/api.js`, "Service unavailable" on live failure, 44 px targets, outcome shapes plus text.

## 10. Open points for the owner

1. **Google's wording is unchecked.** `docs.cloud.google.com` is blocked in this cloud session. Either allow it in the environment's network settings, or the implementer checks Section 3 and the G sources from a machine that can reach it. No Google claim is published until then.
2. **The static check rejects the Optimise source.** `tests/static-checks.sh` greps every `.html` file for `optimiz`, and G6's URL contains "optimizing". Recommendation: narrow the check so it ignores `href` values and still checks all visible text. The alternative, citing only G2 for the Optimise phase, drops the phase's primary source.
3. **Fictional owner.** This plan reuses the Environmental Agency so the publication keeps one fictional organisation. The alternative is a new fictional name.
4. **Two classification calls for the factual reviewer.** `jquery-ui-dist` is classed Proceed because the guide maps jQuery UI sortable to SortableJS, but the package includes other widgets. `overlayscrollbars` is classed Proceed (native scrollbar CSS) as the guide says, although AdminLTE 4's own demo pages still load OverlayScrollbars 2 from a CDN.
5. **Branch.** `AGENTS.md` rule 12 names `gcp-acceleration-story`, which PR #6 merged. This plan restarts that branch from `main`. The lead may prefer a new branch name and an updated rule 12.

## Appendix A. AdminLTE 3.2.0 runtime dependencies, classified

Source: the `dependencies` block of A2 (61 entries). Classified against the replacement table, class renames, icon section and jQuery section of A4. Rule: named with one replacement, or kept, is **Proceed**; named with two alternatives is **Ask**; not named is **Escalate**. Recount if either source changes.

**Proceed (37)**

| Package | AdminLTE 4 replacement |
|---|---|
| `bootstrap` | Bootstrap 5.3, with the class and `data-*` renames |
| `jquery` | Removed, last, once no calls remain |
| `@fortawesome/fontawesome-free` | Bootstrap Icons |
| `select2`, `@ttskch/select2-bootstrap4-theme`, `bootstrap4-duallistbox` | Tom Select |
| `datatables.net` and its 14 Bootstrap 4 packages (`datatables.net-bs4` and 13 `datatables.net-*-bs4` extensions), `jsgrid` | Tabulator |
| `daterangepicker`, `tempusdominus-bootstrap-4` | Flatpickr |
| `summernote` | Quill |
| `bootstrap-slider` | noUiSlider |
| `icheck-bootstrap` | Native `.form-check` |
| `inputmask` | IMask |
| `jquery-knob-chif` | Chart.js 4 doughnut |
| `jquery-mapael` | jsVectorMap |
| `jquery-ui-dist` | SortableJS, for jQuery UI sortable (open point 4) |
| `fullcalendar` | FullCalendar 7 |
| `chart.js` | Chart.js 4 |
| `ekko-lightbox` | GLightbox |
| `overlayscrollbars` | Native scrollbar CSS (open point 4) |
| `toastr` | Bootstrap toasts |
| `sweetalert2` | SweetAlert2, kept; jQuery-free since version 7 |

**Ask (2)**

| Package | Documented alternatives |
|---|---|
| `dropzone` | Dropzone 6 or FilePond |
| `bootstrap-colorpicker` | Pickr or the native colour input |

**Escalate (22): not named in A4**

`@lgaitan/pace-progress`, `@sweetalert2/theme-bootstrap-4`, `bootstrap-switch`, `bs-custom-file-input`, `bs-stepper`, `codemirror`, `fastclick`, `filterizr`, `flag-icon-css`, `flot`, `fs-extra`, `ion-rangeslider`, `jquery-mousewheel`, `jquery-validation`, `jqvmap-novulnerability`, `jszip`, `moment`, `pdfmake`, `popper.js`, `raphael`, `sparklines`, `uplot`.
