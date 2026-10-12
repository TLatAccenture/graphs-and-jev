# MCP Shed Revamp Plan

**Goal:** Turn the Graphs + Jev field guide into a field guide for agentic migration and modernisation built around the MCP shed. Graphs and Jev stay in the guide, as two of five pillars rather than the whole story.

**Approach:** Keep the static site's architecture, design system and interaction components. Replace the narrative, the sample data and two of the three use cases. Write and approve the copy before touching any HTML.

**Status:** Superseded by 2026-10-12-mcp-shed-cursor-handover.md

---

## Phase 0: Decisions to make first

These change what gets built, so they come before any work.

### D1. Branding and where it lives (blocking)

The working title is "Accenture Acceleration for Migration and Modernisation". The site is currently public, hosted under a personal GitHub account (`TLatAccenture`), and its README says it is not an official Accenture product. An Accenture-branded offering on a personal public site contradicts that disclaimer and is likely to need brand and marketing approval.

| Option | What it means | Trade-off |
|---|---|---|
| A. Internal Accenture asset | Accenture name used; hosted internally (for example in the knowledge repository) | Needs an internal host; no public link for clients |
| B. Public, neutral name | For example "The MCP Shed: a field guide to agentic migration and modernisation"; personal research disclaimer kept | Public and shareable now; no Accenture branding |
| C. Public and Accenture-branded | Only after formal approval | Slowest; approval outside our control |

Recommendation: build under B now, so the work is not blocked, and keep the brand in one place (title, header, footer, README) so switching to A or C is a find-and-replace.

### D2. Named clients and departments

The DCCEEW page is described as "a demonstration for" a named Australian Government department. Under a migration and modernisation pitch from an Accenture practice lead, that reads as a client engagement. Decide whether to keep the name, or make the scenario a generic "federal environment agency".

### D3. Sequencing against the Cloud Run plan

`2026-10-08-cloud-run-rust-publication*.md` moves the current content to Cloud Run with a Rust backend. If the content is about to change, deploying the old content first is wasted effort. Recommendation: pause the Cloud Run plan, do this revamp on the static site, then rebase the Cloud Run plan onto the new pages (the Rust decision pipeline still applies, with new domains).

### D4. Claims that need evidence before they go public

| Claim | Status | Action |
|---|---|---|
| Stripe Minions merge 1,000+ PRs a week | Confirmed in Stripe's engineering blog | Cite stripe.dev |
| Stripe's "Toolshed" MCP server exposes around 400 tools | Secondary source only; not found in Stripe's own posts so far | Find the primary source or drop the number |
| Jev routes and gates in under 100 ms | The site currently calls Jev performance "vendor-reported" | Measure p50/p95 on stated hardware, or say "designed for" |
| "RLCD models" | Not defined anywhere in the repo | Define the term or drop it |
| MCP is the de facto standard | Broadly accepted | One citation (spec, adoption by major vendors) |
| Official MCP servers for SAP, Salesforce, Databricks, ServiceNow, Atlassian, M365, AWS, Azure | Not yet checked | Research task in Phase 1, dated, with links |

### D5. Cost is in the problem statement but not in the solution

The problem statement stresses cost today and tomorrow. None of the five pillars addresses cost directly. Either add a sixth pillar (Economics: cost per migrated object, token spend, run-cost of the target estate) or make cost an explicit part of Visibility. Recommendation: fold it into Visibility as "Visibility and cost", and reuse the existing engine cost section.

### D6. Repository name and URL

`graphs-and-jev` no longer describes the content. Renaming the GitHub repository keeps a redirect for the repo, but the GitHub Pages URL path changes and old links break. Decide once D1 is settled; until then keep the current name.

---

## Phase 1: Narrative and copy (no HTML)

**Output:** `docs/copy/` with one markdown file per page, approved before Phase 2.

### Task 1.1: Lock the core message

Draft, tightened from the brief (British English, no em-dashes, plain):

> **Problem.** Agents need the right tools for the job. Most enterprises have built up a sprawling estate over decades: storage in AWS and Azure, SaaS platforms such as SAP, Salesforce, Databricks, ServiceNow and Atlassian, and years of Microsoft 365 files in PowerPoint, Excel and Word. Migrating and modernising that estate with agents in 2026 needs a different way of thinking, one that accounts for what the work costs today and what the estate will cost to run tomorrow.
>
> **Solution.** The MCP shed: a governed, curated set of MCP servers that gives each migration agent the tools its task needs, and nothing more.
>
> **Why now.** Context windows keep growing, but well-architected agents still need careful context management. The Model Context Protocol has become the de facto standard for connecting agents to tools: a common language for networked function calling, as APIs were for the previous generation of systems. It has its critics, but adoption has largely settled the argument. Stripe's in-house coding agents, Minions, now merge more than a thousand pull requests a week. We expect the same shift in enterprise migration and modernisation.

The five pillars, in stack order (bottom to top):

1. **Connectivity.** Standard, governed MCP servers replace brittle one-off scripts and the credential leaks that come with them.
2. **Relational context.** A graph in Spanner Graph, or another engine, gives agents lineage and the enterprise ontology, so a change upstream does not silently break something downstream.
3. **Logic.** Skills and hooks encode prescriptive migration playbooks and safety checks, reducing drift, broken code and malformed schemas.
4. **Reflexes.** Jev and other small decision models route, classify and gate tool calls fast enough to sit in the hot path, and stop uncalibrated actions before they run.
5. **Visibility and cost.** Evals show engineers what agents did; BigQuery and Looker deliver results to users at production speed and reliability; cost per migrated object is tracked from day one.

Note on wording: a fast classifier does not stop hallucination. It stops an uncalibrated output from becoming an action. The copy should claim the second, not the first.

Steps:
1. Agree the message above (or a revision of it).
2. Agree the bounded outcomes vocabulary for migration. Proposal: **proceed, ask, refuse, escalate, no data** (mapped from answer, clarify, reject, review, no data in the current code).

### Task 1.2: Verify the MCP shed catalogue

For each system in the brief, record: official MCP server (yes/no, vendor or community), hosting model (remote or local), auth model, read/write scope, date checked, link. Systems: AWS (S3 and others), Azure (Blob and others), SAP, Salesforce, Databricks, ServiceNow, Atlassian (Jira, Confluence), Microsoft 365 (SharePoint, OneDrive, Office files), BigQuery, Spanner, Looker.

Output: `docs/copy/shed-catalogue.md`, which feeds the technical guide table.

### Task 1.3: Page copy

One file each for the landing page, the two use-case pages and the reframed Little Bunnies page, following the existing page structure (business question, what the graph adds, what Jev controls, safety boundary, demo prompts).

---

## Phase 2: Landing page (`index.html`)

Keep the visual system (`guide.css`, fonts, layout grid, SVG style). Section by section:

| Current section | New section | Work |
|---|---|---|
| Hero: "Turn connected evidence into bounded decisions" | Hero: the MCP shed | Rewrite |
| How it works: three steps (connect, control, show) | Five pillars as a layered stack | New SVG (`assets/mcp-shed-stack.svg`); rewrite glossary to add Skill, Hook, MCP server, Lineage |
| Guided example: ACME, 12-fact graph | Guided example: one migration followed end to end | New sample graph and questions (below); reuse the evidence-path component and hop slider |
| Use cases: DCCEEW, Parkinson's, Little Bunnies | Use cases: estate discovery, lineage-guarded migration, Little Bunnies | Rewrite cards |
| Trust and deployment | Governance: least-privilege tools, refusal before tool call, audit trail, no runtime egress in the demo | Rewrite, keep the structure |
| Technical guide | Technical guide (Phase 4) | Extend |

### Task 2.1: New guided example graph (about 15 to 20 facts)

A small fictional enterprise estate, for example:
SAP table `KNA1` (customer master) → Databricks pipeline → BigQuery `customers` → Looker dashboard "Monthly churn" (owner: Finance); Salesforce Account sync job; ServiceNow CI for the sync job; two versions of a pricing workbook in SharePoint.

Example questions, one per outcome:
- "Which reports break if we drop `KNA1.REGION`?" → **proceed** with the two-hop blast radius and its path.
- "Migrate the pricing workbook to BigQuery." → **ask**: two versions exist; which is the source of truth?
- "Retire the legacy CRM sync job." → **escalate**: a downstream report has no recorded owner.
- "Copy production customer records to the dev sandbox." → **refuse** before any tool is called.

### Task 2.2: Implement

1. Replace the sample graph and questions in the inline script; keep the rendering code.
2. Update hero, nav labels, `<title>`, meta description, glossary.
3. Add the stack SVG, light and dark.

---

## Phase 3: Use-case pages

### Task 3.1: `estate-discovery/` (replaces `dcceew/`)

The current DCCEEW page already tells the right story: evidence spread across documents, tables, records and policy; reach, then connect, then decide. Reframe it from "which dataset answers this question" to "what do we actually have, and what depends on what":
- Reach: MCP servers read the CMDB (ServiceNow), storage inventories (S3, Blob), Confluence, SharePoint.
- Connect: the graph links systems, datasets, owners and consumers.
- Decide: Jev answers inventory questions only when the graph covers them; asks when ownership is ambiguous; refuses out-of-scope access.
Rewrite the fixture in `demo-data.js` accordingly. Keep a redirect stub at `dcceew/` until links are updated (subject to D2).

### Task 3.2: `lineage-migration/` (new, replaces `parkinsons/`)

A schema or code migration (for example a COBOL or legacy SQL job moved to BigQuery) where the agent must not break downstream consumers:
- Logic: a playbook skill defines the steps and checks.
- Relational context: the graph computes the blast radius before any change.
- Reflexes: Jev gates each step (proceed, ask, refuse, escalate).
- Visibility: each step writes an eval record.
Reuse the Parkinson's page shell (safety banner becomes the change-safety banner; refusal before retrieval becomes refusal before tool call).

### Task 3.3: `little-bunnies/` (keep, reframe)

The small-world result is the context-management argument in the brief: an agent needs a two-hop neighbourhood of the lineage graph, not the whole estate. Change the framing copy only; keep the methodology and numbers.

### Task 3.4: Retire Parkinson's

Remove the page, `assets/parkinsons-research.svg` and its fixture. It does not fit a migration story. It stays in git history.

---

## Phase 4: Technical guide and labs

| Item | Pillar | Work |
|---|---|---|
| MCP shed catalogue table (from Task 1.2) | Connectivity | New |
| Tool-scoping lab: one agent with the full shed versus a router-selected subset; shows tokens spent on tool definitions and selection accuracy (simulated, deterministic) | Connectivity, Reflexes | New lab |
| Lab 1: text versus graph retrieval | Relational context | Change sample graph to the estate |
| Lab 2: token budget | Relational context | Change sample graph to the estate |
| Lab 3: calibrated thresholds | Reflexes | Rename actions to migration actions (approve schema change, route ticket, flag for review) |
| Engine grades and costs, Spanner Graph spotlight | Relational context | Keep; re-check prices (last checked 25 September and 2 October 2026) |
| Playbook skill and hook example (one annotated example) | Logic | New, short |
| Eval record schema and a BigQuery/Looker outline | Visibility and cost | New, short |
| One question, five dialects | Relational context | Change the query to a lineage question |

Vendor logos: the engine logos are used in a comparative guide. Adding SAP, Salesforce and other SaaS logos to a page that may carry Accenture branding raises trademark questions. Use text names for the shed catalogue.

---

## Phase 5: Verification

1. Every factual claim in D4 resolved: cited, measured, or removed.
2. Internal links and project-relative paths (`/graphs-and-jev/` or the new path).
3. Each guided example and demo returns its intended outcome (proceed, ask, refuse, escalate, no data).
4. Desktop and mobile rendering, light and dark, no console errors.
5. No real client data, credentials or personal data in fixtures.
6. Independent read-through by someone who has not seen the drafts: can they state the problem, the shed and the five pillars after the landing page?

## Phase 6: Publication

1. Publish to GitHub Pages under the D1 decision.
2. Update README, `docs/design.md` and the repository description.
3. Rebase the Cloud Run plan onto the new pages and domains (D3).

---

## What stays, changes and goes

| File | Fate |
|---|---|
| `guide.css` | Stays; small additions for the stack diagram |
| `index.html` | Most copy and the sample graph replaced; components kept |
| `dcceew/` | Becomes `estate-discovery/` |
| `parkinsons/` | Removed; shell reused for `lineage-migration/` |
| `little-bunnies/` | Stays; framing copy changed |
| `demo-data.js` | Rewritten for the new domains |
| `assets/evidence-path.svg`, `small-world.svg`, `spanner-graph-architecture.svg`, `engine-logos/` | Stay |
| `assets/dcceew-landscape.svg` | Redrawn as an estate landscape |
| `assets/parkinsons-research.svg` | Removed |
| `README.md`, `docs/design.md` | Rewritten after D1 |
| `.github/workflows/pages.yml` | Unchanged unless the repo is renamed |

## Rough effort

Estimates for one person with agent help, excluding approval waits: Phase 1, 2 days; Phase 2, 2 days; Phase 3, 2 to 3 days; Phase 4, 2 to 3 days; Phase 5 and 6, 1 day. About two working weeks in total.

## Sources

- Stripe, "Minions: Stripe's one-shot, end-to-end coding agents": https://stripe.dev/blog/minions-stripes-one-shot-end-to-end-coding-agents
- Stripe, Part 2: https://stripe.dev/blog/minions-stripes-one-shot-end-to-end-coding-agents-part-2
- VirtusLab, "Toolshed model: 400+ tools behind one MCP (Stripe)" (secondary): https://visdom-maturity-matrix.virtuslab.com/guides/infrastructure/toolshed-model-400-tools-behind-one-mcp-stripe
