const assert = require("node:assert/strict");
const fs = require("node:fs");
const vm = require("node:vm");

const window = {};
vm.runInNewContext(fs.readFileSync("static/fixtures/catalogue.js", "utf8"), {
  window,
  performance: { now: () => 0 },
});

const cases = [
  ["hi", "clarify", "Hello!"],
  ["Hello there!", "clarify", "Hello!"],
  ["good evening team", "clarify", "Hello!"],
  ["thanks", "clarify", "welcome"],
  ["Greetings team, annual CO2 for Australia in 2024", "answer"],
  ["tell me a joke", "reject", "Sorry"],
  ["write a haiku", "reject", "Sorry"],
  ["hourly ozone by street", "reject", "Sorry"],
  ["annual CO2 for Australia in 2024", "answer"],
  ["pollution", "clarify"],
  ["emissions", "clarify"],
  ["data", "clarify"],
  ["everything", "clarify"],
];

(async () => {
  for (const [request, outcome, marker] of cases) {
    const body = await (await window.SITE_FIXTURES.catalogue({ request, preference: "" })).json();
    assert.equal(body.outcome, outcome, `${request}: ${JSON.stringify(body)}`);
    if (marker) assert.match(body.message, new RegExp(marker), request);
  }
})().catch(error => { console.error(error); process.exit(1); });
