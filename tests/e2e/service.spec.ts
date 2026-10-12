import vm from "node:vm";
import { test, expect, isLive } from "./helpers";

// Section 10 item 10, for the Rust service and the WS6 container (TARGET=live, BASE_URL set).
// Pass the values given to `docker run -e ...` as EXPECT_BRAND_NAME (and optionally EXPECT_API_MODE)
// to check /config.js reflects them, correctly escaped.
test.describe("service routes", () => {
  test.skip(!isLive, "Only the Rust service and the container render /config.js and serve /api/*.");

  for (const path of ["", "config.js", "api/health", "api/ready"]) {
    test(`GET /${path} returns 200`, async ({ request }, info) => {
      test.skip(info.project.name !== "desktop", "Not viewport dependent.");
      const res = await request.get(path);
      expect(res.status()).toBe(200);
    });
  }

  test("/config.js is valid JavaScript and reflects the environment", async ({ request }, info) => {
    test.skip(info.project.name !== "desktop", "Not viewport dependent.");
    const res = await request.get("config.js");
    expect(res.headers()["content-type"]).toMatch(/^text\/javascript/);
    expect(res.headers()["cache-control"]).toContain("no-cache");
    const sandbox: { window: { SITE_CONFIG?: any } } = { window: {} };
    vm.runInNewContext(await res.text(), sandbox);
    const config = sandbox.window.SITE_CONFIG;
    expect(config?.brand?.name).toBeTruthy();
    expect(Object.keys(config.api.routes).sort()).toEqual(["catalogue", "health", "pipeline"]);
    if (process.env.EXPECT_BRAND_NAME !== undefined) expect(config.brand.name).toBe(process.env.EXPECT_BRAND_NAME);
    if (process.env.EXPECT_API_MODE !== undefined) expect(config.api.mode).toBe(process.env.EXPECT_API_MODE);
  });
});
