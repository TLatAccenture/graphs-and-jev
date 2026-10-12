/* Shared sample estate graph for the landing guided example and the technical-guide labs. ViewBox 0 0 400 380. */
"use strict";
window.ESTATE_GRAPH = {
  nodes: [
    {id: "kna1", label: "SAP KNA1", type: "Table", x: 60, y: 40, text: "KNA1 is the SAP customer master table."},
    {id: "region", label: "KNA1.REGION", type: "Column", x: 60, y: 130, text: "KNA1.REGION holds each customer's sales region code."},
    {id: "bqcust", label: "BigQuery customers", type: "Table", x: 200, y: 40, text: "The customers table in BigQuery is the migration target for KNA1."},
    {id: "bqreg", label: "customers.region", type: "Column", x: 200, y: 130, text: "customers.region is the BigQuery column mapped from KNA1.REGION."},
    {id: "dbx", label: "Databricks customer_sync", type: "Job", x: 340, y: 40, text: "customer_sync is a Databricks job that loads BigQuery customers nightly."},
    {id: "policy", label: "Policy: production PII", type: "Policy", x: 340, y: 130, text: "Production customer data may not be copied outside production."},
    {id: "lkreg", label: "Looker: Regional sales", type: "Dashboard", x: 120, y: 215, text: "The Regional sales dashboard reports revenue by region."},
    {id: "lkchurn", label: "Looker: Monthly churn", type: "Dashboard", x: 280, y: 215, text: "The Monthly churn dashboard reports customer churn by region."},
    {id: "finance", label: "Finance", type: "Team", x: 360, y: 290, text: "Finance owns the regional sales and churn dashboards."},
    {id: "crm", label: "Legacy CRM sync", type: "Job", x: 40, y: 300, text: "Legacy CRM sync is a scheduled job registered in ServiceNow."},
    {id: "sfacct", label: "Salesforce Account", type: "Object", x: 140, y: 300, text: "Salesforce Account holds customer accounts for the sales team."},
    {id: "regret", label: "Regulatory return", type: "Report", x: 250, y: 345, text: "The quarterly regulatory return draws on Salesforce account data."},
    {id: "xlsx3", label: "Pricing FY26 v3.xlsx", type: "File", x: 120, y: 360, text: "Pricing FY26 v3.xlsx is a SharePoint workbook of list prices."},
    {id: "xlsxf", label: "Pricing FY26 final.xlsx", type: "File", x: 300, y: 360, text: "Pricing FY26 final.xlsx is a SharePoint workbook of list prices."},
    {id: "price", label: "Price list", type: "Entity", x: 210, y: 290, text: "The price list sets list prices applied to Salesforce quotes."}
  ],
  edges: [
    ["kna1", "HAS_COLUMN", "region"], ["bqcust", "HAS_COLUMN", "bqreg"], ["region", "MAPS_TO", "bqreg"],
    ["dbx", "WRITES", "bqcust"], ["bqreg", "USED_BY", "lkreg"], ["bqreg", "USED_BY", "lkchurn"],
    ["finance", "OWNS", "lkreg"], ["finance", "OWNS", "lkchurn"], ["policy", "GOVERNS", "bqcust"],
    ["crm", "FEEDS", "sfacct"], ["sfacct", "FEEDS", "regret"], ["kna1", "SYNCS_TO", "sfacct"],
    ["xlsx3", "DESCRIBES", "price"], ["xlsxf", "DESCRIBES", "price"], ["price", "APPLIES_TO", "sfacct"]
  ],
  colours: {
    Table: "var(--purple)", Column: "var(--purple-mid)", Job: "var(--purple-mid)",
    Dashboard: "var(--black)", Report: "var(--black)", File: "#7a7a7a",
    Policy: "var(--reject)", Team: "var(--purple-deep)", Object: "var(--purple)", Entity: "var(--purple)"
  },
  node(id) { return this.nodes.find(n => n.id === id); },
  neighbours(id) {
    const out = [];
    for (const [a, r, b] of this.edges) {
      if (a === id) out.push({id: b, rel: r, dir: "out"});
      if (b === id) out.push({id: a, rel: r, dir: "in"});
    }
    return out;
  },
  tokens(n) { return Math.ceil(n.text.split(/\s+/).length * 1.3); }
};
