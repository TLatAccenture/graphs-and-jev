import type { Page } from "@playwright/test";
import { test, expect, expectOutcome, STEP_LABELS } from "./helpers";

// Section 10 item 6, rules from the WS4 brief. Lineage runs from fixtures on every target,
// including the Rust service (item 8: "the lineage page still runs from fixtures").
const EXAMPLES = [
  {
    button: "Rename KNA1.REGION to SALES_REGION in the BigQuery target",
    label: "Proceed",
    message: "Safe to proceed with a compatibility view. Two dashboards read this column.",
  },
  {
    button: "Migrate COBOL job CUSTUPD to BigQuery SQL",
    label: "Ask",
    message: "Two versions of the CUSTREC copybook exist. Which one does production use?",
  },
  {
    button: "Drop table KNA1 after cutover",
    label: "Escalate",
    message: "KNA1 syncs to Salesforce Account, which feeds the regulatory return. The return has no recorded owner.",
  },
  {
    button: "Copy the production customer table to dev for testing",
    label: "Refuse",
    message: "Refused before any tool call. Production customer data may not leave production.",
  },
  {
    button: "Migrate table ZLEGACY_TMP",
    label: "No data",
    message: "ZLEGACY_TMP is not in the lineage graph. Discover it before migrating it.",
  },
];

const PAGE = "lineage-migration/";

function watchApi(page: Page) {
  const calls: string[] = [];
  page.on("request", r => { if (new URL(r.url()).pathname.includes("/api/")) calls.push(r.url()); });
  return calls;
}

async function expectFixtureSource(page: Page, calls: string[]) {
  await expect(page.locator("main")).toContainText("in-browser fixtures");
  expect(calls, "lineage must not call the API").toEqual([]);
}

for (const ex of EXAMPLES) {
  test(`example "${ex.button}" returns ${ex.label}`, async ({ page }) => {
    const calls = watchApi(page);
    await page.goto(PAGE);
    await expectOutcome(page, STEP_LABELS, ex.label, () => page.locator("#demo").getByRole("button", { name: ex.button }).click());
    await expect(page.locator("main")).toContainText(ex.message);
    await expectFixtureSource(page, calls);
  });
}

test('free text "hello" returns Ask', async ({ page }) => {
  const calls = watchApi(page);
  await page.goto(PAGE);
  const box = page.locator("#demo textarea").first();
  await box.fill("hello");
  await expectOutcome(page, STEP_LABELS, "Ask", () => box.evaluate(t => (t as HTMLTextAreaElement).form!.requestSubmit()));
  await expect(page.locator("main")).toContainText("Name the object and the change");
  await expectFixtureSource(page, calls);
});
