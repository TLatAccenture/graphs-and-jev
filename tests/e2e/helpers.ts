import { test as base, expect, type Locator, type Page } from "@playwright/test";

export const target = process.env.TARGET || "root";
/** Rust service or container: catalogue answers come from the live API, lineage from fixtures. */
export const isLive = target === "live";

export const test = base.extend<{ requireBaseURL: void }>({
  requireBaseURL: [
    async ({ baseURL }, use) => {
      test.skip(!baseURL, "BASE_URL is not set; live targets need the Rust service or the container URL.");
      await use();
    },
    { auto: true },
  ],
});
export { expect };

/** Every page in handover Section 1.2, relative to the base URL. */
export const PAGES = [
  "./",
  "technical-guide/",
  "environmental-agency/",
  "lineage-migration/",
  "little-bunnies/",
] as const;

export const TECH_ANCHORS = ["connectivity", "relational-context", "logic", "reflexes", "visibility-and-cost"];

/** Section 4.1 defaults. */
export const DEFAULT_CONFIG = {
  brand: {
    org: "Accenture",
    name: "Migration Acceleration with Google Cloud",
    tagline: "Stack migration and modernisation",
    kind: "Thought piece",
    disclaimer:
      "An Accenture thought piece on agentic migration and modernisation. It is not a product or service offer from Accenture or Google Cloud. Product names and marks belong to their owners.",
  },
  api: {
    mode: "fixtures",
    baseUrl: "",
    timeoutMs: 15000,
    routes: { health: "/api/health", catalogue: "/api/catalogue", pipeline: "/api/pipeline" },
    domains: { catalogue: "fixtures", lineage: "fixtures" } as Record<string, string>,
  },
};

type Config = typeof DEFAULT_CONFIG;

/** Answer /config.js (static file or Rust-rendered) with the given config instead. */
export async function useConfig(page: Page, edit: (c: Config) => void) {
  const config: Config = JSON.parse(JSON.stringify(DEFAULT_CONFIG));
  edit(config);
  await page.route(/\/config\.js(\?.*)?$/, route =>
    route.fulfill({
      contentType: "text/javascript; charset=utf-8",
      body: `window.SITE_CONFIG = ${JSON.stringify(config)};`,
    }),
  );
}

export const liveCatalogue = (c: Config) => {
  c.api.mode = "live";
  c.api.domains = { catalogue: "live", lineage: "fixtures" };
};

/**
 * Counts visible elements in <main> whose own text starts with an outcome label.
 * Controls, links and definition terms (the "how to read the result" legends) are ignored,
 * so a rise in the count means a result now shows that label.
 */
function labelCounts(page: Page, labels: string[]) {
  return page.evaluate(labels => {
    const skip = "button, a, label, legend, dt, option, select, textarea, summary, [role=tab], [role=button]";
    const counts: Record<string, number> = Object.fromEntries(labels.map(l => [l, 0]));
    for (const el of document.querySelectorAll("main *")) {
      if (el.closest(skip) || !el.checkVisibility()) continue;
      const own = [...el.childNodes].filter(n => n.nodeType === Node.TEXT_NODE).map(n => n.textContent).join("").trim();
      if (!own) continue;
      for (const l of labels) {
        const escaped = l.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
        if (new RegExp(`^(?:Outcome:\\s*)?${escaped}(?![\\w])`).test(own)) counts[l]++;
      }
    }
    return counts;
  }, labels);
}

/** Runs `action`, then expects `expected` to appear as a result label and no other label from `vocabulary` to appear. */
export async function expectOutcome(page: Page, vocabulary: string[], expected: string, action: () => Promise<unknown>) {
  const before = await labelCounts(page, vocabulary);
  await action();
  await expect
    .poll(async () => (await labelCounts(page, vocabulary))[expected], { message: `outcome label "${expected}" should appear` })
    .toBeGreaterThan(before[expected]);
  const after = await labelCounts(page, vocabulary);
  for (const l of vocabulary) {
    if (l !== expected) expect(after[l], `unexpected outcome label "${l}"`).toBeLessThanOrEqual(before[l]);
  }
}

/** Section 5.3 UI labels (landing, lineage). */
export const STEP_LABELS = ["Proceed", "Ask", "Refuse", "Escalate", "No data", "Service unavailable"];
/** Environmental Agency: the same vocabulary, with Answer in place of Proceed (presentation plan, item 7). */
export const CATALOGUE_LABELS = ["Answer", "Ask", "Refuse", "Escalate", "No data", "Service unavailable"];

/** Expands every <details> around the target, as a reader would before using it. */
export async function openDetails(target: Locator) {
  await target.evaluate(el => {
    for (let d = el.closest("details"); d; d = d.parentElement && d.parentElement.closest("details")) d.open = true;
  });
}

/** Scrolls the whole page so lazy-loaded images below the fold start loading. */
export async function scrollThrough(page: Page) {
  await page.evaluate(async () => {
    const step = Math.max(200, window.innerHeight / 2);
    for (let y = 0; y < document.documentElement.scrollHeight; y += step) {
      window.scrollTo(0, y);
      await new Promise(r => setTimeout(r, 30));
    }
    // Lazy images inside horizontally scrolling charts (390px) never cross the viewport otherwise.
    for (const img of document.querySelectorAll<HTMLImageElement>("img[loading=lazy]")) {
      if (img.complete) continue;
      img.scrollIntoView({ block: "center", inline: "center" });
      await new Promise(r => setTimeout(r, 30));
    }
    window.scrollTo(0, 0);
  });
}
