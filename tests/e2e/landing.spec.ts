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
    // The page opens with the first scenario selected; start from a different one so the label count changes.
    const other = SCENARIOS.find(o => o.label !== s.label)!;
    await example.getByRole("button", { name: other.button }).click();
    await expectOutcome(page, STEP_LABELS, s.label, () => example.getByRole("button", { name: s.button }).click());
    await expect(example).toContainText(s.answer);
  });
}

// Section 10 item 4, pillar detail text as published on the landing page.
const PILLARS = [
  { name: "Existing Stack", detail: "Nothing is ripped out on day one" },
  { name: "MCP Tool Shed", detail: "scoped, short-lived credentials" },
  { name: "System 1 (Jev)", detail: "it stops an unsure output becoming an action" },
  { name: "ADK with Gemini", detail: "planned System 2 orchestration layer" },
  { name: "Modernisation Springboard", detail: "cost per migrated object" },
];

const CONTROLS = "button, [role=button], [role=tab], summary, a[href^='#'], [tabindex='0']";

for (const p of PILLARS) {
  const layer = (page: Page) =>
    page.locator("#pillars").locator(CONTROLS).filter({ hasText: p.name }).first();

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
    await page.goto("about:blank");
    const res = await page.goto(href);
    expect(res?.status()).toBe(200);
    await expect(page.locator(`[id="${url.hash.slice(1)}"]`)).toHaveCount(1);
  }
});

test("status ribbon separates the live service from the planned reference architecture", async ({ page }) => {
  await page.goto("./");
  const status = page.locator("#implementation-status");
  await expect(status).toBeVisible();
  await expect(status.locator('[data-state="live"]')).toContainText("Live today");
  await expect(status.locator('[data-state="reference"]')).toContainText("Google Cloud reference architecture");
  await expect(status).toContainText("The Google Cloud reference architecture is planned and not live in this demo.");
});

test("there are exactly five accessible pillar controls in the published order", async ({ page }) => {
  await page.goto("./#pillars");
  const controls = page.locator("#pillars .stack-layer");
  await expect(controls).toHaveCount(5);
  await expect(controls.locator("strong")).toHaveText(PILLARS.map(p => p.name));
  for (let i = 0; i < PILLARS.length; i++) {
    await expect(controls.nth(i)).toHaveAttribute("aria-controls", /pillar-/);
    await controls.nth(i).focus();
    await expect(controls.nth(i)).toBeFocused();
    expect(
      await controls.nth(i).evaluate(el => {
        const style = getComputedStyle(el);
        return style.outlineStyle !== "none" && parseFloat(style.outlineWidth) > 0;
      }),
      `${PILLARS[i].name} has a visible focus indicator`,
    ).toBe(true);
  }
});

const PHASES = ["assess", "plan", "deploy", "optimise"] as const;
const LANES = [
  { id: "documents", text: ["Unstructured documents", "Storage Transfer Service", "Cloud Storage", "Google Cloud RAG Engine"] },
  { id: "tabular", text: ["Spreadsheets and tabular files", "Cloud Storage", "BigQuery"] },
  { id: "relational", text: ["Operational relational data", "Database Migration Service", "Datastream", "AlloyDB AI", "TypeSafe's hosted Jev API"] },
  { id: "applications", text: ["Applications and jobs", "Migrate to Virtual Machines", "Compute Engine", "Cloud Run"] },
] as const;

test("one Gemini Migration Agent orchestrates four gated phases, with approval between Plan and Deploy", async ({ page }) => {
  await page.goto("./#google-cloud-architecture");
  const plane = page.locator("#google-cloud-architecture");
  await expect(plane).toContainText("Planned reference architecture");
  await expect(plane).toContainText("not live in this demo");
  const orchestrator = plane.locator("[data-orchestrator]");
  await expect(orchestrator).toHaveCount(1);
  await expect(orchestrator).toBeVisible();
  await expect(orchestrator).toContainText("Gemini Migration Agent");
  await expect(orchestrator).toContainText("Built with Agent Development Kit (ADK)");

  // Content order, which is also the reading order without CSS and in print.
  const order = await plane.locator(".phases > li").evaluateAll(items =>
    items.map(item => item.getAttribute("data-phase") ?? item.getAttribute("data-checkpoint")),
  );
  expect(order).toEqual(["assess", "plan", "human-approval", "deploy", "optimise"]);

  for (const phase of PHASES) {
    const card = plane.locator(`[data-phase="${phase}"]`);
    await expect(card).toBeVisible();
    for (const field of ["Inputs", "Agent action", "Jev gate", "Output", "State"]) {
      await expect(card.locator("dt", { hasText: field }).first()).toBeVisible();
    }
    await expect(card).toContainText("Planned reference");
  }
  await expect(plane.locator("[data-checkpoint='human-approval']")).toContainText("Person approves the plan");
  await expect(plane.locator("[data-phase='deploy']")).toContainText("It does not move bytes.");
});

test("four representative lanes show source, payload, service, destination and evidence in order", async ({ page }) => {
  await page.goto("./#google-cloud-architecture");
  const plane = page.locator("#google-cloud-architecture");
  await expect(plane).toContainText("Destination is chosen by workload. Not all data belongs in BigQuery.");
  const lanes = plane.locator("[data-lane]");
  await expect(lanes).toHaveCount(LANES.length);
  expect(await lanes.evaluateAll(items => items.map(item => item.getAttribute("data-lane")))).toEqual(LANES.map(l => l.id));
  for (const [index, lane] of LANES.entries()) {
    const row = lanes.nth(index);
    await expect(row).toContainText("Representative reference path");
    const fields = await row.locator(".lane-fields > div > dt").allTextContents();
    expect(fields).toEqual(["Source", "What moves", "Movement or service", "Destination", "Validation or evidence"]);
    for (const text of lane.text) await expect(row).toContainText(text);
  }
});

test("every Google Cloud product mark has adjacent readable product text and loads locally", async ({ page }) => {
  const problems: string[] = [];
  page.on("requestfailed", request => problems.push(`${request.url()} ${request.failure()?.errorText}`));
  page.on("console", message => {
    if (message.type() === "error") problems.push(message.text());
  });
  await page.goto("./");
  const marks = page.locator("img[data-product-mark]");
  await expect(marks).toHaveCount(10);
  for (let i = 0; i < 10; i++) {
    const mark = marks.nth(i);
    await expect(mark).toHaveAttribute("src", /^(?:assets\/gcp-icons\/|assets\/engine-logos\/)/);
    expect(
      await mark.evaluate(img => {
        const product = img.closest("[data-product]");
        return Boolean(product && product.textContent?.trim() && (img as HTMLImageElement).naturalWidth > 0);
      }),
      `mark ${i + 1} has adjacent visible text and loaded pixels`,
    ).toBe(true);
  }
  expect(problems).toEqual([]);
});

test("Why now markers link to the compact source notes below the prose", async ({ page }) => {
  await page.goto("./#problem");
  const markers = page.locator("#problem a.source-marker");
  await expect(markers).toHaveCount(7);
  const hrefs = await markers.evaluateAll(links => links.map(link => link.getAttribute("href")));
  expect(new Set(hrefs).size).toBe(7);
  for (const href of hrefs) {
    expect(href).toMatch(/^#why-source-\d+$/);
    await expect(page.locator(href!)).toBeVisible();
  }
  await expect(page.locator("#why-sources")).toContainText("secondary source");
});

test("status, architecture rail and source notes stay inside the viewport", async ({ page }) => {
  await page.goto("./");
  for (const selector of ["#implementation-status", "#google-cloud-architecture", "#why-sources"]) {
    const overflow = await page.locator(selector).evaluate(element => {
      const box = element.getBoundingClientRect();
      return { left: box.left, right: box.right, viewport: document.documentElement.clientWidth };
    });
    expect(overflow.left, `${selector} left edge`).toBeGreaterThanOrEqual(-1);
    expect(overflow.right, `${selector} right edge`).toBeLessThanOrEqual(overflow.viewport + 1);
  }
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth),
    "page horizontal overflow",
  ).toBeLessThanOrEqual(0);
});

test("the first viewport includes a visible live and reference status cue", async ({ page }) => {
  await page.goto("./");
  const visiblePixels = await page.locator("#implementation-status").evaluate(element => {
    const box = element.getBoundingClientRect();
    return Math.max(0, Math.min(box.bottom, window.innerHeight) - Math.max(box.top, 0));
  });
  expect(visiblePixels).toBeGreaterThanOrEqual(24);
});

test("new reference links keep 44px targets and the mobile header targets do not overlap", async ({ page }) => {
  await page.goto("./");
  for (const selector of [
    "#problem .source-marker",
    "#google-cloud-architecture .product",
    "#example .hops button",
  ]) {
    const boxes = await page.locator(selector).evaluateAll(elements =>
      elements.map(element => {
        const box = element.getBoundingClientRect();
        return { width: box.width, height: box.height };
      }),
    );
    expect(boxes.length, `${selector} exists`).toBeGreaterThan(0);
    for (const box of boxes) {
      expect(box.width, `${selector} target width`).toBeGreaterThanOrEqual(44);
      expect(box.height, `${selector} target height`).toBeGreaterThanOrEqual(44);
    }
  }

  const targets = await page.locator(".site-header").evaluate(element => {
    const brand = element.querySelector(".brand")!.getBoundingClientRect();
    const cta = element.querySelector(".site-nav .cta")!.getBoundingClientRect();
    return { brandRight: brand.right, ctaLeft: cta.left };
  });
  expect(targets.ctaLeft - targets.brandRight, "brand and CTA hit-area gap").toBeGreaterThanOrEqual(1);
});

test("print keeps status labels, architecture roles and Why now source notes visible", async ({ page }) => {
  await page.emulateMedia({ media: "print" });
  await page.goto("./");
  for (const selector of [
    "#implementation-status [data-state='live']",
    "#implementation-status [data-state='reference']",
    "#google-cloud-architecture [data-orchestrator]",
    "#google-cloud-architecture [data-phase='deploy']",
    "#google-cloud-architecture [data-checkpoint='human-approval']",
    "#google-cloud-architecture [data-lane='applications']",
    "#why-sources",
  ]) {
    await expect(page.locator(selector)).toBeVisible();
  }

  const guided = await page.locator("#example").evaluate(element => {
    const result = element.querySelector(".result")!.getBoundingClientRect();
    const graph = element.querySelector(".guided-graph")!.getBoundingClientRect();
    return { resultTop: result.top, resultBottom: result.bottom, graphTop: graph.top };
  });
  expect(guided.graphTop, "print graph begins alongside the decision result").toBeLessThan(guided.resultBottom);
});

test("print uses fewer than the historical twelve A4 pages", async ({ page }) => {
  await page.goto("./");
  const pdf = await page.pdf({ format: "A4", printBackground: false });
  const pageCount = pdf.toString("latin1").match(/\/Type\s*\/Page\b/g)?.length ?? 0;
  expect(pageCount).toBeGreaterThan(0);
  expect(pageCount).toBeLessThan(12);
});

test("reduced motion leaves the status, architecture and source notes stable and readable", async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("./");
  const report = await page.evaluate(() => {
    const root = document.querySelector("#implementation-status, #google-cloud-architecture, #why-sources");
    const elements = [
      ...document.querySelectorAll("#implementation-status, #implementation-status *, #google-cloud-architecture, #google-cloud-architecture *, #why-sources, #why-sources *"),
    ];
    return {
      root: Boolean(root),
      hidden: elements.filter(element => {
        const style = getComputedStyle(element);
        return style.display === "none" || style.visibility === "hidden" || style.opacity === "0";
      }).length,
      animations: document
        .getAnimations()
        .filter(animation => elements.includes((animation.effect as KeyframeEffect | null)?.target as Element)).length,
    };
  });
  expect(report.root).toBe(true);
  expect(report.hidden).toBe(0);
  expect(report.animations).toBe(0);
});
