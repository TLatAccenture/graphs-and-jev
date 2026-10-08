# Graphs + Jev Publication Implementation Plan

**Goal:** Publish the complete Graphs + Jev field guide as a standalone, static GitHub Pages site.

**Architecture:** Copy only the static field-guide files into a clean repository, rewrite root-relative paths for the GitHub Pages project prefix, and replace two backend calls with deterministic browser fixtures. Deploy with GitHub’s official Pages actions.

**Tech Stack:** HTML, CSS, vanilla JavaScript, GitHub Actions, GitHub Pages.

## Tasks

1. Extract the four pages and local assets from canonical `ArchegonDev/graphragapp`.
2. Rewrite internal links and assets for `/graphs-and-jev/`.
3. Add deterministic DCCEEW and Parkinson’s browser fixtures and accurate static-demo copy.
4. Add attribution, disclaimer, `.nojekyll`, and Pages workflow.
5. Verify links, paths, decisions, console, desktop, and mobile views.
6. Create `TLatAccenture/graphs-and-jev`, push `main`, enable Actions Pages, and verify the live URL.
