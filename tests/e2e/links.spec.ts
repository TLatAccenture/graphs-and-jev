import { test, expect } from "./helpers";

// Section 10 item 2: crawl from the landing page; every internal link and asset resolves.
// Links are site-wide, so the crawl runs once (desktop project).
test("every internal link and asset resolves", async ({ page, request, baseURL }, info) => {
  test.skip(info.project.name !== "desktop", "Links do not depend on viewport.");
  test.setTimeout(120_000);
  const base = new URL(baseURL!);
  const internal = (u: URL) => u.origin === base.origin;
  const page_ = (u: URL) => u.origin + u.pathname;

  const queue = [base.href];
  const visited = new Set<string>();
  const ids = new Map<string, Set<string>>();
  const checked = new Map<string, number>();
  const anchors: { from: string; url: URL }[] = [];
  const errors: string[] = [];

  while (queue.length) {
    const url = queue.shift()!;
    if (visited.has(url)) continue;
    visited.add(url);
    await page.goto(url);
    const found = await page.evaluate(() => {
      const abs = (v: string | null, at = document.baseURI) => (v ? new URL(v, at).href : null);
      const links = [...document.querySelectorAll("a[href]")].map(a => abs(a.getAttribute("href")));
      const assets = [
        ...[...document.querySelectorAll("img[src], script[src], iframe[src]")].map(e => abs(e.getAttribute("src"))),
        ...[...document.querySelectorAll("link[href]")].map(e => abs(e.getAttribute("href"))),
        ...[...document.querySelectorAll("[srcset]")].flatMap(e =>
          e.getAttribute("srcset")!.split(",").map(s => abs(s.trim().split(/\s+/)[0])),
        ),
        ...[...document.querySelectorAll("use, image")].map(e => abs(e.getAttribute("href") || e.getAttribute("xlink:href"))),
      ];
      for (const sheet of document.styleSheets) {
        let rules: CSSRuleList;
        try { rules = sheet.cssRules; } catch { continue; }
        for (const r of rules) for (const m of r.cssText.matchAll(/url\("?([^")]+)"?\)/g)) {
          if (!m[1].startsWith("data:")) assets.push(abs(m[1], sheet.href || document.baseURI));
        }
      }
      return { links, assets, ids: [...document.querySelectorAll("[id]")].map(e => e.id) };
    });
    ids.set(page_(new URL(url)), new Set(found.ids));

    for (const raw of [...found.links, ...found.assets]) {
      if (!raw) continue;
      const u = new URL(raw);
      if (!/^https?:$/.test(u.protocol) || !internal(u)) continue;
      if (!u.pathname.startsWith(base.pathname)) {
        errors.push(`${url}: ${u.href} leaves the base path ${base.pathname}`);
        continue;
      }
      if (u.hash) anchors.push({ from: url, url: u });
      const key = page_(u);
      if (!checked.has(key)) {
        const res = await request.get(key);
        checked.set(key, res.status());
        if (res.status() !== 200) errors.push(`${url}: ${key} returned ${res.status()}`);
        else if (found.links.includes(raw) && (res.headers()["content-type"] || "").includes("text/html")) queue.push(key);
      }
    }
  }

  for (const { from, url } of anchors) {
    const id = decodeURIComponent(url.hash.slice(1));
    if (!id) continue;
    let known = ids.get(page_(url));
    if (!known && checked.get(page_(url)) === 200) {
      await page.goto(page_(url));
      known = new Set(await page.evaluate(() => [...document.querySelectorAll("[id]")].map(e => e.id)));
      ids.set(page_(url), known);
    }
    if (known && !known.has(id)) errors.push(`${from}: anchor ${url.href} has no element with id "${id}"`);
  }

  expect(visited.size, "pages crawled").toBeGreaterThanOrEqual(5);
  expect([...new Set(errors)]).toEqual([]);
});
