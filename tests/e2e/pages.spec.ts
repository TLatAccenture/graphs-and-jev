import { test, expect, PAGES, scrollThrough } from "./helpers";
import type { Page } from "@playwright/test";

/** Whether the brand name and the "Try the demo" button overlap in the header. */
const headerClash = (page: Page) => page.evaluate(() => {
  const name = document.querySelector(".site-header [data-brand=name]")!.getBoundingClientRect();
  const cta = document.querySelector(".site-nav .cta")!.getBoundingClientRect();
  return name.right > cta.left && name.left < cta.right && name.bottom > cta.top && name.top < cta.bottom;
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

    if (test.info().project.name === "mobile") {
      // The narrowest phones in common use: the header must still fit without overlap.
      await page.setViewportSize({ width: 320, height: 640 });
      expect(await headerClash(page), "brand name overlaps Try the demo at 320px").toBe(false);
    }

    expect(problems).toEqual([]);
  });
}
