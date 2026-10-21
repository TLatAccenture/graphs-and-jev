import { test, expect, PAGES, scrollThrough } from "./helpers";
import type { Page } from "@playwright/test";

/** Whether the brand name and the "Try the demo" button overlap in the header. */
const headerClash = (page: Page) => page.evaluate(() => {
  const name = document.querySelector(".site-header [data-brand=name]")!.getBoundingClientRect();
  const cta = document.querySelector(".site-nav .cta")!.getBoundingClientRect();
  return name.right > cta.left && name.left < cta.right && name.bottom > cta.top && name.top < cta.bottom;
});
const brandSquare = (page: Page) => page.evaluate(() => getComputedStyle(document.querySelector(".site-header .brand")!, "::before").width);
const sideways = (page: Page) => page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
/** Whether the nav clips its last item, "Try the demo". */
const ctaClipped = (page: Page) => page.evaluate(() => {
  const nav = document.querySelector(".site-nav")!.getBoundingClientRect();
  return document.querySelector(".site-nav .cta")!.getBoundingClientRect().right > nav.right + 0.5;
});

// Section 10 item 1: every page returns 200, no console errors, no failed requests, no horizontal scroll at 390px.
for (const path of PAGES) {
  test(`page ${path} loads cleanly`, async ({ page }) => {
    const problems: string[] = [];
    page.on("console", m => { if (m.type() === "error") problems.push(`console: ${m.text()}`); });
    page.on("pageerror", e => problems.push(`pageerror: ${e.message}`));
    page.on("requestfailed", r => problems.push(`failed: ${r.url()} ${r.failure()?.errorText}`));
    page.on("response", r => { if (r.status() >= 400) problems.push(`HTTP ${r.status()}: ${r.url()}`); });

    const res = await page.goto(path);
    expect(res?.status()).toBe(200);
    await scrollThrough(page);
    await page.waitForLoadState("networkidle");

    // Lazy images below the fold start loading only once scrolled to; give them time to finish.
    await expect
      .poll(() => page.evaluate(() =>
        [...document.images].filter(i => i.checkVisibility() && (!i.complete || i.naturalWidth === 0)).map(i => i.currentSrc || i.src),
      ), { message: "images that did not load" })
      .toEqual([]);

    const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
    expect(overflow, "horizontal scroll (px)").toBeLessThanOrEqual(0);
    expect(await headerClash(page), "brand name overlaps Try the demo").toBe(false);
    expect(await brandSquare(page), "brand square").toBe("12px");

    if (test.info().project.name === "mobile") {
      // The narrowest phones in common use: the header must still fit without overlap or sideways scroll.
      await page.setViewportSize({ width: 320, height: 640 });
      expect(await headerClash(page), "brand name overlaps Try the demo at 320px").toBe(false);
      expect(await brandSquare(page), "brand square at 320px").toBe("12px");
      expect(await sideways(page), "horizontal scroll at 320px (px)").toBeLessThanOrEqual(0);
    } else {
      // Just above the mobile breakpoint, such as an iPad in portrait, the desktop row must not clip "Try the demo".
      await page.setViewportSize({ width: 834, height: 800 });
      expect(await ctaClipped(page), "Try the demo clipped at 834px").toBe(false);
    }

    expect(problems).toEqual([]);
  });
}
