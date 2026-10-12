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
