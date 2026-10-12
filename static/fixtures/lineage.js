/* In-browser fixture for the lineage-migration demo. Deterministic rules over the sample estate;
   it calls no model and no system. Returns the Section 9.3 lineage shape from the handover. */
"use strict";
(() => {
  const LABEL = {
    kna1: "SAP KNA1", sfacct: "Salesforce Account", regret: "Regulatory return",
    bqreg: "customers.region", lkreg: "Looker: Regional sales", lkchurn: "Looker: Monthly churn",
    custupd: "COBOL job CUSTUPD", custrec_a: "CUSTREC copybook, version A", custrec_b: "CUSTREC copybook, version B"
  };
  const edge = (from, ty, to) => ({from, from_label: LABEL[from], ty, to, to_label: LABEL[to]});
  const stage = (id, title) => ({stage: id, title});
  const steps = list => list.map(([name, status]) => ({name, status}));

  /* Rules run in this order; the first match wins. Confidence values are illustrative. */
  const RULES = [
    {
      when: t => t.includes("prod") && (t.includes("copy") || t.includes("dev")),
      outcome: "reject", confidence: 0.99,
      message: "Refused before any tool call. Production customer data may not leave production.",
      why: ["The production PII policy governs the BigQuery customers table.", "The gate stopped the request, so no MCP server was called and nothing was read or copied."],
      stages: [stage("gate", "Gate: refused under the production PII policy")],
      edges: [], playbook: null, tools: []
    },
    {
      when: t => t.includes("rename") && t.includes("region"),
      outcome: "answer", confidence: 0.94,
      message: "Safe to proceed with a compatibility view. Two dashboards read this column.",
      why: ["customers.region is used by the Regional sales and Monthly churn dashboards.", "A view alias keeps the old name readable, so neither dashboard breaks.", "Finance owns both dashboards and is told before the change runs."],
      stages: [
        stage("gate", "Gate: in scope, object and change named"),
        stage("lineage", "Lineage: two downstream dashboards found"),
        stage("playbook", "Playbook: five of five steps done"),
        stage("hook", "Hook check: schema valid, no production write"),
        stage("eval", "Eval record written")
      ],
      edges: [edge("bqreg", "USED_BY", "lkreg"), edge("bqreg", "USED_BY", "lkchurn")],
      playbook: {skill: "migrate-table-sap-to-bigquery", steps: steps([
        ["Read lineage", "done"], ["Generate DDL", "done"], ["Create view alias region", "done"],
        ["Validate schema", "done"], ["Notify owner Finance", "done"]
      ])},
      tools: ["lineage.blast_radius", "bigquery.dry_run_ddl", "bigquery.create_view", "notify.owner"]
    },
    {
      when: t => t.includes("cobol") || t.includes("custupd"),
      outcome: "clarify", confidence: 0.58,
      message: "Two versions of the CUSTREC copybook exist. Which one does production use?",
      why: ["CUSTUPD reads the CUSTREC record layout, and the estate holds two versions of it.", "Converting against the wrong layout would produce a malformed schema, so the agent asks rather than guesses."],
      stages: [
        stage("gate", "Gate: in scope, job named"),
        stage("lineage", "Lineage: two copybook versions found"),
        stage("playbook", "Playbook: stopped after reading lineage"),
        stage("eval", "Eval record written")
      ],
      edges: [edge("custupd", "READS", "custrec_a"), edge("custupd", "READS", "custrec_b")],
      playbook: {skill: "convert-cobol-batch-job", steps: steps([
        ["Read lineage", "done"], ["Choose CUSTREC copybook version", "blocked"],
        ["Generate BigQuery SQL", "skipped"], ["Validate schema", "skipped"]
      ])},
      tools: ["lineage.blast_radius"]
    },
    {
      when: t => t.includes("drop"),
      outcome: "review", confidence: 0.41,
      message: "KNA1 syncs to Salesforce Account, which feeds the regulatory return. The return has no recorded owner.",
      why: ["The dependency is known; the person accountable for the regulatory return is not.", "A person decides before the table is dropped."],
      stages: [
        stage("gate", "Gate: in scope, table named"),
        stage("lineage", "Lineage: two-hop path to the regulatory return"),
        stage("playbook", "Playbook: waiting for an owner"),
        stage("hook", "Hook check: destructive change blocked without approval"),
        stage("eval", "Eval record written")
      ],
      edges: [edge("kna1", "SYNCS_TO", "sfacct"), edge("sfacct", "FEEDS", "regret")],
      playbook: {skill: "retire-source-table", steps: steps([
        ["Read lineage", "done"], ["Confirm downstream owners", "blocked"], ["Drop table KNA1", "skipped"]
      ])},
      tools: ["lineage.blast_radius"]
    },
    {
      when: t => t.includes("zlegacy"),
      outcome: "no_data", confidence: 0.9,
      message: "ZLEGACY_TMP is not in the lineage graph. Discover it before migrating it.",
      why: ["The lineage graph has no node for ZLEGACY_TMP, so its blast radius is unknown."],
      stages: [stage("gate", "Gate: in scope, table named"), stage("lineage", "Lineage: object not found")],
      edges: [], playbook: null, tools: ["lineage.blast_radius"]
    }
  ];
  const FALLBACK = {
    outcome: "clarify", confidence: 0.5,
    message: "Name the object and the change, for example \"rename KNA1.REGION\".",
    why: ["The request does not name an object in the estate and a change to make to it."],
    stages: [stage("gate", "Gate: object or change missing")],
    edges: [], playbook: null, tools: []
  };

  function runId(text) {
    let h = 5381;
    for (const c of text) h = ((h * 33) ^ c.codePointAt(0)) >>> 0;
    return "fixture-" + h.toString(16).padStart(8, "0");
  }

  function lineage({request}) {
    const text = String(request || "").toLowerCase();
    const rule = RULES.find(r => r.when(text)) || FALLBACK;
    const body = {
      outcome: rule.outcome,
      message: rule.message,
      explanation: {text: [rule.message, ...rule.why].join("\n")},
      stages: rule.stages,
      backend_label: "In-browser lineage rules",
      model: "lineage-fixture-v1",
      residency: "browser",
      /* Nothing is called in the browser, so calls, wall time, tokens and cost are zero, not measured. */
      totals: {decisions: 1, calls: 0, wall_ms: 0},
      graph: {edges: rule.edges},
      playbook: rule.playbook,
      eval: {
        run_id: runId(text), decision: rule.outcome, confidence: rule.confidence, tools_called: rule.tools,
        tokens_in: 0, tokens_out: 0, latency_ms: 0, cost_usd: 0
      }
    };
    return {ok: true, status: 200, json: async () => body};
  }

  window.SITE_FIXTURES = Object.assign(window.SITE_FIXTURES || {}, {lineage});
})();
