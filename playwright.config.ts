import { defineConfig } from "@playwright/test";

/*
 * TARGET selects where the pages come from (handover Section 7, WS7):
 *   root     python3 -m http.server serving static/ at /            (fixtures)
 *   subpath  python3 -m http.server serving the repo root, pages at /static/ (fixtures)
 *   live     BASE_URL from the environment: the Rust service (cargo run) or the WS6 container.
 *            Every test skips when BASE_URL is unset. Set CONTAINER=1 for the container-only checks.
 */
const target = process.env.TARGET || "root";

const servers = {
  root: { port: 18170, dir: "static", path: "/" },
  subpath: { port: 18171, dir: ".", path: "/static/" },
} as const;

const local = target === "root" || target === "subpath" ? servers[target] : undefined;
const baseURL = local
  ? `http://127.0.0.1:${local.port}${local.path}`
  : process.env.BASE_URL && process.env.BASE_URL.replace(/\/?$/, "/");

export default defineConfig({
  testDir: "tests/e2e",
  outputDir: "test-results",
  timeout: 30_000,
  expect: { timeout: 7_000 },
  fullyParallel: true,
  retries: 0,
  reporter: [["list"]],
  use: {
    browserName: "chromium",
    baseURL,
    actionTimeout: 10_000,
    navigationTimeout: 15_000,
    trace: "retain-on-failure",
  },
  metadata: { target },
  projects: [
    { name: "desktop", use: { viewport: { width: 1280, height: 800 } } },
    {
      name: "mobile",
      use: { viewport: { width: 390, height: 844 }, deviceScaleFactor: 3, isMobile: true, hasTouch: true },
    },
  ],
  webServer: local && {
    // python3 -m http.server, with the listen backlog raised from 5: parallel browsers otherwise get
    // ERR_CONNECTION_RESET / REFUSED from the stock server.
    command:
      `python3 -c "import functools, http.server as s; s.ThreadingHTTPServer.request_queue_size = 256; ` +
      `s.ThreadingHTTPServer(('127.0.0.1', ${local.port}), functools.partial(s.SimpleHTTPRequestHandler, directory='${local.dir}')).serve_forever()"`,
    url: baseURL,
    reuseExistingServer: !process.env.CI,
    stdout: "ignore",
    stderr: "ignore",
  },
});
