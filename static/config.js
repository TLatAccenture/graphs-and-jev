/* Defaults for static serving. The Rust service renders /config.js from env vars. Keep keys stable. */
window.SITE_CONFIG = {
  brand: {
    org: "Accenture",
    name: "Migration Acceleration with Google Cloud",
    tagline: "Stack migration and modernisation",
    kind: "Thought piece",
    disclaimer: "An Accenture thought piece on agentic migration and modernisation. It is not a product or service offer from Accenture or Google Cloud. Product names and marks belong to their owners."
  },
  api: {
    mode: "fixtures",            // "fixtures" | "live"
    baseUrl: "",                 // "" means same origin. e.g. "https://api.example.run.app"
    timeoutMs: 15000,
    routes: {
      health: "/api/health",
      catalogue: "/api/catalogue",
      pipeline: "/api/pipeline"
    },
    domains: {                   // per-domain override; omit a key to inherit api.mode
      catalogue: "fixtures",
      lineage: "fixtures"
    }
  }
};
