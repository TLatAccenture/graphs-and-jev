"use strict";

const response = body => Promise.resolve({ok: true, status: 200, json: async () => body});
const stages = names => names.map(([stage, title]) => ({stage, title}));
const catalogueStages = stages([
  ["gate", "Answer, clarify or reject"],
  ["enrich", "Pollutants"],
  ["query", "Breakdown"],
  ["discover", "Coverage"],
  ["rank", "Preference fit"],
  ["explain", "Explanation"]
]);

function staticCatalogue() {
  return response({summary: "Representative catalogue loaded.", sources: Array.from({length: 8}, (_, id) => ({id}))});
}

function dcceewResult(request, preference) {
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

function parkinsonsResult(request) {
  const q = request.toLowerCase();
  if (/dose|prescrib|diagnos/.test(q)) return {
    outcome: "reject",
    message: "This demonstration does not answer diagnosis, dose or prescribing questions.",
    stages: [{stage: "gate", title: "Research-scope safety gate", question: {criteria: {reject: "Dose, diagnosis and prescription requests are refused before retrieval."}}}],
    citations: [], graph: {nodes: [], edges: []},
    backend_label: "Static research rules", model: "research-fixture-v1", residency: "browser"
  };
  if (q.includes("approved") || q.includes("motor symptoms")) return {
    outcome: "clarify", message: "Narrow the question to a named compound or intervention.",
    explanation: {text: "The representative evidence does not support a specific regulatory-status relationship for this broad wording."},
    stages: [{stage: "gate", title: "Research-scope safety gate", question: {criteria: {clarify: "Broad questions are narrowed before an answer."}}}],
    citations: [], graph: {nodes: [], edges: []},
    backend_label: "Static research rules", model: "research-fixture-v1", residency: "browser"
  };
  const url = "https://europepmc.org/article/MED/16240353";
  return {
    outcome: "answer", message: "Published research reports an association between LRRK2 and Parkinson’s disease.",
    explanation: {text: "Representative evidence only. Open and assess the cited source before relying on this relationship."},
    stages: [{stage: "gate", title: "Research-scope safety gate", question: {criteria: {answer: "A retrieved source states a typed research relationship."}}}],
    citations: [{title: "Mutations in LRRK2 cause autosomal-dominant parkinsonism", url}],
    graph: {nodes: [{id: "LRRK2"}, {id: "PD"}], edges: [{from: "LRRK2", from_label: "Gene", ty: "associated_with", to: "Parkinson’s disease", to_label: "Condition", title: "Mutations in LRRK2 cause autosomal-dominant parkinsonism", url}]},
    backend_label: "Static research rules", model: "research-fixture-v1", residency: "browser"
  };
}

function staticPipeline({domain, request, preference}) {
  return response(domain === "parkinsons" ? parkinsonsResult(request) : dcceewResult(request, preference));
}
