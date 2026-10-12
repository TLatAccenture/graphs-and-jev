import type { Page } from "@playwright/test";
import { test, expect, expectOutcome, CATALOGUE_LABELS, isLive, useConfig, liveCatalogue } from "./helpers";

const PAGE = "environmental-agency/";
const examples = (page: Page) => page.locator("#demo [data-q]");
const pipelineCalls = (page: Page) => {
  const calls: { body: Record<string, unknown>; status: number }[] = [];
  page.on("response", r => {
    if (new URL(r.url()).pathname.endsWith("/api/pipeline")) calls.push({ body: r.request().postDataJSON(), status: r.status() });
  });
  return calls;
};

// Section 10 item 5 (fixtures), and item 8's unmocked check against the Rust service (TARGET=live).
// The mode is read from the page as served: the Rust service defaults the catalogue to the live API,
// and an event build (API_MODE_CATALOGUE=fixtures, presentation plan item 2) keeps it in the browser.
const EXPECTED = ["Answer", "Answer", "Ask", "Refuse"];
for (const [i, label] of EXPECTED.entries()) {
  test(`example ${i + 1} returns ${label}`, async ({ page }) => {
    const calls = pipelineCalls(page);
    await page.goto(PAGE);
    await expect(examples(page)).toHaveCount(4);
    const live = await page.evaluate(() => (window as any).siteApi.mode("catalogue") === "live");
    if (!isLive) expect(live, "static targets serve in-browser fixtures").toBe(false);
    await expectOutcome(page, CATALOGUE_LABELS, label, () => examples(page).nth(i).click());
    await expect(page.locator("main")).toContainText(live ? "live API at" : "in-browser fixtures");
    if (live) {
      expect(calls.map(c => c.status)).toEqual([200]);
    } else {
      expect(calls, "fixtures mode must not call the API").toEqual([]);
    }
  });
}

// Section 10 item 8: live mode with route interception.
const MOCK_DATASET = "Mock live air quality dataset";
const MOCK_ANSWER = {
  outcome: "answer",
  explanation: { text: "One mocked live dataset covers the request.\nReturned by the intercepted API route." },
  stages: [{ stage: "gate", title: "Answer, clarify or reject" }, { stage: "discover", title: "Coverage" }],
  query: { pollutants: ["PM2_5"], levels: { GEO: "country", TIME: "year" } },
  solutions: [{ id: 1, cells: 6, sources: [{ name: MOCK_DATASET, publisher: "official", updated: 2026 }], fit: null, notes: [] }],
  backend_label: "Mock live service",
  model: "mock-v1",
  residency: "test",
  totals: { decisions: 2, calls: 2, wall_ms: 1 },
};

test("live mode renders a valid API response and names the live API", async ({ page }) => {
  await useConfig(page, liveCatalogue);
  const bodies: unknown[] = [];
  await page.route("**/api/catalogue", r => r.fulfill({ json: { summary: "Mock live catalogue.", sources: [{ id: 1 }] } }));
  await page.route("**/api/pipeline", r => {
    bodies.push(r.request().postDataJSON());
    return r.fulfill({ json: MOCK_ANSWER });
  });
  await page.goto(PAGE);
  await expectOutcome(page, CATALOGUE_LABELS, "Answer", () => examples(page).first().click());
  await expect(page.locator("main")).toContainText(MOCK_DATASET);
  await expect(page.locator("main")).toContainText("live API at");
  expect(bodies).toHaveLength(1);
  expect(Object.keys(bodies[0] as object).sort(), "Section 4.3 request body").toEqual(["domain", "preference", "request"]);
  expect((bodies[0] as { domain: string }).domain).toBe("catalogue");
});

test("live mode with a 500 shows Service unavailable and no datasets", async ({ page }) => {
  await useConfig(page, liveCatalogue);
  const fail = { status: 500, json: { detail: "Mocked failure." } };
  await page.route("**/api/catalogue", r => r.fulfill(fail));
  await page.route("**/api/pipeline", r => r.fulfill(fail));
  await page.goto(PAGE);
  const first = examples(page).first();
  const [q, p] = [await first.getAttribute("data-q"), await first.getAttribute("data-p")];
  // Whatever the in-browser fixture would have answered must not appear: no fixture fallback.
  const fixtureNames: string[] = await page.evaluate(async ([request, preference]) => {
    const fixture = (window as any).SITE_FIXTURES?.catalogue;
    if (!fixture) return [];
    const body = await (await fixture({ domain: "catalogue", request, preference })).json();
    return (body.solutions || []).flatMap((s: any) => (s.sources || []).map((x: any) => x.name));
  }, [q, p]);
  const mentions = () => page.locator("main").evaluate((m, names) => names.filter(n => m.innerText.includes(n)).length, fixtureNames);
  const before = await mentions();
  await expectOutcome(page, CATALOGUE_LABELS, "Service unavailable", () => first.click());
  expect(await mentions(), "fixture datasets shown after a live failure").toBeLessThanOrEqual(before);
  await expect(page.locator("main")).not.toContainText(MOCK_DATASET);
});

// Ported from tests/browser_dcceew_race.py: a second example clicked while the first request is
// in flight must not start a request or overwrite the first result.
test("overlapping requests keep the first result (stale-result race)", async ({ page }) => {
  await useConfig(page, liveCatalogue);
  const requests: string[] = [];
  await page.route("**/api/catalogue", r => r.fulfill({ json: { summary: "Test catalogue.", sources: [] } }));
  await page.route("**/api/pipeline", async r => {
    requests.push(r.request().postDataJSON().request);
    await new Promise(res => setTimeout(res, 1000));
    await r.fulfill({ json: { outcome: "clarify", message: "First request completed.", stages: [] } });
  });
  await page.goto(PAGE);
  const scenarios = examples(page);
  const form = page.locator("form", { has: page.locator("textarea") });
  const firstRequest = await scenarios.nth(0).getAttribute("data-q");

  await scenarios.nth(0).click();
  await expect(form).toHaveAttribute("aria-busy", "true");
  await scenarios.nth(1).evaluate(b => (b as HTMLButtonElement).click());
  await expect(form).not.toHaveAttribute("aria-busy");

  expect(requests).toEqual([firstRequest]);
  await expect(scenarios.nth(0)).toHaveAttribute("aria-pressed", "true");
  await expect(scenarios.nth(1)).toHaveAttribute("aria-pressed", "false");
  await expect(scenarios.nth(0)).toBeEnabled();
  await expect(scenarios.nth(1)).toBeEnabled();
  await expect(form.locator("[aria-live]")).toHaveAttribute("data-state", "done");
  await expect(page.locator("main")).toContainText("First request completed.");
});
