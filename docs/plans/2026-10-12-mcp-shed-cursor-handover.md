# Handover: MCP Shed revamp (Cursor agent swarm)

**Read this whole file before writing code.** It is the single source of truth for the swarm. If something here conflicts with older docs in `docs/`, this file wins. `AGENTS.md` at the repo root holds the rules every agent must follow.

| | |
|---|---|
| Deadline | 2 days from kick-off |
| Repo | `graphs-and-jev` (rename later, not in scope) |
| Baseline | `origin/main` at `5475dd9`: pages already live under `static/`, served with the API by one Rust process on Cloud Run. No `demo-data.js`, no Pages workflow. |
| Stack | Pages: static HTML, CSS, vanilla JavaScript, SVG, no framework, no build step. Server: the existing Rust crate (`Cargo.toml`, `src/`). |
| Target | The existing Cloud Run service `graphs-and-jev` in `ninth-airship-386815`, `australia-southeast1`, released through `deploy/deploy.sh` (private candidate, verify, promote). See `docs/deployment/cloud-run.md`. |
| Branching | All work lands on branch `mcp-shed`. Each workstream works in its own git worktree on branch `mcp-shed-wsN` cut from `mcp-shed`; the lead agent merges each into `mcp-shed`. One pull request from `mcp-shed` to `main` at the end. Nothing merges to `main` before the final review. |
| Agents | Cursor subagents, one per workstream. Commits use the repository's `TLatAccenture` identity with no co-author trailers. |
| Reviewer | The lead agent reviews against Section 11 once the swarm reports done, then runs the deployment (Section 7, WS6). |

---

## 1. What we are building

Convert the "Graphs + Jev" field guide into a **thought piece** on agentic migration and modernisation, built around **the MCP shed**. It shows how we think. It is not a product, service offer or pricing page.

Graphs and Jev stay, as two of five pillars rather than the whole story.

### 1.1 Decisions already made (do not revisit)

1. **Thought piece**, Accenture-branded through config (Section 4). Footer states it is a thought piece, not a product or service offer.
2. **DCCEEW is removed everywhere.** The catalogue use case becomes a fictional **"Environmental Agency"**. Always write "Environmental Agency" (never "Environment Agency", which is a real UK body), and say once per page that it is fictional.
3. **Cloud Run, parameterised, on the existing Rust service.** The Rust process keeps serving `static/` and the API from one origin. Brand text and API settings come from environment variables, which the Rust service renders as `/config.js`. Deploying must mean: set environment variables, run `deploy/deploy.sh`. No code edits to switch between in-browser fixtures and the live API. The Parkinson's research path (live Wikipedia and Europe PMC retrieval) is removed from the server along with the page.
4. **Every number gets a link.** "RLCD" is replaced by **"System 1 model"** (TypeSafe's term is "System One model"; use "System 1" in our copy and cite theirs).
5. **Fifth pillar is "Visibility and cost".** No sixth pillar.
6. **Parkinson's is removed.** Its page shell is reused for the new lineage-migration page.

### 1.2 Site map (after)

```
static/
  index.html                      Landing: problem, the shed, five pillars, guided example, use cases, governance
  technical-guide/index.html      NEW: everything from the old #technical-guide section, organised by pillar, plus new content
  environmental-agency/index.html Was dcceew/. Catalogue discovery before migration (works with the existing Rust catalogue API)
  lineage-migration/index.html    NEW (from parkinsons/ shell). Lineage-guarded migration demo, fixtures only for now
  little-bunnies/index.html       Kept. Reframed to lineage context
  config.js                       Brand + API defaults for static serving; the Rust service answers /config.js from env vars instead
  js/api.js                       The only code that talks to an API or fixtures
  js/estate-graph.js              Shared sample graph (landing guided example + labs)
  fixtures/catalogue.js           Environmental Agency fixture (was demo-data.js, DCCEEW branch)
  fixtures/lineage.js             Lineage-migration fixture
  guide.css                       Shared design system (WS0 only)
  styles/*.css                    Per-page styles, one file per workstream
  assets/                         SVGs and engine logos
Cargo.toml, Cargo.lock, src/      Existing Rust server (Axum): static files, /config.js, /api/*
catalogue.json                    Packaged catalogue for the Rust catalogue domain
Dockerfile                        Existing scratch image with a static musl binary (keep)
deploy/                           Existing guarded Cloud Run release script and env example
docs/api-contract.md              Request/response contract for a live backend
docs/deployment/cloud-run.md      Existing deployment record (updated after release)
tests/*.rs                        Rust API and static-frontend contract tests
tests/e2e/, tests/static-checks.sh Playwright end-to-end tests and static checks
```

Deleted: `static/parkinsons/`, `static/dcceew/`, `static/assets/parkinsons-research.svg`, `src/research.rs` and the `parkinsons` pipeline domain, `tests/browser_dcceew_race.py` (ported to Playwright). `static/assets/dcceew-landscape.svg` is renamed to `static/assets/environmental-agency-landscape.svg` (content unchanged unless WS3 redraws it). There is no `demo-data.js` on `main`; WS0 recovers the DCCEEW fixture logic from history with `git show 95763d0:demo-data.js`.

---

## 2. Execution plan and file ownership

### 2.1 Order

```
Day 1 morning     WS0 Foundation (one agent, blocking). Merge before anything else starts.
Day 1 afternoon   WS1 to WS6 in parallel (six agents).
Day 2 morning     WS1 to WS6 finish; WS7 writes tests against Section 10 in parallel.
Day 2 afternoon   WS7 runs everything, files defects to owners; owners fix; the lead agent reviews.
Day 2 evening     Lead agent releases through deploy/deploy.sh: private candidate, verify, promote.
```

### 2.2 Ownership (hard rule: only touch files you own)

| WS | Owner of | Must not touch |
|---|---|---|
| WS0 Foundation | Directory moves inside `static/`, `static/config.js`, `static/js/api.js`, `static/js/estate-graph.js`, `static/fixtures/*.js` (initial versions), `static/guide.css`, header/footer markup on every page (initial pass) | Page body copy beyond stubs, anything outside `static/` |
| WS1 Landing | `static/index.html` (everything inside `<main>`), `static/styles/landing.css`, `static/assets/mcp-shed-stack.svg`, `static/assets/hero-shed.svg` (optional) | `technical-guide/`, `js/`, `guide.css` |
| WS2 Technical guide | `static/technical-guide/index.html`, `static/styles/tech.css`, `static/js/labs.js`, `scripts/dense.py` | `index.html`, `js/estate-graph.js` (request changes from WS0) |
| WS3 Environmental Agency | `static/environmental-agency/index.html`, `static/fixtures/catalogue.js`, `static/styles/environmental-agency.css`, `static/assets/environmental-agency-landscape.svg` | Other pages |
| WS4 Lineage migration | `static/lineage-migration/index.html`, `static/fixtures/lineage.js`, `static/styles/lineage.css`, `static/assets/lineage-blast-radius.svg` | Other pages |
| WS5 Little Bunnies + docs | `static/little-bunnies/index.html`, `README.md`, `docs/design.md`, `docs/plans/2026-10-12-mcp-shed-revamp.md` (status line only) | Other pages |
| WS6 Server and deployment | `Cargo.toml`, `Cargo.lock`, `src/**`, `catalogue.json`, `Dockerfile`, `.dockerignore`, `.gcloudignore`, `deploy/**`, `scripts/smoke-container.sh`, `tests/*.rs`, `tests/test-deploy.sh`, `docs/api-contract.md` | `static/**` except reading. Do not re-add a Pages workflow. |
| WS7 Verification | `tests/e2e/**`, `tests/static-checks.sh`, `tests/browser_dcceew_race.py` (delete after porting), `package.json`, `package-lock.json`, `playwright.config.ts` | Everything else (files defects instead) |
| Lead agent | `AGENTS.md`, this handover, `docs/deployment/cloud-run.md`, merges into `mcp-shed`, the release | |

Each page's owner also owns that page's `<head>` text: the `<title>` page title and `<meta name="description">` (rewrite every description for the new framing). WS0 only adds the `config.js` and `js/api.js` script tags and the `data-page-title` attribute.

If you need a change in a file you do not own, write it in your PR description under "Requests for other workstreams". Do not make it yourself.

---

## 3. Writing rules (all copy, all agents)

1. **British English** (organise, modernise, colour in prose, catalogue, programme, licence as a noun). CSS property names are exempt.
2. **No em dashes** (the character `—`) anywhere in HTML, Markdown or JS strings. Use a colon, a comma, brackets or a new sentence. The existing pages contain some; remove them.
3. **Plain and factual.** No jokes, no rhetorical flourishes, no "revolutionise", "unlock", "supercharge", "game-changing", "seamless", "cutting-edge". Short sentences.
4. **Every number or factual claim about a third party gets a link** from Section 8. If a claim has no link in Section 8, either find a primary source and add it to your PR description, or remove the claim.
5. **Vendor figures are labelled "vendor-reported".** Our own illustrative numbers are labelled "illustrative".
6. **Never invent figures** (latencies, prices, token counts, adoption numbers). Where a lab needs an assumption, make it a visible, editable parameter labelled "assumption".
7. Accessibility patterns already in the pages stay: `aria-live` status lines, `aria-pressed` buttons, labelled SVGs, skip link, 44px touch targets, outcome shapes plus text labels (never colour alone).
8. Terms, used consistently:
   - **MCP shed**: the governed, curated set of MCP servers available to migration agents. Lower case "shed" in running text; "The MCP Shed" only as a title.
   - **System 1 model**: a small, fast model that returns typed decisions with calibrated confidence. **Jev** is TypeSafe's System 1 model and is our example. First use per page: "Jev, TypeSafe's System 1 decision model".
   - **Skill**: a packaged, prescriptive playbook an agent loads for a task. **Hook**: a check that runs automatically before or after a tool call and can block it.
   - **Lineage**: which data and code feed which other data, code and reports.

---

## 4. Configuration contract (WS0 implements, everyone uses)

### 4.1 `static/config.js`

This file is the **only** place brand text and API settings live in `static/`. When the pages are served by plain static hosting (for example `python3 -m http.server`), this file is used as is. When they are served by the Rust service, the Rust service answers `GET /config.js` itself, rendering the same object from environment variables (WS6, Section 7), so the file on disk is never edited per environment. Default contents:

```js
/* Defaults for static serving. The Rust service renders /config.js from env vars. Keep keys stable. */
window.SITE_CONFIG = {
  brand: {
    org: "Accenture",
    name: "The MCP Shed",
    tagline: "Agentic migration and modernisation",
    kind: "Thought piece",
    disclaimer: "A personal research thought piece on agentic migration and modernisation. It is not an official Accenture or Google Cloud product, a service offer or a statement of capability. Product names and marks belong to their owners."
  },
  api: {
    mode: "fixtures",            // "fixtures" | "live"
    baseUrl: "",                 // "" means same origin. e.g. "https://api.example.run.app"
    timeoutMs: 15000,
    routes: {
      health: "/api/health",
      catalogue: "/api/catalogue",
      pipeline: "/api/pipeline"
    },
    domains: {                   // per-domain override; omit a key to inherit api.mode
      catalogue: "fixtures",
      lineage: "fixtures"
    }
  }
};
```

### 4.2 Brand binding

- Every page includes, in `<head>` order: `config.js`, then `js/api.js` (relative paths, see 4.4).
- Elements with `data-brand="org|name|tagline|kind|disclaimer"` get their `textContent` replaced from `SITE_CONFIG.brand` on `DOMContentLoaded`. The HTML ships with the default text already inside those elements so the page reads correctly with JavaScript off.
- `<title>` pattern: `{Page title} · {brand.name}`; landing page: `{brand.name}: {brand.tagline}`. `js/api.js` sets `document.title` from `data-page-title` on `<body>` if present.

### 4.3 `static/js/api.js`

Exposes one global, `window.siteApi`, with:

```js
siteApi.catalogue()                                   // -> Promise<Response-like>
siteApi.pipeline({ domain, request, preference })     // domain: "catalogue" | "lineage"
siteApi.health()
siteApi.mode(domain)                                  // -> "fixtures" | "live"
```

Rules:
- In `fixtures` mode, call `window.SITE_FIXTURES[domain](payload)`, registered by `fixtures/*.js`. Fixtures return a Response-like object `{ok, status, json()}` exactly as the old `demo-data.js` did (`git show 95763d0:demo-data.js`).
- In `live` mode, `health()` and `catalogue()` are `fetch(baseUrl + route, { method: "GET", signal, credentials: "omit" })`. `pipeline()` is `fetch(baseUrl + routes.pipeline, { method: "POST", headers: {"Content-Type": "application/json"}, body: JSON.stringify({domain, request, preference}), signal, credentials: "omit" })`. Every call has an `AbortController` timeout of `timeoutMs`. No credentials, no cookies.
- Same-origin API routes are absolute from the site root (`/api/...`) because the API lives at the origin root, not beside the page. This is the one exception to Section 4.4.
- **Live failures never fall back to fixtures.** Return `{ok:false, status, json: async () => ({outcome:"error", detail})}` so the page shows "Service unavailable". Never fabricate a result.
- Pages show the active mode in the demo's technical details: "Decided by: in-browser fixtures" or "Decided by: live API at {host}".
- No page may call `fetch` directly or reference `demo-data.js`.

### 4.4 Paths

- **All internal links and asset URLs are relative.** Nothing may contain `/graphs-and-jev/`. From a subpage: `../guide.css`, `../config.js`, `../#use-cases`. From the landing page: `guide.css`, `technical-guide/`.
- The site must work unchanged when served at `/` (Cloud Run) and, in fixtures mode, at `/any/sub/path/` (any static host). WS7 tests both. GitHub Pages is already retired; do not re-add it.

---

## 5. Shared copy (use verbatim unless marked "draft"; improve wording only if it stays within Section 3)

> **Owner edit (12 Oct):** the landing copy in `static/index.html` was tightened at the owner's request. Where it differs from this section, the page wins. Keep new copy equally direct: short headings, one idea per sentence.

### 5.1 Core message

**Hero eyebrow:** Agentic migration and modernisation

**Hero H1:** Give migration agents the right tools *for the job.* (the italic span keeps the existing purple `em` style)

**Hero lede:** Most enterprise estates grew over decades: storage in AWS and Azure, platforms such as SAP, Salesforce, Databricks, ServiceNow and Atlassian, and years of Word, Excel and PowerPoint files in Microsoft 365. Migrating that estate with agents needs a different approach, one that counts what the work costs today and what the result will cost to run tomorrow. We call that approach the MCP shed.

**Problem (section copy):** Agents need the right tools for the job. Enterprises rarely have one system of record. They have many, each with its own interface, credentials and quirks, and the knowledge of how they connect often lives in people's heads or in one-off scripts. An agent dropped into that estate either sees too little to act safely, or is handed so many tools and so much context that it slows down, costs more and makes confident mistakes.

**Solution (section copy):** The MCP shed is a governed, curated set of MCP servers that gives each migration agent the tools its task needs, and nothing more. Around it sit four further layers: fast decision models that gate each step, prescriptive playbooks, a graph of lineage and meaning, and measurement that shows what the agents did and what it cost.

**Why now (section copy):** Context windows keep growing [L3], but well-architected agents still need careful context management: models use information in the middle of long prompts less well [L4], and tool definitions and results consume the window [L5][L6]. The Model Context Protocol has become the common standard for connecting agents to tools [L7]: a shared language for networked function calling, much as APIs were for the previous generation of systems. It has its critics, but adoption has largely settled the argument. Stripe's in-house coding agents, Minions, now merge more than a thousand pull requests a week [L1], drawing on a central MCP server reported to expose over 400 tools [L2]. We expect the same shift in enterprise migration and modernisation.

### 5.2 The five pillars (stack order, bottom to top; render as a layered stack, bottom layer first in reading order on mobile)

| # | Pillar | One line | Detail |
|---|---|---|---|
| 1 | **Connectivity** | Standard, governed MCP servers replace brittle one-off scripts. | Each system in the estate is reached through an MCP server with scoped, short-lived credentials and an audit trail. Agents receive only the servers their task needs, which keeps tool definitions out of the context window and secrets out of scripts. |
| 2 | **Relational context** | A graph holds lineage and the enterprise ontology. | Spanner Graph, or another graph engine, records which tables, jobs, files and reports feed which others, and what each business term means. Before an agent changes anything, it can see what sits upstream and downstream, so a change in one place does not silently break another. |
| 3 | **Logic** | Skills and hooks turn migration playbooks into enforced steps. | Skills give agents a prescriptive playbook for each migration pattern. Hooks run before and after tool calls to validate schemas, block unsafe actions and require approval where needed. Together they reduce drift, broken code and malformed schemas. |
| 4 | **Reflexes** | System 1 models route, classify and gate in milliseconds, not seconds. | A System 1 model such as Jev returns a typed decision with a calibrated confidence. TypeSafe reports end-to-end response times of 70 to 500 ms [L8], against seconds for a frontier model, fast enough to sit in front of every tool call. It does not make a model stop hallucinating; it stops an uncalibrated output from becoming an action. |
| 5 | **Visibility and cost** | Evals show what agents did; BigQuery and Looker show users the result and the cost. | Every agent step writes an eval record: decision, confidence, tools called, tokens, latency and outcome. Engineers use the records to find failure patterns. BigQuery and Looker turn them into production reporting for users, including cost per migrated object, so the spend today and the run cost tomorrow are both visible. |

### 5.3 Outcome vocabulary

Backend codes stay the same (the Rust API already uses them). UI labels change:

| Code | UI label (landing, lineage) | Shape | Meaning |
|---|---|---|---|
| `answer` | Proceed | circle, green | Evidence supports the step; it runs |
| `clarify` | Ask | diamond, amber | A detail is missing or ambiguous; ask, do not guess |
| `reject` | Refuse | triangle, red | Out of scope or unsafe; stops before any tool call |
| `review` | Escalate | diamond, amber | Confidence too low or a fact is missing; a person decides |
| `no_data` | No data | square, grey | The request is clear but nothing in the graph covers it |
| `error` | Service unavailable | triangle, red | Live API failed; nothing fabricated |

The Environmental Agency page may keep catalogue wording ("Answer: datasets found", "Clarify: more detail needed") because it answers questions rather than running steps.

### 5.4 Glossary (landing page `dl.defs`, replaces the current five entries)

- **MCP**: Model Context Protocol. An open standard for connecting AI applications to tools, resources and live systems [L7].
- **MCP server**: A service that exposes one system's tools and data to agents over MCP, for example a CMDB, a data warehouse or a document store.
- **MCP shed**: The governed set of MCP servers available to migration agents, with each agent given only what its task needs.
- **Skill**: A packaged playbook an agent loads for a specific task, such as migrating a table or converting a batch job.
- **Hook**: A check that runs automatically before or after a tool call and can block it.
- **System 1 model**: A small, fast model that returns a typed decision with a calibrated confidence instead of free text [L8]. Jev is an example.
- **Lineage graph**: Facts (nodes) and typed relationships (edges) recording which data, code and reports feed which others.
- **Eval record**: A structured log of one agent step: decision, confidence, tools, tokens, latency, outcome.

### 5.5 Governance band (replaces "Trust and deployment"; keep `.proof` layout)

Heading: **Built to show its working, and to say no.**
- **Least privilege**: Each agent gets only the MCP servers and scopes its task needs.
- **Refusal before tool call**: Unsafe requests, such as copying production customer data to a test environment, stop before any server is called.
- **Bounded outcomes**: Proceed, ask, refuse, escalate or no data. Nothing in between.
- **Traceable**: Every outcome returns the lineage path and the evidence behind it.
- **Measured**: Every step writes an eval record, including tokens and latency.
- **Portable**: Static pages and a documented API contract. The same pages run from in-browser fixtures or from a Rust service on Cloud Run that serves the pages and the API from one origin.

Scope note below the list: "The demonstrations use a small in-memory graph and deterministic rules. They are illustrative, not a production migration tool."

---

## 6. The shared sample estate graph (WS0 creates `static/js/estate-graph.js`)

Replaces the ACME graph. Exposes `window.ESTATE_GRAPH` with the same API as the current `G` object (`nodes`, `edges`, `node(id)`, `neighbours(id)`, `tokens(n)`). ViewBox `0 0 400 380`. Coordinates are a starting point; WS1 must adjust for legibility (no overlapping labels at 390px wide) and send final coordinates back to WS0.

| id | label | type | x | y | text |
|---|---|---|---|---|---|
| kna1 | SAP KNA1 | Table | 60 | 40 | KNA1 is the SAP customer master table. |
| region | KNA1.REGION | Column | 60 | 130 | KNA1.REGION holds each customer's sales region code. |
| bqcust | BigQuery customers | Table | 200 | 40 | The customers table in BigQuery is the migration target for KNA1. |
| bqreg | customers.region | Column | 200 | 130 | customers.region is the BigQuery column mapped from KNA1.REGION. |
| dbx | Databricks customer_sync | Job | 340 | 40 | customer_sync is a Databricks job that loads BigQuery customers nightly. |
| policy | Policy: production PII | Policy | 340 | 130 | Production customer data may not be copied outside production. |
| lkreg | Looker: Regional sales | Dashboard | 120 | 215 | The Regional sales dashboard reports revenue by region. |
| lkchurn | Looker: Monthly churn | Dashboard | 280 | 215 | The Monthly churn dashboard reports customer churn by region. |
| finance | Finance | Team | 360 | 290 | Finance owns the regional sales and churn dashboards. |
| crm | Legacy CRM sync | Job | 40 | 300 | Legacy CRM sync is a scheduled job registered in ServiceNow. |
| sfacct | Salesforce Account | Object | 140 | 300 | Salesforce Account holds customer accounts for the sales team. |
| regret | Regulatory return | Report | 250 | 345 | The quarterly regulatory return draws on Salesforce account data. |
| xlsx3 | Pricing FY26 v3.xlsx | File | 120 | 360 | Pricing FY26 v3.xlsx is a SharePoint workbook of list prices. |
| xlsxf | Pricing FY26 final.xlsx | File | 300 | 360 | Pricing FY26 final.xlsx is a SharePoint workbook of list prices. |
| price | Price list | Entity | 210 | 290 | The price list sets list prices applied to Salesforce quotes. |

Edges (from, REL, to):
```
kna1 HAS_COLUMN region      bqcust HAS_COLUMN bqreg     region MAPS_TO bqreg
dbx WRITES bqcust           bqreg USED_BY lkreg          bqreg USED_BY lkchurn
finance OWNS lkreg          finance OWNS lkchurn         policy GOVERNS bqcust
crm FEEDS sfacct            sfacct FEEDS regret          kna1 SYNCS_TO sfacct
xlsx3 DESCRIBES price       xlsxf DESCRIBES price        price APPLIES_TO sfacct
```

Type colours: map new types onto the existing palette variables (`Table` and `Column` purple shades, `Job` purple-mid, `Dashboard` and `Report` black, `File` grey, `Policy` reject red, `Team` purple-deep, `Object` and `Entity` purple). Keep `.node`/`.edge` CSS.

### 6.1 Guided example scenarios (WS1)

| Button | Start | Hops | Path | Outcome | Answer text | Why text |
|---|---|---|---|---|---|---|
| Which reports break if we drop KNA1.REGION? | region | 2 | region MAPS_TO bqreg; bqreg USED_BY lkreg; bqreg USED_BY lkchurn | answer (Proceed) | Two Looker dashboards, Regional sales and Monthly churn, read the BigQuery column mapped from KNA1.REGION. Both are owned by Finance. | Two hops from the column reach every report that depends on it. The change can proceed once Finance is told, with the blast radius attached. |
| Migrate the pricing workbook to BigQuery. | price | 1 | xlsx3 DESCRIBES price; xlsxf DESCRIBES price | clarify (Ask) | Two workbooks describe the same price list: Pricing FY26 v3.xlsx and Pricing FY26 final.xlsx. | The graph has no source-of-truth edge between them. Picking one would be a guess, so the agent asks which is authoritative. |
| Retire the legacy CRM sync job. | crm | 2 | crm FEEDS sfacct; sfacct FEEDS regret | review (Escalate) | The job feeds Salesforce Account, which feeds the quarterly regulatory return. The return has no recorded owner. | The dependency is known; the person accountable for it is not. A person decides. |
| Copy production customer records to the dev sandbox. | bqcust | 1 | policy GOVERNS bqcust | reject (Refuse) | Refused. Production customer data may not be copied outside production. | The policy edge applies before any MCP server is called. Nothing was read or copied. |

Hop-slider message when depth is below the scenario's hops: "The evidence path needs more hops than this, so the agent would not proceed."

---

## 7. Workstream briefs

### WS0 Foundation (blocking, Day 1 morning)

1. The pages are already under `static/`. `git mv static/dcceew static/environmental-agency` and `git mv static/parkinsons static/lineage-migration`. Rename `static/assets/dcceew-landscape.svg` → `static/assets/environmental-agency-landscape.svg`. Delete `static/assets/parkinsons-research.svg` and the root `.nojekyll`.
2. Create `static/technical-guide/index.html` as a stub with the shared header/footer and an empty `<main>`; move the entire `<section id="technical-guide">` block and all its scripts (engine data, charts, labs, snippets, local runs) out of `static/index.html` into it unchanged, so WS2 starts from working code. Fix `assets/engine-logos/` references to `../assets/engine-logos/`.
3. Create `config.js`, `js/api.js`, `js/estate-graph.js` per Sections 4 and 6. Recover the old in-browser fixture with `git show 95763d0:demo-data.js` (read only; do not restore the file). Its DCCEEW branch (`dcceewResult`) becomes `fixtures/catalogue.js` registering `SITE_FIXTURES.catalogue`, plus a fixture for `siteApi.catalogue()` built from the same data. Check its four example outcomes match what the live Rust catalogue domain returns today; report any difference rather than changing either side. Create `fixtures/lineage.js` registering a stub returning `{outcome:"no_data", message:"Fixture not built yet."}`. Point the Environmental Agency page's existing `fetch` calls at `siteApi` so it works in both modes (WS3 refines the page afterwards).
4. Rewrite every internal URL to relative (Section 4.4).
5. Apply the shared header and footer (below) to every page.
6. `guide.css`: add `--connect`, `--context`, `--logic`, `--reflex`, `--visibility` pillar colour tokens (choose from the existing palette so contrast stays AA; Accenture purple remains the identity colour), plus a `.stack` component for the five-layer pillar diagram (WS1 fills content). Change nothing else.
7. Commit, then announce "WS0 merged" so WS1 to WS6 start.

Shared header:
```html
<header class="site-header"><div class="in">
  <a class="brand" href="{rel}"><span data-brand="name">The MCP Shed</span></a>
  <nav class="site-nav" aria-label="Primary">
    <a href="{rel}#pillars">Approach</a>
    <a href="{rel}#use-cases">Use cases</a>
    <a href="{rel}technical-guide/">Technical guide</a>
    <a class="cta" href="{demo anchor}">Try the demo</a>
  </nav>
</div></header>
```
Shared footer: brand line (`<span data-brand="org">Accenture</span> · <span data-brand="kind">Thought piece</span>`), the disclaimer (`data-brand="disclaimer"`), and links to the three use-case pages and the technical guide. No Palindrome credit line (the value-story work removed it deliberately).

**WS0 outcome (merged).** `siteApi` adds `siteApi.source(domain)`, returning "in-browser fixtures" or "live API at {host}" for the technical details line. Live non-2xx responses become `{ok:false, status, json → {outcome:"error", detail}}`. Subpage breadcrumbs and body copy still carry old text for their owners to replace.

**Done when:** every page loads from `static/` with no console errors at `/` and at a sub-path (`python3 -m http.server` from `static/` and from its parent); `grep -r "/graphs-and-jev/" static` is empty; the Environmental Agency demo works through `siteApi` in fixtures mode. Rust `tests/static_frontend.rs` is expected to fail after WS0 (it asserts the old pages); WS6 rewrites it, so do not edit it.

### WS1 Landing page

Sections in order (ids are fixed, other pages link to them):
1. `#top` hero (5.1). Hero art: either a new `assets/hero-shed.svg` (a shed of labelled tool slots, a few lit for one agent's task) or the stack SVG. Original artwork only, no vendor logos.
2. `#problem` problem and why now (5.1), with inline citation links.
3. `#pillars` the five-layer stack (5.2): an SVG or HTML/CSS stack (`assets/mcp-shed-stack.svg` or `.stack`), each layer clickable, opening its detail text and a "Go deeper" link to the matching technical-guide anchor (Section WS2). Glossary (5.4) below.
4. `#example` guided example on `ESTATE_GRAPH` with the four scenarios (6.1). Reuse the existing rendering code; change data and labels only. Intro: "Choose a migration request. The graph highlights the lineage that bears on it, and the result shows what the System 1 gate decides and why. This runs in your browser on a 15-fact sample estate; no model or database is called."
5. `#use-cases` three feature cards: Environmental Agency, Lineage-guarded migration, Little Bunnies. Each card: business question, what the graph adds, what the gate controls, button.
6. `#governance` (5.5).
7. A short closing band linking to the technical guide.

Remove the Spanner spotlight, engine grades, labs, snippets and local runs from the landing page (WS0 moves them; WS1 deletes any leftovers).

Baseline: `mcp-shed` already contains the migration-first homepage from branch `design/value-story` (see `docs/plans/2026-10-12-migration-first-homepage*.md`). Reuse its structure, components and CSS wherever they serve the sections above. Where its copy or section order differs from Sections 5 and 6, this handover wins. `static/guide.css` gained value-story styles in that merge; WS1 moves any landing-only rules it needs into `styles/landing.css` through a request to WS0 rather than editing `guide.css`.

**Done when:** all four scenarios produce their outcomes in Section 6.1; layer clicks work by mouse and keyboard; no horizontal scroll at 390px; Lighthouse accessibility 95 or higher.

### WS2 Technical guide (`technical-guide/index.html`)

Page title: "Technical guide". Intro: "For technical advisers: how each pillar works, with runnable labs, graph-engine grades and costs, and the sources behind every figure." Sticky in-page nav with five anchors: `#connectivity`, `#relational-context`, `#logic`, `#reflexes`, `#visibility-and-cost`.

| Anchor | Content |
|---|---|
| `#connectivity` | **MCP shed catalogue table.** Rows: AWS (S3), Azure (Blob Storage), Google Cloud (BigQuery), SAP, Salesforce, Databricks, ServiceNow, Atlassian (Jira, Confluence), Microsoft 365 (SharePoint, OneDrive, Word/Excel/PowerPoint files), Looker. Columns: official MCP server (vendor / community / none found), remote or local, auth model, typical read and write scope, link, date checked. **Only list a server with a working official URL you have opened.** Otherwise write "No official server found as of {date}". Text names only, no SaaS logos. Below it, **Lab 4: tool scoping** (new, see below). |
| `#relational-context` | The Spanner Graph spotlight (unchanged copy, but replace "GCP event spotlight" eyebrow with "Spotlight"), engine grades and scores, entry production cost, Lab 1 and Lab 2 switched to `ESTATE_GRAPH`, "One question, five dialects" switched to a lineage query, "Run it on your laptop". |
| `#logic` | One annotated example: a short `SKILL.md` for "migrate a table from SAP to BigQuery" (steps: read lineage, generate DDL, generate view aliases for renamed columns, validate against target schema, write eval record) and one pre-tool-call hook (pseudo-code: block any write tool whose target environment is production unless an approval token is present; validate generated DDL against the target schema). Mark both "illustrative". |
| `#reflexes` | What a System 1 model is (cite [L8], [L9]). Latency statement exactly as 5.2 pillar 4, with "vendor-reported" and the independent caveat: "An independent review measured a 524 ms median for a hosted round trip [L10]." Then Lab 3 with actions renamed: "Approve schema change" (error cost 50), "Route migration ticket" (2), "Flag for review" (5). |
| `#visibility-and-cost` | Eval record schema as a JSON example (fields: `run_id`, `step`, `agent`, `skill`, `tools_called[]`, `decision`, `confidence`, `outcome`, `tokens_in`, `tokens_out`, `latency_ms`, `cost_usd`, `object_id`, `timestamp`). A BigQuery table definition for it (DDL, partitioned by date) and three example Looker tiles described in words: cost per migrated object, outcomes by step, escalation rate by skill. Link [L11], [L12]. No invented cost figures. |
| Sources | Merge the existing "Sources and caveats" with Section 8 links used on this page. Re-check the price dates; if you cannot re-check, keep the original dates exactly. |

**Lab 1 changes:** preset questions become: "Which dashboards use KNA1.REGION?", "What feeds the regulatory return?", "Which workbook is the source of the price list?", "What does customer_sync write?". The precomputed `DENSE` table is invalid for the new texts. Recompute it with `scripts/dense.py` (sentence-transformers `all-MiniLM-L6-v2`, cosine similarity of each node `text` to each preset question, rounded to 3 dp). If the model cannot be run in your environment, remove `DENSE`, make the lab BM25-only, and update the lab note text to say so. Do not hand-write similarity numbers.

**Lab 2 changes:** default seed `bqcust`. Replace "Jev takes about 64,000 tokens a request, of which about 32,000 for the state and the longest question" with a sourced figure from TypeSafe docs [L9] or delete the sentence.

**Lab 4 (new): tool scoping.** Deterministic, no network. Inputs (all editable, all labelled "assumption"): number of MCP servers in the shed (default 11), tools per server (default 36), tokens per tool definition (default 150), servers the task needs (default 2). Output: tokens spent on tool definitions with the whole shed loaded versus a routed subset, and the share of a context window (editable, default 200,000 tokens) each uses. Note text: "Loading every tool definition into every request costs context before the agent has done anything [L5]. A System 1 router that picks the servers for the task keeps that cost proportional to the task." Code must be viewable and editable like Labs 1 to 3.

Five-dialects query (lineage): "Which dashboards use KNA1.REGION?" Cypher reference:
```
MATCH (:Column {name: 'KNA1.REGION'})-[:MAPS_TO]->(:Column)-[:USED_BY]->(d:Dashboard)
RETURN d.name;
```
Write Gremlin, GQL (`GRAPH LineageGraph`), MongoDB `$graphLookup` and Kuzu Python equivalents in the same style as today.

**Done when:** all four labs run, edit and reset; every number on the page has a link or an "assumption"/"illustrative" label.

### WS3 Environmental Agency (`environmental-agency/index.html`)

Keep the page structure (gap, reach, before any request, connect, decide, try it, scope, sources). It already explains RAG versus MCP and object-storage sifting, which is the Connectivity argument.

Changes:
- Title "Environmental Agency catalogue discovery". Eyebrow "Use case · Discover the estate before you migrate it".
- H1: "Know what you hold before you move it."
- Lede (draft): "Before an agency's data estate can be migrated, someone has to establish what exists, where it lives and what depends on it. This demonstration uses the air-quality catalogue of a fictional Environmental Agency. The evidence is spread across documents, tables, catalogue records and reference data. First it has to be brought within reach through the MCP shed. Then it has to be connected in a graph. Then a System 1 model decides whether it is enough to answer."
- Replace every "DCCEEW" and the full department name. Replace "Jev" first mention per 3.8; keep "Jev" thereafter.
- Remove all em dashes (there are several in the `.fitgrid`, `.ask` and source list).
- Demo calls `siteApi.pipeline({domain: "catalogue", request, preference})` and `siteApi.catalogue()`. Keep the existing response handling (it matches the Rust API), including the guard against stale results when requests overlap.
- `fixtures/catalogue.js`: make it mirror the live Rust catalogue domain, not the old `dcceewResult` approximations. The lead agent captured the Rust responses for the four examples in `/tmp/mcp-shed/rust-pipeline-1.json` to `-4.json` and `/tmp/mcp-shed/rust-catalogue.json`. The fixture returns the same outcomes, datasets, cell counts and messages for those four requests, with `backend_label` "In-browser catalogue rules" and `residency` "browser" so the mode stays visible. Other requests keep a reasonable rule-based fallback. Do not run cargo.
- Add to "Added in production": "An MCP shed in front of the catalogue, object storage and source systems, with delegated, short-lived access."
- Landscape SVG: rename only, unless time allows a redraw. It must not contain any department name.

**Done when:** the four examples return answer, answer, clarify, reject exactly as today; the page works in live mode against the contract (WS7 mocks it).

### WS4 Lineage-guarded migration (`lineage-migration/index.html`)

Built on the Parkinson's page shell (hero, brief band, demo grid, result panel, aside). Replace the red clinical banner with a change-safety banner: "**Illustrative, not a migration tool.** The demo runs on a sample estate in your browser. It never connects to a real system."

- Title "Lineage-guarded migration". Eyebrow "Use case · Change without breaking downstream". H1: "See the blast radius before the change runs."
- Brief band: Business question ("Can this migration step run without breaking anything downstream?"); Graph contribution ("Computes the blast radius: every table, job and report that depends on the object being changed."); Playbook and hooks ("A migration skill sets the steps; hooks validate the schema and block unsafe writes."); System 1 gate ("Proceeds, asks, refuses or escalates at each step, with a calibrated confidence.").
- Demo: request textarea plus five example buttons. Result panel shows: outcome, message, **stages trail** (gate → lineage → playbook → hook check → eval record), **blast radius** as evidence paths, **playbook steps** with status (done / skipped / blocked), **eval record** as a small table, technical details with raw JSON.
- `fixtures/lineage.js` rules (match on lower-cased request text; evaluate in this order: the reject rule first, then rename, COBOL, drop, ZLEGACY, fallback):

| Example button text | Match | Outcome | Message | Notes |
|---|---|---|---|---|
| Rename KNA1.REGION to SALES_REGION in the BigQuery target | contains "rename" and "region" | answer | Safe to proceed with a compatibility view. Two dashboards read this column. | Blast radius: bqreg → lkreg, bqreg → lkchurn. Playbook: read lineage (done), generate DDL (done), create view alias `region` (done), validate schema (done), notify owner Finance (done). |
| Migrate COBOL job CUSTUPD to BigQuery SQL | contains "cobol" or "custupd" | clarify | Two versions of the CUSTREC copybook exist. Which one does production use? | Playbook stops after "read lineage". |
| Drop table KNA1 after cutover | contains "drop" | review | KNA1 syncs to Salesforce Account, which feeds the regulatory return. The return has no recorded owner. | Escalate. Path: kna1 SYNCS_TO sfacct; sfacct FEEDS regret. |
| Copy the production customer table to dev for testing | contains "prod" and ("copy" or "dev") | reject | Refused before any tool call. Production customer data may not leave production. | Stages: gate only. No graph, no playbook. |
| Migrate table ZLEGACY_TMP | contains "zlegacy" | no_data | ZLEGACY_TMP is not in the lineage graph. Discover it before migrating it. | Stages: gate, lineage. |
| anything else | | clarify | Name the object and the change, for example "rename KNA1.REGION". | |

Response shape: Section 9.3 `lineage` schema. Every fixture response sets `backend_label: "In-browser lineage rules"`, `model: "lineage-fixture-v1"`, `residency: "browser"`.

- Optional SVG `assets/lineage-blast-radius.svg` for the hero (original art, uses the sample estate labels).

**Done when:** all five examples return their outcomes; the free-text fallback works; no page text mentions Parkinson's, clinical, dose or prescribing.

### WS5 Little Bunnies and docs

`little-bunnies/index.html`: keep the experiment, numbers and methodology exactly. Change framing:
- Eyebrow "Experiment · Little Bunnies". H1 unchanged ("Two hops reach most of what matters.").
- Lede: "A migration agent rarely needs the whole lineage graph. In a small-world network, a short neighbourhood around the object being changed already holds most of the connected context, so the agent can stay inside a fixed budget without loading the estate."
- Brief band: Business question "How much of the lineage graph does an agent need to load before it can act safely?"; replace "Jev control" with "System 1 control: works within a fixed state budget. Two hops fill about 70% of it." Keep the 32,000-token figure only if WS2 finds a TypeSafe source [L9]; otherwise phrase as "an illustrative 32,000-token budget".

`README.md`: rewrite for the thought piece: what it is, the five pillars in one line each, page list, run locally (fixtures: `cd static && python3 -m http.server 8000`; full service: `cargo run`, then open `http://127.0.0.1:8080/`), configure (`static/config.js` for static hosting, the env vars in WS6 for the Rust service), deploy (point to `docs/deployment/cloud-run.md` and `deploy/deploy.sh`), disclaimer from `SITE_CONFIG.brand.disclaimer`, "No reuse licence is granted yet."

`docs/design.md`: replace with a one-page summary of this handover's architecture (Sections 1.2, 4, 9).

`docs/plans/2026-10-12-mcp-shed-revamp.md`, `docs/plans/2026-10-12-migration-first-homepage-design.md`, `docs/plans/2026-10-12-migration-first-homepage.md`: change only the Status line (add one if missing) to "Superseded by 2026-10-12-mcp-shed-cursor-handover.md". `docs/implementation-plan.md`: replace with a pointer to the handover (it describes the retired DCCEEW and Parkinson's pages). WS5 owns these files.

### WS6 Server and deployment (existing Rust service on Cloud Run)

Goal: **set env vars, run the existing release script.** No source edits per environment. Start from the Rust crate, `Dockerfile` and `deploy/` already on `main`; read `docs/deployment/cloud-run.md` first. Keep the scratch image, static musl binary, numeric non-root user, request IDs, structured logs without request text, body and field limits, and the process-local rate limit exactly as they are unless a step below changes them.

Server changes (`src/**`, `Cargo.toml`, `Cargo.lock`):
1. **Remove Parkinson's.** Delete `src/research.rs`, the `parkinsons` pipeline domain, the `with_research` hook and its tests. Drop `reqwest` and any other dependency only that path used, so the service makes no outbound calls. Run `cargo audit` afterwards.
2. **Domains.** Make the `backend` field in `PipelineIn` optional (default `"catalogue"`): `siteApi` sends exactly `{domain, request, preference}`, which today is rejected with 422. `POST /api/pipeline` accepts `domain: "catalogue"` for the existing catalogue rules; keep `"air"` as an alias so older clients still work. `"lineage"` returns HTTP 501 with `{"detail": "Lineage domain not implemented on this service."}` (the page runs it from fixtures; Section 9). Any other value keeps the existing unknown-domain error, with the available list updated.
3. **`GET /config.js`.** Rendered by the server from the env vars below, serialised with `serde_json` (so quotes and newlines are escaped correctly), as `window.SITE_CONFIG = {...};` with `Content-Type: text/javascript; charset=utf-8` and `Cache-Control: no-cache`. This route takes precedence over `static/config.js`. Unset vars fall back to the Rust defaults in the table. Test it with a value containing quotes, a backslash and `</script>`.
4. **Headers.** Add `X-Content-Type-Options: nosniff`, `Referrer-Policy: strict-origin-when-cross-origin`, and a `Content-Security-Policy` allowing the Google Fonts origins, the inline scripts and styles the pages already use, and `connect-src 'self'` plus `API_BASE_URL` when it is set. `Cache-Control: no-cache` for `*.html`; long cache for `/assets/`.
5. **Health.** Keep `/api/health` and `/api/ready` (Cloud Run probes already use them). Do not add `/healthz`; Cloud Run reserves some paths ending in `z`.

Environment variables (Rust defaults; `static/config.js` keeps the Section 4.1 defaults):

| Var | Rust default | Maps to |
|---|---|---|
| `BRAND_ORG` | Accenture | `brand.org` |
| `BRAND_NAME` | The MCP Shed | `brand.name` |
| `BRAND_TAGLINE` | Agentic migration and modernisation | `brand.tagline` |
| `BRAND_KIND` | Thought piece | `brand.kind` |
| `BRAND_DISCLAIMER` | (Section 4.1 text) | `brand.disclaimer` |
| `API_MODE` | live | `api.mode` |
| `API_BASE_URL` | (empty, same origin) | `api.baseUrl` |
| `API_TIMEOUT_MS` | 15000 | `api.timeoutMs` |
| `API_ROUTE_HEALTH` / `API_ROUTE_CATALOGUE` / `API_ROUTE_PIPELINE` | /api/health, /api/catalogue, /api/pipeline | `api.routes.*` |
| `API_MODE_CATALOGUE` | inherit `API_MODE` | `api.domains.catalogue` |
| `API_MODE_LINEAGE` | fixtures | `api.domains.lineage` |

Tests (`tests/*.rs`): rewrite `tests/static_frontend.rs` to the new page set and contract: every page in Section 1.2 returns 200; no page references `parkinsons`, `dcceew` or `demo-data.js`; only `js/api.js` contains `fetch(`; internal links and assets are relative (replace the old root-path assertion); no secrets, analytics or project IDs in browser assets (keep the existing check). Update `tests/api_boundary.rs` for the domain changes, the removed research path and `/config.js` (defaults, overrides, escaping, headers). `cargo test` passes.

Container and release (`Dockerfile`, `.dockerignore`, `.gcloudignore`, `deploy/**`, `scripts/smoke-container.sh`, `tests/test-deploy.sh`):
- Keep the `Dockerfile` unless a removed dependency changes the build. Update `scripts/smoke-container.sh`: drop the live-retrieval check; add `/config.js` (reflects env vars passed to `docker run`) and a catalogue pipeline answer through `domain: "catalogue"`; keep the root-filesystem allowlist, static-ELF and UID checks.
- Extend `deploy/deploy.sh` so the env file can carry the app variables above and they are passed to the Cloud Run revision (for example via `--update-env-vars` built from an allowlist of exactly these keys). Keep every existing preflight, the private candidate, digest pinning and promotion flow. Update `deploy/cloud-run.env.example` and `tests/test-deploy.sh` to cover the new keys. Never put credentials in the env file.
- **Do not run `deploy/deploy.sh` against GCP.** The lead agent runs the release after the final review.

Keep `POST /api/decide` (existing route, covered by `tests/api_boundary.rs`) and document it in the contract. When rewriting `tests/static_frontend.rs`, keep every assertion from the merged value-story work that still holds for the new pages, and replace the rest; say in your report which ones you dropped and why.

`docs/api-contract.md`: write up Section 9 for a backend team, including CORS: if `API_BASE_URL` is a different origin, that API must return `Access-Control-Allow-Origin` for the site's origin and allow `GET`, and `POST` with `Content-Type: application/json`. Same origin (this Rust service) needs no CORS.

**Done when:** `cargo test` and `cargo audit` pass; `docker build --platform linux/amd64 -t graphs-and-jev:test .` succeeds; `IMAGE=graphs-and-jev:test scripts/smoke-container.sh` passes; `docker run -e PORT=8080 -e BRAND_NAME='Shed "test"' -p 8080:8080 graphs-and-jev:test` serves the site and `/config.js` shows the escaped value; `bash tests/test-deploy.sh` passes; WS7's live-mode tests pass against the container.

### WS7 Verification

Set up Playwright (`@playwright/test`, Chromium only) under `tests/e2e/`. Tests in Section 10. Run them against: (a) `python3 -m http.server` serving `static/` at `/` (fixtures), (b) the same files served under `/sub/path/` (fixtures), (c) the Rust service from `cargo run` (live catalogue, fixture lineage), (d) the WS6 container. Port the stale-result race check in `tests/browser_dcceew_race.py` to Playwright for the Environmental Agency page, then delete the Python file. Write `tests/static-checks.sh` exactly as Section 10. File defects as a list per owning workstream. Do not fix other workstreams' files.

---

## 8. Citation register (the only approved links for figures; add new ones in your PR description)

| Id | Claim | Source |
|---|---|---|
| L1 | Stripe Minions merge more than a thousand pull requests a week | https://stripe.dev/blog/minions-stripes-one-shot-end-to-end-coding-agents |
| L2 | Stripe's central MCP server is reported to expose over 400 tools (secondary; say "reported") | https://visdom-maturity-matrix.virtuslab.com/guides/infrastructure/toolshed-model-400-tools-behind-one-mcp-stripe |
| L3 | Long context windows (Gemini) | https://ai.google.dev/gemini-api/docs/long-context |
| L4 | Lost in the Middle | https://aclanthology.org/2024.tacl-1.9/ |
| L5 | Token cost of MCP tool definitions and results | https://www.anthropic.com/engineering/code-execution-with-mcp |
| L6 | Context engineering for agents | https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents |
| L7 | MCP specification | https://modelcontextprotocol.io/specification/2026-07-28/server |
| L8 | TypeSafe: System One models and Jev; 70 to 500 ms end-to-end; 40x to 200x faster on System One-shaped queries (vendor-reported) | https://typesafe.ai/blog/introducing-system-one-models-and-jev |
| L9 | TypeSafe documentation | https://docs.typesafe.ai/introduction |
| L10 | Independent review of Jev latency (524 ms hosted median) | https://hackernoon.com/inside-jev-how-decision-models-trade-text-generation-for-typed-calibrated-answers |
| L11 | BigQuery documentation | https://cloud.google.com/bigquery/docs |
| L12 | Looker documentation | https://cloud.google.com/looker/docs |
| L13 | Spanner Graph overview (existing) | https://cloud.google.com/spanner/docs/graph/overview |
| L14 | TypeSafe Models documentation: Jev context length, 64k tokens per request, 32k tokens for `state` plus the longest question (vendor-reported) | https://docs.typesafe.ai/models |

Do **not** write "sub-100 ms". The vendor's own range is 70 to 500 ms.

---

## 9. API contract (summary; WS6 writes the full `docs/api-contract.md`)

### 9.1 Routes

| Method | Route | Purpose |
|---|---|---|
| GET | `/api/health` | Liveness, `{"status":"ok"}` |
| GET | `/api/ready` | Readiness (catalogue and static directory loaded); Cloud Run startup probe |
| GET | `/config.js` | `window.SITE_CONFIG` rendered from env vars (Rust service only) |
| GET | `/api/catalogue` | `{"summary": string, "sources": [{"id": number, ...}]}` |
| POST | `/api/pipeline` | Body `{"domain": "catalogue" \| "lineage", "request": string (max 500), "preference": string \| null (max 300)}`. The Rust service answers `catalogue` (alias `air`) and returns 501 for `lineage` until a lineage backend exists. |

### 9.2 Common response fields

```json
{
  "outcome": "answer | clarify | reject | review | no_data",
  "message": "string, shown for non-answer outcomes",
  "explanation": {"text": "first line is the answer; later lines are reasons"},
  "stages": [{"stage": "gate", "title": "string"}],
  "backend_label": "string", "model": "string", "residency": "string",
  "totals": {"decisions": 0, "calls": 0, "wall_ms": 0}
}
```
Errors: non-2xx with `{"detail": "string"}`. The front end shows "Service unavailable" and never substitutes fixtures.

### 9.3 Domain-specific fields

- `catalogue`: unchanged from the current Rust API (`query`, `solutions[]` with `sources`, `cells`, `fit`, `notes`). The fixture in `fixtures/catalogue.js` returns the same shape.
- `lineage`:
```json
{
  "graph": {"edges": [{"from": "id", "from_label": "string", "ty": "MAPS_TO", "to": "id", "to_label": "string"}]},
  "playbook": {"skill": "string", "steps": [{"name": "string", "status": "done | skipped | blocked"}]},
  "eval": {"run_id": "string", "decision": "string", "confidence": 0.0, "tools_called": ["string"], "tokens_in": 0, "tokens_out": 0, "latency_ms": 0, "cost_usd": 0.0}
}
```
Stage ids for lineage: `gate`, `lineage`, `playbook`, `hook`, `eval`.

---

## 10. Acceptance tests (WS7 automates; owners run locally before reporting done)

Static checks (`tests/static-checks.sh`; run from the repo root; prints every violation and exits non-zero if there are any, zero otherwise):
```bash
#!/usr/bin/env bash
set -uo pipefail
fail=0
check() { local name="$1"; shift; local out; out="$("$@" 2>/dev/null)"; if [[ -n "$out" ]]; then printf 'FAIL %s\n%s\n' "$name" "$out"; fail=1; fi; }
check "sub-path prefix"   grep -rn "/graphs-and-jev/" static
check "removed topics"    grep -rniE "dcceew|department of climate|parkinson|levodopa|clinical|RLCD" static README.md docs/design.md src
check "em dash"           grep -rn "—" static README.md docs/design.md docs/api-contract.md
check "sub-100 claim"     grep -rn "sub-100" static README.md
check "old brand"         grep -rn "Graphs + Jev" static README.md docs/design.md
check "fetch outside api" bash -c 'grep -rln "fetch(" static | grep -v "^static/js/api.js$"'
check "US spelling"       bash -c 'grep -rniE "\b(organiz|moderniz|optimiz|behavior|catalog\b)" static --include=*.html | grep -v "behavior:"'
exit "$fail"
```

Playwright (each at `/` and `/sub/path/` in fixtures mode, and at `/` on the Rust service; desktop 1280px and mobile 390px):
1. Every page returns 200, has no console errors, no failed requests, no horizontal scroll at 390px.
2. Every internal link and asset resolves (crawl from the landing page).
3. Landing guided example: each of the four scenarios shows its outcome label from 5.3.
4. Landing pillars: each layer opens by click and by Enter; each "Go deeper" link lands on an existing technical-guide anchor.
5. Environmental Agency: the four examples return Answer, Answer, Clarify, Reject.
6. Lineage: the five examples return Proceed, Ask, Escalate, Refuse, No data; free text "hello" returns Ask.
7. Technical guide: Labs 1 to 4 render output on load; "Run edited code" and "Reset code" work; the five dialect tabs switch.
8. Live mode: with `config.js` set to `mode: "live"` and Playwright route interception returning a valid catalogue response, the Environmental Agency page renders it and shows "live API" in technical details. With the route returning 500, it shows "Service unavailable" and no datasets. On the Rust service (no interception), the four Environmental Agency examples return Answer, Answer, Clarify, Reject from the live API, and the lineage page still runs from fixtures. The stale-result race check (ported from `tests/browser_dcceew_race.py`) passes.
9. Brand: overriding `SITE_CONFIG.brand.name` changes the header and `<title>`.
10. Container (WS6 image): `/`, `/config.js`, `/api/health`, `/api/ready` return 200; `config.js` reflects env vars passed to `docker run`, correctly escaped.

---

## 11. Review checklist (for the lead agent's review after the swarm)

- Section 1.1 decisions all honoured; nothing reopened.
- Copy follows Section 3; shared copy in Section 5 used; no invented figures; every figure linked to Section 8 or labelled.
- The five pillars read as one argument from problem to cost, not five unrelated sections.
- Ownership respected; no stray edits outside each workstream's files.
- Config and API contract behave as specified, including no fixture fallback in live mode.
- Section 10 passes in full; screenshots of every page at 1280px and 390px attached to the final PR.
- `cargo test`, `cargo audit`, the container smoke test and `tests/test-deploy.sh` pass; the server makes no outbound calls.
- Remaining known gaps listed honestly in the final PR (for example: lineage domain has no live backend yet).
- Every page has a rewritten `<meta name="description">`; no "Graphs + Jev" brand text remains (the ideas of graphs and Jev stay, as pillars).
- Release (lead agent, after the PR is approved and merged): update the GitHub repository description to the new framing (homepage URL unchanged); `deploy/deploy.sh` builds a private candidate; verify it against Section 10 items 1, 5, 6, 8 and 10; promote; update `docs/deployment/cloud-run.md` with the new revision, digest and settings.
