/* The only code in static/ that talks to an API or to fixtures. Pages use window.siteApi. */
"use strict";
(() => {
  const config = () => window.SITE_CONFIG || {};
  const api = () => config().api || {};

  function mode(domain) {
    const settings = api();
    return (domain && settings.domains && settings.domains[domain]) || settings.mode || "fixtures";
  }

  function failure(status, detail) {
    return {ok: false, status, json: async () => ({outcome: "error", detail})};
  }

  function source(domain) {
    if (mode(domain) !== "live") return "in-browser fixtures";
    const base = api().baseUrl;
    return "live API at " + (base ? new URL(base, location.href).host : location.host);
  }

  async function live(route, init) {
    const settings = api();
    const controller = new AbortController();
    // Left running on success so the abort also bounds reading the body.
    setTimeout(() => controller.abort(), settings.timeoutMs || 15000);
    let res;
    try {
      res = await fetch((settings.baseUrl || "") + settings.routes[route], {...init, signal: controller.signal, credentials: "omit"});
    } catch (e) {
      return failure(0, controller.signal.aborted ? "The service did not respond in time." : "The service could not be reached.");
    }
    if (res.ok) return res;
    let detail = "The service returned HTTP " + res.status + ".";
    try { detail = (await res.json()).detail || detail; } catch (e) { /* non-JSON error body */ }
    return failure(res.status, detail);
  }

  function fixture(domain, method, payload) {
    const fn = window.SITE_FIXTURES && window.SITE_FIXTURES[domain];
    const target = method ? fn && fn[method] : fn;
    if (typeof target !== "function") return Promise.resolve(failure(0, "No in-browser fixture is loaded for " + domain + "."));
    return Promise.resolve(target(payload));
  }

  window.siteApi = {
    mode,
    source,
    health() {
      if (mode() !== "live") return Promise.resolve({ok: true, status: 200, json: async () => ({status: "ok", mode: "fixtures"})});
      return live("health", {method: "GET"});
    },
    catalogue() {
      if (mode("catalogue") !== "live") return fixture("catalogue", "list");
      return live("catalogue", {method: "GET"});
    },
    pipeline({domain, request, preference}) {
      const payload = {domain, request, preference: preference || null};
      if (mode(domain) !== "live") return fixture(domain, null, payload);
      return live("pipeline", {
        method: "POST",
        headers: {"Content-Type": "application/json"},
        body: JSON.stringify(payload)
      });
    }
  };

  /* Brand binding. Pages ship default text; this replaces it from SITE_CONFIG.brand.
     <body data-page-title="X"> gives "X · {name}"; an empty value marks the landing page. */
  document.addEventListener("DOMContentLoaded", () => {
    const brand = config().brand || {};
    document.querySelectorAll("[data-brand]").forEach(node => {
      const value = brand[node.dataset.brand];
      if (typeof value === "string") node.textContent = value;
    });
    const page = document.body.dataset.pageTitle;
    if (page === undefined || !brand.name) return;
    document.title = page ? page + " · " + brand.name : brand.name + (brand.tagline ? ": " + brand.tagline : "");
  });
})();
