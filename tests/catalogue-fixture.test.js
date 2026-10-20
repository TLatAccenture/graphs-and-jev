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
  ["good day", "clarify", "Hello!"],
  ["test", "clarify", "Hello!"],
  ["what can you do?", "clarify", "Hello!"],
  ["Hi, what can you do?", "clarify", "Hello!"],
  ["hey, how does this work?", "clarify", "Hello!"],
  ["hello, can you help me?", "clarify", "Hello!"],
  ["how are you?", "clarify", "Hello!"],
  ["what's this?", "clarify", "Hello!"],
  ["what data do you have?", "clarify", "Hello!"],
  ["thanks", "clarify", "welcome"],
  ["ok thanks", "clarify", "welcome"],
  ["great, thanks Jev", "clarify", "welcome"],
  ["thank you so much", "clarify", "welcome"],
  ["bye", "clarify", "Goodbye"],
  ["thanks, bye", "clarify", "Goodbye"],
  ["that's all", "clarify", "Goodbye"],
  ["ok", "clarify", "Ready when you are"],
  ["got it", "clarify", "Ready when you are"],
  ["hi all", "clarify", "Hello!"],
  ["good morning all", "clarify", "Hello!"],
  ["thanks all", "clarify", "welcome"],
  ["ta very much", "clarify", "welcome"],
  ["cheers a lot", "clarify", "welcome"],
  ["hello there jev, what can you do?", "clarify", "Hello!"],
  ["hi there jev thanks", "clarify", "welcome"],
  ["what's jev?", "clarify", "Hello!"],
  ["how can you help?", "clarify", "Hello!"],
  ["is anybody there?", "clarify", "Hello!"],
  ["what pollutants do you have?", "clarify", "Hello!"],
  ["no thanks", "clarify", "Goodbye"],
  ["take care", "clarify", "Goodbye"],
  ["thanks, that's great", "clarify", "welcome"],
  ["lovely, thanks", "clarify", "welcome"],
  ["no worries", "clarify", "Ready when you are"],
  ["Greetings team, annual CO2 for Australia in 2024", "answer"],
  ["Good day, annual CO2 for Australia in 2024", "answer"],
  ["tell me a joke", "reject", "Sorry"],
  ["write a haiku", "reject", "Sorry"],
  ["hourly ozone by street", "reject", "Sorry"],
  ["hi, hourly ozone by street", "reject", "Sorry"],
  ["annual CO2 for Australia in 2024", "answer"],
  ["pollution", "clarify"],
  ["emissions", "clarify"],
  ["data", "clarify"],
  ["everything", "clarify"],
];

// The Rust rules and this fixture must classify small talk with the same patterns and replies.
const rust = fs.readFileSync("src/request.rs", "utf8");
const fixture = fs.readFileSync("static/fixtures/catalogue.js", "utf8");
const grab = (text, re) => [...text.matchAll(re)].map(m => m.slice(1).join(" -> "));
for (const [what, count, rustRe, fixtureRe] of [
  ["openers and closers", 2, /const SMALL_TALK_(OPENERS|CLOSERS): &str = r"([^"]*)";/g, /const (OPENERS|CLOSERS) = "([^"]*)";/g],
  ["patterns", 5, /r"([^"]*)",\s*(GOODBYE|THANKS|GREETING|READY)_REPLY/g, /\["([^"]*)", (GOODBYE|THANKS|GREETING|READY)\]/g],
  ["replies", 4, /const (GOODBYE|THANKS|GREETING|READY)_REPLY: &str =\s*"((?:[^"\\]|\\.)*)";/g, /const (GOODBYE|THANKS|GREETING|READY) = "((?:[^"\\]|\\.)*)";/g],
]) {
  const fromRust = grab(rust, rustRe);
  assert.equal(fromRust.length, count, `small-talk ${what} not found in src/request.rs`);
  assert.deepEqual(grab(fixture, fixtureRe), fromRust, `small-talk ${what} differ between src/request.rs and the fixture`);
}

(async () => {
  for (const [request, outcome, marker] of cases) {
    const body = await (await window.SITE_FIXTURES.catalogue({ request, preference: "" })).json();
    assert.equal(body.outcome, outcome, `${request}: ${JSON.stringify(body)}`);
    if (marker) assert.match(body.message, new RegExp(marker), request);
  }
})().catch(error => { console.error(error); process.exit(1); });
