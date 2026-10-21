import { test, expect, PAGES, useConfig } from "./helpers";

// Section 10 item 9: overriding SITE_CONFIG.brand.name changes the header and <title> (Section 4.2).
const NAME = 'Test Shed "override"';
const TAGLINE = "Test tagline";

for (const path of PAGES) {
  test(`brand override on ${path}`, async ({ page }) => {
    await useConfig(page, c => { c.brand.name = NAME; c.brand.tagline = TAGLINE; });
    await page.goto(path);
    await expect(page.locator(".site-header [data-brand=name]")).toHaveText(NAME);
    if (path === "./") await expect(page).toHaveTitle(`${NAME}: ${TAGLINE}`);
    else await expect(page).toHaveTitle(new RegExp(` · ${NAME.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}$`));
  });
}

test("a long brand name keeps anchored jumps clear of the header at 320px", async ({ page }, info) => {
  test.skip(info.project.name !== "mobile", "Narrow-screen check.");
  await useConfig(page, c => { c.brand.name = "Migration Acceleration with Google Cloud, configured with a much longer name"; });
  await page.setViewportSize({ width: 320, height: 640 });
  await page.goto("environmental-agency/");
  await expect
    .poll(() => page.evaluate(() =>
      document.querySelector(".site-header")!.getBoundingClientRect().height <= parseFloat(getComputedStyle(document.documentElement).scrollPaddingTop),
    ), { message: "sticky header taller than the scroll offset" })
    .toBe(true);
});
