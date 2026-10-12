# Migration-First Homepage Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Rebuild the Graphs + Jev homepage as a clear migration-to-AI-acceleration story while preserving all functioning demonstrations and technical evidence.

**Architecture:** Keep the vanilla HTML/CSS/JavaScript stack and Rust static-serving boundary. Reorder and rewrite `static/index.html`, extend `static/guide.css`, and add no dependencies or API changes. All work stays on `design/value-story` and ships as an unmerged pull request.

**Tech Stack:** Semantic HTML, vanilla CSS, vanilla JavaScript, Rust static frontend contract tests, browser automation.

---

### Task 1: Lock the editorial and preservation contracts

**Files:**
- Modify: `tests/static_frontend.rs`
- Test: `tests/static_frontend.rs`

1. Add failing tests requiring the migration-first hero and the four stage names.
2. Add failing tests requiring the business claim, illustrative caveat, fixed five outcomes, agent spine and seven future measures.
3. Add failing tests forbidding the Palindrome attribution, an empty footer, unsupported savings language, remote runtime assets, and the retired outcome vocabulary.
4. Retain existing contracts for routes, primary navigation, use cases, Spanner spotlight, labs, local assets and safety wording.
5. Run `cargo test --test static_frontend` and record expected failures.

### Task 2: Rebuild the page narrative

**Files:**
- Modify: `static/index.html`

1. Rewrite the hero around migration as an AI accelerator.
2. Move the four-stage migration visual spine directly after the hero.
3. Reframe and retain the guided graph example.
4. Reframe and retain all three use-case links/content.
5. Add the decision-value visual story, exact work-unit model and well-architected-agent spine.
6. Move Trust after the architecture story and simplify its language.
7. Preserve the complete technical guide and all source/caveat content.
8. Remove the Palindrome footer entirely.
9. Run focused static tests.

### Task 3: Implement the visual mechanisms and responsive system

**Files:**
- Modify: `static/guide.css`
- Modify: `static/index.html`

1. Add only namespaced visual-story and migration-story styles using existing tokens.
2. Implement sticky 96vh beats, one state per beat, faded history and reused tiles.
3. Add the single clickable 1–8 rail, previous/next, expand-all, keyboard handling and live status.
4. Add reduced-motion default behavior plus explicit Preview motion / Respect reduced motion control.
5. Add robust mobile flow at 320px with no ticket, caption or document overflow.
6. Implement exact residual workload allocation and exhaustive arithmetic self-check.
7. Run focused tests and browser behavior checks.

### Task 4: Verify every existing and new interaction

**Files:**
- Create: `tests/browser-homepage.mjs` only if a zero-dependency CDP/Playwright harness already exists; otherwise keep the runnable audit command in PR evidence.
- Test: `static/index.html`

1. Exercise guided scenarios, hop controls and graph nodes.
2. Exercise all eight story states by scroll, rail, keyboard and previous/next.
3. Exercise motion opt-in/out and reduced-motion fallback.
4. Sweep all reachable workload slider combinations; assert nonnegative counts and exact totals.
5. Exercise all disclosures, engine modes, labs, edited runs, resets, dialect tabs and copy controls.
6. Check every unique internal and external link target.
7. Verify 1440×900, 1024×768 and 320×844 screenshots and overflow measurements.

### Task 5: Review the story and code independently

1. Run content/business-value review: executive promise, migration logic, Google placement and claim honesty.
2. Run accessibility review: semantics, keyboard, live regions, focus, reduced motion, contrast and 320px layout.
3. Run code-quality/security review: DOM safety, namespacing, event handling, arithmetic and no new runtime egress.
4. Route the final design/code diff through the local Claude Code shim with `claude-opus-5`; fix and re-review substantive findings.
5. Run `cargo test --locked --all-targets`, `cargo build --release`, `cargo audit`, formatting/diff checks and browser audits.

### Task 6: Commit, push and open the review PR

1. Commit with exact `TLatAccenture` author/committer identity and no co-author trailer.
2. Push `design/value-story`.
3. Open a PR to `main` with the story rationale, changed section order, claim boundaries, test evidence, screenshots and Cloud Run unchanged note.
4. Verify contributors remain only `TLatAccenture`.
5. Stop with the PR open; do not merge or deploy.
