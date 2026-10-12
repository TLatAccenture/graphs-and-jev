# Presentation readiness plan

**Date:** 13 October 2026
**Status:** Implemented and tested; see Results at the end. Still to do, by Anthony: commit and merge, the event deploy, and the slide changes.
**Builds on:** `2026-10-12-mcp-shed-cursor-handover.md` and `AGENTS.md`. Where this file differs, this file wins.

## Purpose

The site will be shown live: on a projector or screen share at 1920×1080, often at 125% browser zoom, alongside an architecture slide that reads left to right. It may also be shared afterwards as a link or a PDF. The presentation review of 12 October found ten problems. This plan fixes the eight that live in the repository and lists the two that live in the slides.

Item numbers match the review.

## Rules

Everything in `AGENTS.md` still applies: British English, no em dashes, plain copy, relative paths, only `static/js/api.js` calls `fetch`, no fixture fallback in live mode, existing accessibility patterns kept, every figure sourced. In addition:

- Nothing moves keyboard focus that the user did not move.
- Motion still follows the reader's reduced-motion setting.
- No new third-party requests. After this work the pages make none at all.

## Changes in the repository

### Item 1. Show every demo result on screen

**Problem.** A click on a demo example renders the result below the bottom of the screen, so the audience sees a button change colour and nothing else. At 1920×1080 the Environmental Agency and lineage results start about 1,250 to 1,300px down. On the landing page the decision is cut off at 1920 and fully off-screen at 125% zoom.

**Change.**
- New `static/js/ui.js` exposing `window.siteUi.reveal(element)`. It scrolls the element into view only when it is not already fully visible, aligns its top below the sticky header when it is taller than the screen, and otherwise brings its bottom edge into view. Smooth or instant scrolling follows the page's existing `scroll-behavior`, which is already `auto` under reduced motion. Focus does not move.
- Landing guided example: reveal `#exampleResult` when the reader chooses a request. The scenario selected on page load does not scroll the page.
- Landing guided example, on screens at least 980px wide and 700px tall: the graph column is sticky below the header and the graph is capped to the height that keeps the whole column on screen. The graph stays in view while the result scrolls in.
- Environmental Agency and lineage demos: reveal the result panel after each request the reader starts.

**Files.** `static/js/ui.js` (new), `static/index.html`, `static/environmental-agency/index.html`, `static/lineage-migration/index.html`, `static/styles/landing.css`.

**Done when.** At 1920×1080 and 1536×864, after a click on any example on the three demos, the result's top edge is below the header and, when the result fits, its bottom edge is on screen. On the landing page the graph is fully on screen at the same moment. Loading any page leaves it at the top.

### Item 2. Keep the audience off the shared API during the talk

**Problem.** The Environmental Agency demo calls the live API. Its rate limit is shared by everyone the server is serving, so an audience opening a shared link can push the presenter into "Service unavailable: Too many requests".

**Change.** No code change to the pages: the catalogue mode is already a deployment setting. For the event build, set `API_MODE_CATALOGUE=fixtures` so the demo runs in the browser. Its four examples give the same outcomes in both modes. Also set `MIN_INSTANCES=1` for the day, so the first visitor does not wait for a cold start (3.9 seconds measured on 12 October).
- `deploy/cloud-run.env.example` documents both settings in an "Event build" block.
- The Playwright Environmental Agency spec reads the mode the page was actually served with, so the live suite passes against an event build as well as a live one.

**Files.** `deploy/cloud-run.env.example`, `tests/e2e/environmental-agency.spec.ts`.

**Done when.** The live suite passes against the Rust service started with `API_MODE_CATALOGUE=fixtures` and with the default. With the event setting, the Environmental Agency demo makes no `/api/pipeline` calls.

### Item 3. One typeface everywhere, served by the site

**Problem.** The three use-case pages load Instrument Sans from Google Fonts; the landing page and technical guide do not, so they fall back to the system font. Moving from the landing page into a demo visibly changes the type. The fonts also depend on venue Wi-Fi reaching Google.

**Change.**
- Self-host Instrument Sans (400, 600, 700) and JetBrains Mono (400, 600, 700), Latin subset, as WOFF2 in `static/assets/fonts/`, with each font's SIL Open Font Licence beside it. Source: Fontsource 5.3.0 packages from the npm registry. Under `/assets/` they get the existing week-long cache.
- `@font-face` rules in `guide.css` with `font-display: swap`, so every page gets the fonts through the stylesheet it already loads.
- Remove the Google Fonts `<link>` and `preconnect` tags.
- Tighten the Content Security Policy: `font-src 'self'`, and `style-src` without `https://fonts.googleapis.com`.
- Update the Rust tests that assert the policy and the remote-link rule, and the policy description in `docs/api-contract.md`.

**Files.** `static/assets/fonts/*` (new), `static/guide.css`, the three use-case pages, `src/site.rs`, `tests/api_boundary.rs`, `tests/static_frontend.rs`, `docs/api-contract.md`, `README.md`.

**Done when.** Every page renders in Instrument Sans with no request to any host other than the site, and `cargo test` passes. Behaviour is otherwise unchanged: the right arrow (→) is in neither font's Latin subset, so it falls back to the system font exactly as it did with Google Fonts.

### Item 4. Lay the architecture out in slide order

**Problem.** The slide reads left to right: Existing Stack, MCP Tool Shed, System 1 (Jev), Gemini Enterprise Agent Platform with Spanner Graph as the context fabric, Modernisation Springboard. The landing page stacks the same five layers vertically with the Springboard on top, and an opened layer pushes the rest below the fold.

**Change.**
- At 1100px and wider, the five layers sit in one row, left to right in slide order, with a chevron between each. The opened layer's detail spans the full width below the row, so the row never moves.
- One layer open at a time, at every width.
- Below 1100px nothing changes: a vertical stack, foundation at the bottom on tablets, in reading order on phones.
- Layer names, numbers and colours are unchanged.

**Files.** `static/guide.css`, `static/styles/landing.css`, `static/index.html`.

**Done when.** At 1100, 1280, 1536 and 1920px wide the five layer buttons share one row in slide order with no text overflowing; opening a layer closes the one before; existing keyboard and click tests pass.

### Item 7. One colour code and one vocabulary on the Environmental Agency page

**Problem.** The page uses teal for RAG and raspberry for MCP. On the landing page teal is the MCP Tool Shed and raspberry is Gemini Enterprise Agent Platform. Its four steps (Reach, Connect, Decide, Show) do not name the layers. It also says Clarify, Reject and Review where every other page says Ask, Refuse and Escalate.

**Change.**
- MCP takes the MCP Tool Shed teal. RAG, which is not one of the five layers, becomes neutral grey. The graph stays Spanner Graph blue and Jev stays System 1 purple.
- The four steps name their layer: Reach, MCP Tool Shed; Connect, context fabric; Decide, System 1 (Jev); Show, visibility.
- Outcome labels become Answer, Ask, Refuse, Escalate and No data, in the result, the reading guide and the Decide section. Answer stays because the catalogue answers a question instead of running a step; the reading guide says so in one line.
- `CATALOGUE_LABELS` and the expected outcomes in the Playwright spec follow. The handover's Section 5.3 allowance for catalogue wording is withdrawn.

**Files.** `static/guide.css`, `static/environmental-agency/index.html`, `tests/e2e/helpers.ts`, `tests/e2e/environmental-agency.spec.ts`.

**Done when.** The page shows no teal or raspberry except where they mean the MCP Tool Shed and Gemini Enterprise Agent Platform, and its four examples return Answer, Answer, Ask and Refuse.

### Item 8. Headings without orphans

**Problem.** At 1920 and at 125% zoom the hero headline leaves "tools." alone on its last line.

**Change.** `text-wrap: balance` on `h1` and `h2`; `text-wrap: pretty` on ledes and section introductions. Browsers without support ignore it.

**Files.** `static/guide.css`.

**Done when.** No heading on the landing page ends with a single word on its own line at 1920, 1536 or 1280px.

### Item 9. Print and Save as PDF

**Problem.** Everything prints since the 12 October fix, but headings separate from their content, the landing page runs to 12 A4 pages with several half empty, and the layer details are hidden. With background graphics off, which is the browser default, white text on the black bands and the footer would print white on white.

**Change.** One `@media print` block in `guide.css`:
- Hide the skip link, the sticky header and the hop buttons.
- Halve band padding and keep headings with what follows; keep cards, figures, tables and list items whole.
- Print all five layer details.
- Turn dark bands, the footer and filled buttons into dark text on white.
- Keep small colour chips that carry meaning (score cells, pills, filled ratings) with `print-color-adjust: exact`.

**Files.** `static/guide.css`.

**Done when.** The landing page saved as an A4 PDF with background graphics off has no blank or white-on-white text, no heading at the foot of a page, all five layer details, and fewer pages than the 12 measured on 12 October.

### Item 10. Graph labels readable from the back of the room

**Problem.** The guided-example graph draws relationship labels at about 11px on a 1920 screen, with underscores ("has_column").

**Change.** Relationship labels from 7.5 to 8.5 units and node labels from 9.5 to 10.5 units in the graph's coordinate space, with spaces instead of underscores, as the lineage path below the graph already shows them. Check for overlaps at 390px and 1920px.

**Files.** `static/index.html`, `static/guide.css`, `static/styles/landing.css`.

**Done when.** No two labels overlap at 390, 1280 or 1920px wide, and the smallest graph label on a 1920 screen is at least 12px.

## Changes in the slides (Anthony)

**Item 5. Names.** "Gemini Agent Platform" becomes **Gemini Enterprise Agent Platform** (Google's current name: <https://docs.cloud.google.com/agent-builder/overview>). "Context fabric: GCP Spanner" becomes **Context fabric: Spanner Graph**.

**Item 6. Colours.** Use the site's layer colours so the audience learns one code:

| Layer | Colour |
|---|---|
| Existing Stack | #4A4A4A |
| MCP Tool Shed | #00737A |
| System 1 (Jev) | #7500C0 |
| Gemini Enterprise Agent Platform | #A3165A |
| Context fabric: Spanner Graph | #0067A5 |
| Modernisation Springboard | #000000 |

## Tests

Existing gates, all of which must pass:

- `bash tests/static-checks.sh`
- `cargo test`
- Playwright `TARGET=root`, `TARGET=subpath`, and `TARGET=live` against `cargo run`, once with the default settings and once with `API_MODE_CATALOGUE=fixtures`.

New `tests/e2e/presentation.spec.ts`, desktop project only, at 1920×1080:

- Each demo brings its result on screen after a click, and the landing graph stays on screen with it.
- No page scrolls on load.
- The five layers form one row in slide order without overflow, and opening one closes the last.
- Every page has Instrument Sans loaded and requests nothing from another host.
- In print media the dark bands print dark text and all five layer details show.

Visual checks: each presenting stop at 1920×1080, 1536×864 and 1280×650; the landing page as an A4 PDF with background graphics off.

## Deploy for the event

`deploy/deploy.sh` needs a clean worktree whose branch matches `origin`, so commit and push first.

1. Review this branch, open a pull request and merge it to `main`.
2. In your private copy of `deploy/cloud-run.env.example`, set:

   ```bash
   API_MODE_CATALOGUE=fixtures
   MIN_INSTANCES=1
   ```

3. Run `deploy/deploy.sh <env-file>`. Optionally run the live Playwright suite against the candidate tag URL it prints. Then run `deploy/promote.sh <revision> <env-file>`.
4. After the event, remove both lines and repeat step 3. That restores the live catalogue API and scaling to zero.

## Run of show (about 10 minutes)

1. Architecture slide, then switch to the site: full screen, 125% zoom.
2. Hero, 30 seconds: one agent borrows 2 of 10 servers, only what its task needs.
3. Problem, 45 seconds: "The models are ready. The estate is not."
4. Approach, 2 minutes: the five layers now read like the slide. Open layer 03 for skills, hooks and the context fabric.
5. Watch the gate decide, 3 minutes: all four requests. The result and the graph stay on screen.
6. Lineage demo, 2 minutes: Rename proceeds with the blast radius and eval record; Copy production is refused before any tool call.
7. Where to start, 1 minute: "Start with one pattern. Scale on evidence."

Keep the Environmental Agency page, Little Bunnies and the technical guide for questions.

On the day:

- Five minutes before you start, open the site and click each demo once.
- Bookmark the stops: `/#pillars`, `/#example`, `/lineage-migration/#demo`, `/#start`.
- If the Wi-Fi fails, `cd static && python3 -m http.server 8000` serves the whole site on in-browser data, fonts included, with no network.

## Results

Checked on 13 October 2026, on branch `presentation-readiness` from `main` at `e9f177b`.

| Gate | Result |
|---|---|
| `bash tests/static-checks.sh` | Pass |
| `cargo test` | 44 passed, including the new font and policy checks |
| Playwright `TARGET=root` | 110 passed, 18 skipped (mobile copies of the presentation checks; service-only tests) |
| Playwright `TARGET=subpath` | 110 passed, 18 skipped |
| Playwright `TARGET=live`, Rust service with default settings | 115 passed, 13 skipped (mobile copies of the presentation checks; container-only tests) |
| Playwright `TARGET=live`, Rust service with `API_MODE_CATALOGUE=fixtures` | 115 passed, 13 skipped |
| `bash tests/test-deploy.sh` | Pass |

By item:

1. Results land on screen after every click on all three demos, in both directions through the examples, at 1920×1080, 1536×864, 1440×900, 1280×800, 1280×650, 1000×760 and 390×844. On the landing page the graph stays whole on screen beside the result at every size where it is sticky (980px wide and 700px tall or more). At 1280×650, which is 150% zoom on a 1080p screen, the graph is not sticky and the result wins. No page scrolls on load.
2. The event settings are documented in `deploy/cloud-run.env.example`. The live suite passes against both configurations because the Environmental Agency spec now reads the mode the page was served with.
3. All five pages render in Instrument Sans with no request to any other host. The policy is now `font-src 'self'`, and `style-src` no longer names Google. The fonts add 116 KB in six files, cached for a week.
4. At 1100px and wider the five layers form one row in slide order with chevrons between them, titles aligned and no text overflowing. One layer is open at a time, and opening a layer brings its detail on screen with the row still visible.
7. The Environmental Agency page uses the landing page's layer colours, names each step's layer, and uses Answer, Ask, Refuse, Escalate and No data.
8. The landing headline sets on three balanced lines at 1920, 1536 and 1280px wide.
9. With background graphics off, the landing page prints to 9 A4 pages (12 before) with dark text throughout, all five layer details, and headings kept with their content.
10. Graph labels are 8.5 and 10.5 units, about 12.5px and 15.5px on a 1920 screen, with spaces instead of underscores and no overlaps at 390 or 1920px.

Also changed, from a parallel implementation of this plan: section links such as `/#pillars` were offset twice (the page's `scroll-padding-top` plus a `scroll-margin-top` on every section) and landed 103px below the header. With the section rule removed they land 19px below it, and at 1536×864 `/#pillars` shows all five layers.

Also changed: `AGENTS.md` points agents at this plan and adds two rules (no third-party requests; reveal results without moving focus), and the ten screenshots in `docs/screenshots/` are refreshed.

Not done here, by design: the event deploy (step "Deploy for the event"), and items 5 and 6 in the slides.
