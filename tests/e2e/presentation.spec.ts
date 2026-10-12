import type { Page } from "@playwright/test";
import { test, expect, PAGES } from "./helpers";

// Presentation plan (docs/plans/2026-10-13-presentation-readiness.md): the site as shown on a
// 1920×1080 projector or screen share. Runs once, in the desktop project.
test.use({ viewport: { width: 1920, height: 1080 } });
test.beforeEach(({}, info) => {
  test.skip(info.project.name !== "desktop", "presentation checks run once, at 1920×1080");
});

/** True once the element is whole on screen below the header, or, if taller than the room, starts just below it. */
function placed(page: Page, selector: string, keep?: string) {
  return page.evaluate(
    ([sel, keepSel]) => {
      const header = document.querySelector(".site-header")!.getBoundingClientRect().bottom;
      const onScreen = (r: DOMRect) => r.top >= header - 1 && r.bottom <= innerHeight + 1;
      const r = document.querySelector(sel)!.getBoundingClientRect();
      const fits = r.height <= innerHeight - header;
      const ok = fits ? onScreen(r) : r.top >= header - 1 && r.top <= header + 40;
      return ok && (!keepSel || onScreen(document.querySelector(keepSel)!.getBoundingClientRect()));
    },
    [selector, keep ?? ""] as const,
  );
}

const DEMOS = [
  { page: "./#example", buttons: "#example [data-scenario]", result: "#exampleResult", keep: ".guided-graph" },
  { page: "environmental-agency/#demo", buttons: "#demo [data-q]", result: "#result" },
  { page: "lineage-migration/#demo", buttons: "#demo [data-req]", result: "#lnResult" },
];

for (const demo of DEMOS) {
  test(`item 1: every example on ${demo.page} shows its result on screen`, async ({ page }) => {
    await page.goto(demo.page);
    const buttons = page.locator(demo.buttons);
    const count = await buttons.count();
    expect(count).toBeGreaterThan(2);
    for (let i = 0; i < count; i++) {
      await buttons.nth(i).click();
      await expect
        .poll(() => placed(page, demo.result, demo.keep), { message: `example ${i + 1}: result${demo.keep ? " and graph" : ""} on screen` })
        .toBe(true);
    }
  });
}

test("item 1: no page scrolls on load", async ({ page }) => {
  for (const path of PAGES) {
    await page.goto(path);
    await page.waitForTimeout(800);
    expect(await page.evaluate(() => scrollY), path).toBe(0);
  }
});

test("item 4: the five layers read left to right in slide order, one open at a time", async ({ page }) => {
  await page.goto("./");
  const layers = page.locator(".stack-layer");
  await expect(layers.locator("strong")).toHaveText([
    "Existing Stack",
    "MCP Tool Shed",
    "System 1 (Jev)",
    "Gemini Enterprise Agent Platform",
    "Modernisation Springboard",
  ]);
  const boxes = await layers.evaluateAll(els => els.map(e => e.getBoundingClientRect()).map(r => ({ top: r.top, left: r.left })));
  for (const [i, box] of boxes.entries()) {
    expect(Math.abs(box.top - boxes[0].top), `layer ${i} shares the row`).toBeLessThan(2);
    if (i) expect(box.left, `layer ${i} sits right of layer ${i - 1}`).toBeGreaterThan(boxes[i - 1].left);
  }
  const overflowing = await page.locator(".stack-layer, .stack-layer *").evaluateAll(els =>
    els.filter(e => e.scrollWidth > e.clientWidth + 1).map(e => e.textContent),
  );
  expect(overflowing).toEqual([]);

  await layers.nth(1).click();
  await layers.nth(3).click();
  await expect(layers.nth(1)).toHaveAttribute("aria-expanded", "false");
  await expect(layers.nth(3)).toHaveAttribute("aria-expanded", "true");
  await expect(page.locator(".stack-detail:not([hidden])")).toHaveCount(1);
  await expect.poll(() => placed(page, "#pillar-logic", ".stack-layer[aria-expanded=true]")).toBe(true);
});

test("item 3: every page renders in Instrument Sans and requests nothing from another host", async ({ page, baseURL }) => {
  const origin = new URL(baseURL!).origin;
  const elsewhere: string[] = [];
  page.on("request", r => {
    const url = r.url();
    if (!url.startsWith(origin) && !url.startsWith("data:")) elsewhere.push(url);
  });
  for (const path of PAGES) {
    await page.goto(path);
    const loaded = await page.evaluate(async () => {
      await document.fonts.ready;
      return [...document.fonts].filter(f => f.status === "loaded").map(f => `${f.family.replace(/"/g, "")} ${f.weight}`);
    });
    expect(loaded, path).toContain("Instrument Sans 400");
    expect(loaded, path).toContain("Instrument Sans 700");
  }
  expect(elsewhere).toEqual([]);
});

test("item 9: print keeps dark bands readable and prints every layer", async ({ page }) => {
  await page.goto("./");
  await page.emulateMedia({ media: "print" });
  const printed = await page.evaluate(() => {
    const colour = (sel: string) => getComputedStyle(document.querySelector(sel)!).color;
    return {
      governance: colour("#governance h2"),
      governanceText: colour("#governance .proof li"),
      footer: colour(".site-footer p"),
      details: [...document.querySelectorAll(".stack-detail")].filter(d => getComputedStyle(d).display !== "none").length,
    };
  });
  for (const key of ["governance", "governanceText", "footer"] as const) {
    expect(printed[key], `${key} prints in dark ink`).toBe("rgb(10, 10, 10)");
  }
  expect(printed.details).toBe(5);
});
