# Graphs + Jev publication design

## Purpose

Publish the complete Graphs + Jev field guide as a standalone personal research demonstration under `TLatAccenture/graphs-and-jev`. It is not an official Accenture or Google Cloud product.

## Architecture

The publication is plain HTML, CSS, JavaScript, SVG, and PNG hosted by GitHub Pages at `/graphs-and-jev/`. The landing page, DCCEEW, Parkinson’s, Little Bunnies, technical guide, and browser labs are retained. Backend-dependent DCCEEW and Parkinson’s interactions use deterministic representative fixtures in `demo-data.js`; they make no runtime API, model, analytics, cookie, or credential calls. External citations open only on user action.

## Delivery requirements

A clean repository history attributes canonical source `ArchegonDev/graphragapp`. GitHub Actions deploys an immutable commit to Pages. Verification covers project-relative paths, internal links, representative decision outcomes, desktop/mobile rendering, and browser console errors.
