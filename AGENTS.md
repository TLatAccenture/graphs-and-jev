# Agent rules for this repository

This repository is being converted into "The MCP Tool Shed", a thought piece on agentic migration and modernisation.

**Before any work, read `docs/plans/2026-10-12-mcp-shed-cursor-handover.md` in full.** It defines the workstreams, who owns which files, the shared copy, the configuration and API contracts, and the acceptance tests. If anything conflicts with it, the handover wins.

## Non-negotiable rules

1. Only edit files your workstream owns (handover Section 2.2). Put anything else under "Requests for other workstreams" in your PR description.
2. WS0 Foundation merges first. No other workstream starts until it has.
3. Static HTML, CSS and vanilla JavaScript only. No frameworks, bundlers or build step in `static/`.
4. All internal links and asset paths are relative. Never write `/graphs-and-jev/`.
5. Only `static/js/api.js` may call `fetch`. Pages talk to data through `window.siteApi`.
6. In live mode, a failed API call shows "Service unavailable". Never fall back to fixtures and never fabricate a result.
7. Copy: British English, no em dashes, plain and factual, no marketing language.
8. Every number or third-party claim links to a source in handover Section 8, or is labelled "illustrative" or "assumption". Never invent figures. Never write "sub-100 ms".
9. Never mention DCCEEW, Parkinson's or RLCD. Use "Environmental Agency" (fictional) and "System 1 model".
10. Keep the existing accessibility patterns: labelled controls, `aria-live` status, keyboard support, outcome shapes plus text, 44px targets.
11. Before reporting done, run `tests/static-checks.sh` and the Playwright tests for your pages, and attach desktop (1280px) and mobile (390px) screenshots.
12. Work only in your own git worktree on branch `mcp-shed-wsN`, cut from `mcp-shed`. Never commit to `main` or `mcp-shed` directly; the lead agent merges.
13. Commit as the repository's configured `TLatAccenture` identity. No co-author trailers.
14. Never run `deploy/deploy.sh` or any `gcloud` command that changes GCP. The lead agent releases.
15. Never commit secrets, tokens, GCP credentials or local environment files.
