import type { Page } from "@playwright/test";
import { test, expect, expectOutcome, STEP_LABELS, TECH_ANCHORS } from "./helpers";

// Section 10 item 3, scenarios from Section 6.1.
const SCENARIOS = [
  {
    button: "Which reports break if we drop KNA1.REGION?",
    label: "Proceed",
    answer: "Two Looker dashboards, Regional sales and Monthly churn, read the BigQuery column mapped from KNA1.REGION.",
  },
  {
    button: "Migrate the pricing workbook to BigQuery.",
    label: "Ask",
    answer: "Two workbooks describe the same price list: Pricing FY26 v3.xlsx and Pricing FY26 final.xlsx.",
  },
  {
    button: "Retire the legacy CRM sync job.",
    label: "Escalate",
    answer: "The job feeds Salesforce Account, which feeds the quarterly regulatory return.",
  },
  {
    button: "Copy production customer records to the dev sandbox.",
    label: "Refuse",
    answer: "Production customer data may not be copied outside production.",
  },
];

for (const s of SCENARIOS) {
  test(`guided example: "${s.button}" shows ${s.label}`, async ({ page }) => {
    await page.goto("./#example");
    const example = page.locator("#example");
    await expectOutcome(page, STEP_LABELS, s.label, () => example.getByRole("button", { name: s.button }).click());
    await expect(example).toContainText(s.answer);
  });
}

// Section 10 item 4, pillar detail text from Section 5.2.
const PILLARS = [
  { name: "Connectivity", detail: "scoped, short-lived credentials" },
  { name: "Relational context", detail: "what sits upstream and downstream" },
  { name: "Logic", detail: "Hooks run before and after tool calls" },
  { name: "Reflexes", detail: "it stops an uncalibrated output from becoming an action" },
  { name: "Visibility and cost", detail: "cost per migrated object" },
];

const CONTROLS = "button, [role=button], [role=tab], summary, a[href^='#'], [tabindex='0']";

for (const p of PILLARS) {
  const layer = (page: Page) =>
    page.locator("#pillars").locator(CONTROLS).filter({ hasText: new RegExp(`^\\s*(?:\\d+\\s*)?${p.name}`) }).first();

  test(`pillar "${p.name}" opens by click`, async ({ page }) => {
    await page.goto("./#pillars");
    await layer(page).click();
    await expect(page.locator("#pillars").getByText(p.detail).filter({ visible: true }).first()).toBeVisible();
  });

  test(`pillar "${p.name}" opens by Enter`, async ({ page }) => {
    await page.goto("./#pillars");
    await layer(page).focus();
    await page.keyboard.press("Enter");
    await expect(page.locator("#pillars").getByText(p.detail).filter({ visible: true }).first()).toBeVisible();
  });
}

test('each "Go deeper" link lands on an existing technical-guide anchor', async ({ page }) => {
  await page.goto("./");
  const hrefs = await page
    .locator("#pillars a")
    .filter({ hasText: /go deeper/i })
    .evaluateAll(as => as.map(a => (a as HTMLAnchorElement).href));
  expect(hrefs).toHaveLength(5);
  const hashes = hrefs.map(h => new URL(h).hash.slice(1));
  expect(new Set(hashes)).toEqual(new Set(TECH_ANCHORS));
  for (const href of hrefs) {
    const url = new URL(href);
    expect(url.pathname.endsWith("/technical-guide/") || url.pathname.endsWith("/technical-guide/index.html"), href).toBe(true);
    const res = await page.goto(href);
    expect(res?.status()).toBe(200);
    await expect(page.locator(`[id="${url.hash.slice(1)}"]`)).toHaveCount(1);
  }
});
