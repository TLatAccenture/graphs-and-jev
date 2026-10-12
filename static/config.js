/* Defaults for static serving. The Rust service renders /config.js from env vars. Keep keys stable. */
window.SITE_CONFIG = {
  brand: {
    org: "Accenture",
    name: "The MCP Shed",
    tagline: "Agentic migration and modernisation",
    kind: "Thought piece",
    disclaimer: "A personal research thought piece on agentic migration and modernisation. It is not an official Accenture or Google Cloud product, a service offer or a statement of capability. Product names and marks belong to their owners."
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
