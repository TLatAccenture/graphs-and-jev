import { test, expect, PAGES } from "./helpers";

// Print and "Save as PDF" never scroll, so scroll-reveal and entrance animations must not apply there.
for (const path of PAGES) {
  test(`${path} prints with every section visible`, async ({ page }) => {
    await page.emulateMedia({ media: "print", reducedMotion: "no-preference" });
    await page.goto(path);
    await page.waitForLoadState("load");
    const report = await page.evaluate(() => {
      const shown = (el: Element) => !el.closest("[hidden]") && el.getClientRects().length > 0;
      const faded = [...document.querySelectorAll("main *")]
        .filter(shown)
        .filter(el => getComputedStyle(el).opacity === "0")
        .filter(el => !el.matches(".elabel--text .emark"))
        .map(el => el.outerHTML.slice(0, 80));
      // The labs' Web Animations flash only tints a background, so count CSS animations.
      return { animations: document.getAnimations().filter(a => a instanceof CSSAnimation).length, faded };
    });
    expect(report.animations, "no CSS animations in print").toBe(0);
    expect(report.faded, "no content left at opacity 0 in print").toEqual([]);
  });
}
