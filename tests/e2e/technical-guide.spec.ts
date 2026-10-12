import { test, expect, openDetails } from "./helpers";

// Section 10 item 7.
const PAGE = "technical-guide/";

test("Labs 1 to 4 render output on load", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", e => errors.push(e.message));
  await page.goto(PAGE);
  const labs = page.locator("main .lab");
  await expect(labs).toHaveCount(4);
  for (let i = 0; i < 4; i++) {
    const out = labs.nth(i).locator(".out");
    await expect(out, `lab ${i + 1} output`).not.toBeEmpty();
    await expect(out.locator(".err"), `lab ${i + 1} error`).toHaveCount(0);
  }
  expect(errors).toEqual([]);
});

for (let i = 0; i < 4; i++) {
  test(`Lab ${i + 1}: "Run edited code" and "Reset code" work`, async ({ page }) => {
    await page.goto(PAGE);
    const lab = page.locator("main .lab").nth(i);
    const code = lab.locator("textarea").first();
    const out = lab.locator(".out");
    const original = await code.inputValue();
    expect(original.length, "lab code is viewable").toBeGreaterThan(0);
    await openDetails(code);

    await code.fill(`/* edited */ ${original}`);
    await lab.getByRole("button", { name: "Run edited code" }).click();
    await expect(out.locator(".err"), "valid edited code runs").toHaveCount(0);
    await expect(out).not.toBeEmpty();

    await code.fill("(");
    await lab.getByRole("button", { name: "Run edited code" }).click();
    await expect(out.locator(".err"), "broken code reports an error").toHaveCount(1);

    await lab.getByRole("button", { name: "Reset code" }).click();
    await expect(code).toHaveValue(original);
    await expect(out.locator(".err")).toHaveCount(0);
    await expect(out).not.toBeEmpty();
  });
}

// Every run, including a syntax error, flashes the output unless the reader asks for reduced motion.
for (const motion of ["no-preference", "reduce"] as const) {
  test(`Lab output flash on every run (${motion} motion)`, async ({ page }) => {
    await page.emulateMedia({ reducedMotion: motion });
    await page.addInitScript(() => {
      const animate = Element.prototype.animate;
      (window as any).__flashes = 0;
      Element.prototype.animate = function (...args: Parameters<Element["animate"]>) {
        if (this.classList.contains("out")) (window as any).__flashes++;
        return animate.apply(this, args);
      };
    });
    await page.goto(PAGE);
    const lab = page.locator("main .lab").first();
    const code = lab.locator("textarea").first();
    await openDetails(code);
    const flashes = () => page.evaluate(() => (window as any).__flashes as number);
    const start = await flashes();
    await lab.getByRole("button", { name: "Run edited code" }).click();
    await code.fill("(");
    await lab.getByRole("button", { name: "Run edited code" }).click();
    await lab.getByRole("button", { name: "Reset code" }).click();
    expect(await flashes() - start).toBe(motion === "reduce" ? 0 : 3);
  });
}

test("the five dialect buttons switch", async ({ page }) => {
  await page.goto(PAGE);
  const tablist = page.getByRole("group", { name: "Query dialect" });
  await expect(tablist).toHaveCount(1);
  await openDetails(tablist);
  const tabs = tablist.getByRole("button");
  await expect(tabs).toHaveCount(5);
  const code = tablist.locator("xpath=..").locator("pre").first();
  const seen = new Set<string>();
  for (let i = 0; i < 5; i++) {
    await tabs.nth(i).click();
    await expect(tabs.nth(i)).toHaveAttribute("aria-pressed", "true");
    await expect(tablist.locator("[aria-pressed=true]")).toHaveCount(1);
    seen.add((await code.textContent()) || "");
  }
  expect(seen.size, "each dialect shows different code").toBe(5);
});
