/* In-browser fixture for the Environmental Agency catalogue. Mirrors the catalogue pipeline response shape. */
"use strict";
(() => {
  const response = body => ({ok: true, status: 200, json: async () => body});
  const catalogueStages = [
    ["gate", "Answer, clarify or reject"],
    ["enrich", "Pollutants"],
    ["query", "Breakdown"],
    ["discover", "Coverage"],
    ["rank", "Preference fit"],
    ["explain", "Explanation"]
  ].map(([stage, title]) => ({stage, title}));

  function catalogueResult(request, preference) {
    const q = request.toLowerCase();
    if (q.trim() === "emissions") return {
      outcome: "clarify", message: "Name a pollutant and a breakdown such as country and year.",
      stages: catalogueStages.slice(0, 1)
    };
    if (q.includes("street") || q.includes("1990") || q.includes("hourly")) return {
      outcome: "reject", message: "The request requires unsupported granularity or years outside 2015–2025.",
      stages: catalogueStages.slice(0, 1)
    };
    const particulate = q.includes("particulate");
    const pollutants = particulate ? ["PM2_5", "PM10"] : q.includes("co2") ? ["CO2"] : ["PM2_5"];
    const level = q.includes("region") ? "region" : "country";
    return {
      outcome: "answer",
      explanation: {text: `Two representative official datasets cover ${pollutants.join(" and ")} by ${level}.\nThe graph matched the pollutant, geography and requested years.\nJev ranked only options whose coverage the graph established.`},
      stages: catalogueStages,
      query: {pollutants, levels: {GEO: level, TIME: "year"}},
      solutions: [{
        id: 1, cells: particulate ? 14 : 6,
        sources: [
          {name: "National Air Quality Observations", publisher: "official", updated: 2025},
          {name: "Environmental Indicators Catalogue", publisher: "official", updated: 2025}
        ],
        fit: preference ? {top: "3", legend: {"3": "strong fit"}} : null,
        notes: ["Illustrative catalogue data; confirm against the authoritative publisher before use."]
      }],
      backend_label: "Static catalogue rules", model: "catalogue-rules-v1", residency: "browser",
      totals: {decisions: 5, calls: 5, wall_ms: 0}
    };
  }

  const catalogue = ({request, preference}) => response(catalogueResult(request || "", preference));
  catalogue.list = () => response({
    summary: "Representative catalogue loaded.",
    sources: Array.from({length: 8}, (_, id) => ({id}))
  });

  window.SITE_FIXTURES = Object.assign(window.SITE_FIXTURES || {}, {catalogue});
})();
