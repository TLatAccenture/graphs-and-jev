/* In-browser fixture for the Environmental Agency catalogue. It mirrors the Rust catalogue domain:
   the four page examples replay responses captured from the Rust service on 12 October 2026, with
   the same preference-fit step the service applies; other requests use simpler in-browser rules.
   Only backend_label, residency, request, preference and wall_ms differ from the service. */
"use strict";
(() => {
  const CATALOGUE = {"pollutants":[{"id":"CO2","label":"CO2","name":"carbon dioxide","unit":"Mt","groups":["ghg"]},{"id":"CO2e100","label":"CO2e (100-year)","name":"CO2-equivalent, 100-year global warming potential","unit":"Mt","groups":["ghg"]},{"id":"CO2e20","label":"CO2e (20-year)","name":"CO2-equivalent, 20-year global warming potential","unit":"Mt","groups":["ghg"]},{"id":"CH4","label":"CH4","name":"methane","unit":"kt","groups":["ghg"]},{"id":"N2O","label":"N2O","name":"nitrous oxide","unit":"kt","groups":["ghg"]},{"id":"PM2_5","label":"PM2.5","name":"fine particulate matter, particles smaller than 2.5 micrometres","unit":"ug/m3","groups":["pm"]},{"id":"PM10","label":"PM10","name":"particulate matter, particles smaller than 10 micrometres","unit":"ug/m3","groups":["pm"]},{"id":"NOx","label":"NOx","name":"nitrogen oxides","unit":"kt","groups":["air"]},{"id":"SO2","label":"SO2","name":"sulphur dioxide","unit":"kt","groups":["air"]},{"id":"SOx","label":"SOx","name":"sulphur oxides","unit":"kt","groups":["air"]},{"id":"NH3","label":"NH3","name":"ammonia","unit":"kt","groups":["air"]},{"id":"CO","label":"CO","name":"carbon monoxide","unit":"kt","groups":["air"]},{"id":"NMVOCs","label":"NMVOCs","name":"non-methane volatile organic compounds","unit":"kt","groups":["air"]},{"id":"BC","label":"BC","name":"black carbon","unit":"kt","groups":["air"]},{"id":"OC","label":"OC","name":"organic carbon","unit":"kt","groups":["air"]},{"id":"AS","label":"As","name":"arsenic","unit":"t","groups":["metal"]},{"id":"CD","label":"Cd","name":"cadmium","unit":"t","groups":["metal"]},{"id":"CR","label":"Cr","name":"chromium","unit":"t","groups":["metal"]},{"id":"CU","label":"Cu","name":"copper","unit":"t","groups":["metal"]},{"id":"HG","label":"Hg","name":"mercury","unit":"t","groups":["metal"]},{"id":"NI","label":"Ni","name":"nickel","unit":"t","groups":["metal"]},{"id":"PB","label":"Pb","name":"lead","unit":"t","groups":["metal"]},{"id":"SE","label":"Se","name":"selenium","unit":"t","groups":["metal"]},{"id":"ZN","label":"Zn","name":"zinc","unit":"t","groups":["metal"]}],"groups":{"pm":"Particulate matter","all":"All pollutants","ghg":"Greenhouse gases","metal":"Heavy metals"},"dimensions":{"SECTOR":["subsector","macrosector"],"TIME":["month","year"],"GEO":["region","country","continent"]},"defaults":{"SECTOR":"macrosector","GEO":"country","TIME":"year"},"summary":"Holds 24 air pollutants and greenhouse gases: CO2, CO2-equivalents, CH4, N2O, PM2.5, PM10, NOx, SO2, SOx, NH3, CO, NMVOCs, black carbon, organic carbon and nine heavy metals. Breakdowns: region, country or continent; month or year, 2015 to 2025; subsector or macrosector.","sources":[{"id":"S1","name":"Global Facility Emissions Inventory","description":"Facility-level greenhouse gas estimates aggregated to country, month and subsector.","publisher":"research","updated":2025,"levels":{"TIME":"month","GEO":"country","SECTOR":"subsector"},"pollutants":["CO2","CH4","N2O","CO2e100","CO2e20"],"coverage":{"GEO":["Italy","Nigeria",15],"TIME":["2021-01","2025-06",54],"SECTOR":["Coal mining","Landfill",12]}},{"id":"S2","name":"National Emissions Compendium","description":"Country-year totals compiled from national inventories, without a sector breakdown.","publisher":"official","updated":2024,"levels":{"GEO":"country","TIME":"year"},"pollutants":["CO2","CH4","N2O","NOx","SO2","CO","BC","OC","NH3","NMVOCs"],"coverage":{"GEO":["Italy","Nigeria",15],"TIME":["2015","2023",9]}},{"id":"S3","name":"European Air Emissions Reporting","description":"Air pollutant emissions reported by European countries, by year and reporting subsector.","publisher":"official","updated":2025,"levels":{"SECTOR":"subsector","GEO":"country","TIME":"year"},"pollutants":["PM2_5","PM10","NOx","SOx","NH3","NMVOCs","CO","AS","CD","CR","CU","HG","NI","PB","SE","ZN"],"coverage":{"SECTOR":["Road transport","Landfill",9],"GEO":["Italy","United Kingdom",7],"TIME":["2015","2024",10]}},{"id":"S4","name":"Regional Air Monitoring Network","description":"Monthly mean concentrations from regional monitoring networks in four European countries.","publisher":"official","updated":2025,"levels":{"GEO":"region","TIME":"month"},"pollutants":["PM2_5","PM10","NOx","SO2","CO"],"coverage":{"GEO":["Veneto","Andalusia",8],"TIME":["2019-01","2025-08",80]}},{"id":"S5","name":"Continental Climate Summary","description":"Greenhouse gas totals by continent, year and macrosector.","publisher":"research","updated":2024,"levels":{"GEO":"continent","SECTOR":"macrosector","TIME":"year"},"pollutants":["CO2","CH4","N2O","CO2e100"],"coverage":{"SECTOR":["Agriculture","Waste",8],"TIME":["2015","2024",10],"GEO":["Europe","Oceania",5]}},{"id":"S6","name":"Asia-Pacific Particulate Survey","description":"Particulate matter and black carbon estimates for four Asia-Pacific countries.","publisher":"research","updated":2023,"levels":{"SECTOR":"macrosector","TIME":"year","GEO":"country"},"pollutants":["PM2_5","PM10","BC"],"coverage":{"TIME":["2016","2022",7],"SECTOR":["Power","Agriculture",5],"GEO":["China","Australia",4]}},{"id":"S7","name":"Transport Emissions Tracker","description":"Monthly emissions from road, air and sea transport for 15 countries.","publisher":"commercial","updated":2025,"levels":{"SECTOR":"subsector","GEO":"country","TIME":"month"},"pollutants":["CO2","NOx","SO2","PM2_5"],"coverage":{"SECTOR":["Road transport","Shipping",3],"TIME":["2018-01","2025-06",90],"GEO":["Italy","Nigeria",15]}},{"id":"S8","name":"Agricultural Emissions Database","description":"Emissions from livestock and crop residue burning by country and year.","publisher":"official","updated":2025,"levels":{"TIME":"year","SECTOR":"subsector","GEO":"country"},"pollutants":["CH4","N2O","NH3"],"coverage":{"TIME":["2015","2025",11],"SECTOR":["Livestock","Crop residue burning",2],"GEO":["Italy","Nigeria",15]}}]};
  const CAPTURED = [
    {"request":"Annual PM2.5 by country, 2019 to 2024","preference":"","backend":"catalogue","backend_label":"Catalogue rules","model":"catalogue-rules-v1","residency":"local","stages":[{"stage":"gate","title":"Answer, clarify or reject","answer":{"type":"choice","probabilities":{"answer":1.0,"clarify":0.0,"reject":0.0},"top":"answer","confidence":1.0}},{"stage":"enrich","title":"Pollutants","threshold":0.5,"graph_candidates":["PM2_5"],"nodes":[{"node":"group:pm","label":"Particulate matter","kind":"group","p":0.0,"selected":false,"by":"explicit catalogue group"},{"node":"group:all","label":"All pollutants","kind":"group","p":0.0,"selected":false,"by":"explicit catalogue group"},{"node":"group:ghg","label":"Greenhouse gases","kind":"group","p":0.0,"selected":false,"by":"explicit catalogue group"},{"node":"group:metal","label":"Heavy metals","kind":"group","p":0.0,"selected":false,"by":"explicit catalogue group"},{"node":"pollutant:PM2_5","label":"PM2.5","kind":"pollutant","p":1.0,"selected":true,"by":"model"}],"groups_expanded":[],"pollutants":["PM2_5"]},{"stage":"query","title":"Breakdown","answers":{"GEO":{"type":"choice","probabilities":{"region":0.0,"country":1.0,"continent":0.0,"none":0.0},"top":"country","confidence":1.0},"TIME":{"type":"choice","probabilities":{"month":0.0,"year":1.0,"none":0.0},"top":"year","confidence":1.0},"SECTOR":{"type":"choice","probabilities":{"subsector":0.0,"macrosector":0.0,"none":1.0},"top":"none","confidence":1.0}},"query":{"pollutants":["PM2_5"],"levels":{"GEO":"country","TIME":"year"},"filters":{"TIME":{"level":"year","members":["2019","2020","2021","2022","2023","2024"]}}}},{"stage":"discover","title":"Discovery","solutions":3},{"stage":"rank","title":"Ranking","by":"coverage (no preference given)","order":["C","A","B"]},{"stage":"explain","title":"Explanation","by":"template"}],"policy":{"min_confidence":0.8,"validated_risk_guarantee":false,"calibration_loaded":false,"time_anchor":null},"query":{"pollutants":["PM2_5"],"levels":{"GEO":"country","TIME":"year"},"filters":{"TIME":{"level":"year","members":["2019","2020","2021","2022","2023","2024"]}}},"solutions":[{"sources":[{"id":"S7","name":"Transport Emissions Tracker","publisher":"commercial","updated":2025,"provides":["PM2_5"]}],"profiles":{"GEO.country":{"Italy":25,"France":35,"Spain":55,"Portugal":18,"Poland":24,"United Kingdom":51,"United States":26,"Brazil":43,"China":52,"India":59,"Japan":35,"Australia":35,"South Africa":50,"Nigeria":16},"TIME.year":{"2019":303,"2020":274,"2021":273,"2022":212,"2023":301,"2024":294}},"cells":84,"notes":["Transport Emissions Tracker is summed over sector but covers only 3 of 13 subsectors"],"id":"C","summary":"Sources: Transport Emissions Tracker (commercial, updated 2025). 14 countries: Italy, France, Spain, Portugal, Poland, United Kingdom, United States, Brazil and 6 more. Years: 2019 to 2024 (6 covered). Pollutants: PM2.5. Transport Emissions Tracker is summed over sector but covers only 3 of 13 subsectors."},{"sources":[{"id":"S3","name":"European Air Emissions Reporting","publisher":"official","updated":2025,"provides":["PM2_5"]}],"profiles":{"GEO.country":{"Italy":57,"France":24,"Germany":58,"Spain":39,"Portugal":51,"Poland":26,"United Kingdom":20},"TIME.year":{"2019":36,"2020":23,"2021":20,"2022":28,"2023":26,"2024":38}},"cells":42,"notes":["European Air Emissions Reporting is summed over sector but covers only 9 of 13 subsectors"],"id":"A","summary":"Sources: European Air Emissions Reporting (official, updated 2025). 7 countries: Italy, France, Germany, Spain, Portugal, Poland, United Kingdom. Years: 2019 to 2024 (6 covered). Pollutants: PM2.5. European Air Emissions Reporting is summed over sector but covers only 9 of 13 subsectors."},{"sources":[{"id":"S4","name":"Regional Air Monitoring Network","publisher":"official","updated":2025,"provides":["PM2_5"]}],"profiles":{"GEO.country":{"Italy":170,"France":61,"Germany":116,"Spain":54},"TIME.year":{"2019":288,"2020":295,"2021":239,"2022":284,"2023":253,"2024":343}},"cells":24,"notes":[],"id":"B","summary":"Sources: Regional Air Monitoring Network (official, updated 2025). 4 countries: Italy, France, Germany, Spain. Years: 2019 to 2024 (6 covered). Pollutants: PM2.5."}],"subgraph":{"nodes":[{"id":"P:PM2_5","kind":"pollutant","label":"PM2.5"},{"id":"L:GEO.region","kind":"level","label":"region","dim":"GEO","selected":false},{"id":"L:GEO.country","kind":"level","label":"country","dim":"GEO","selected":true},{"id":"L:GEO.continent","kind":"level","label":"continent","dim":"GEO","selected":false},{"id":"L:TIME.month","kind":"level","label":"month","dim":"TIME","selected":false},{"id":"L:TIME.year","kind":"level","label":"year","dim":"TIME","selected":true},{"id":"L:SECTOR.subsector","kind":"level","label":"subsector","dim":"SECTOR","selected":false},{"id":"L:SECTOR.macrosector","kind":"level","label":"macrosector","dim":"SECTOR","selected":false},{"id":"S:S3","kind":"source","label":"European Air Emissions Reporting"},{"id":"S:S4","kind":"source","label":"Regional Air Monitoring Network"},{"id":"S:S7","kind":"source","label":"Transport Emissions Tracker"}],"edges":[{"from":"L:GEO.region","to":"L:GEO.country","kind":"rolls_up"},{"from":"L:GEO.country","to":"L:GEO.continent","kind":"rolls_up"},{"from":"L:TIME.month","to":"L:TIME.year","kind":"rolls_up"},{"from":"L:SECTOR.subsector","to":"L:SECTOR.macrosector","kind":"rolls_up"},{"from":"S:S3","to":"P:PM2_5","kind":"provides"},{"from":"S:S3","to":"L:SECTOR.subsector","kind":"at_level"},{"from":"S:S3","to":"L:GEO.country","kind":"at_level"},{"from":"S:S3","to":"L:TIME.year","kind":"at_level"},{"from":"S:S4","to":"P:PM2_5","kind":"provides"},{"from":"S:S4","to":"L:GEO.region","kind":"at_level"},{"from":"S:S4","to":"L:TIME.month","kind":"at_level"},{"from":"S:S7","to":"P:PM2_5","kind":"provides"},{"from":"S:S7","to":"L:SECTOR.subsector","kind":"at_level"},{"from":"S:S7","to":"L:GEO.country","kind":"at_level"},{"from":"S:S7","to":"L:TIME.month","kind":"at_level"}]},"explanation":{"by":"template","text":"3 solutions hold PM2.5 by country and year.\nFirst: C, Transport Emissions Tracker, covering 14 countries, 2019 to 2024. Caution: Transport Emissions Tracker is summed over sector but covers only 3 of 13 subsectors.\nThen: A, European Air Emissions Reporting, covering 7 countries, 2019 to 2024. Caution: European Air Emissions Reporting is summed over sector but covers only 9 of 13 subsectors.\nThen: B, Regional Air Monitoring Network, covering 4 countries, 2019 to 2024."},"outcome":"answer","message":null,"totals":{"decisions":9,"calls":3,"latency_ms":0.0,"cost_usd":0.0,"wall_ms":0.0}},
    {"request":"Particulate matter by region","preference":"","backend":"catalogue","backend_label":"Catalogue rules","model":"catalogue-rules-v1","residency":"local","stages":[{"stage":"gate","title":"Answer, clarify or reject","answer":{"type":"choice","probabilities":{"answer":1.0,"clarify":0.0,"reject":0.0},"top":"answer","confidence":1.0}},{"stage":"enrich","title":"Pollutants","threshold":0.5,"graph_candidates":["PM2_5","PM10"],"nodes":[{"node":"group:pm","label":"Particulate matter","kind":"group","p":1.0,"selected":true,"by":"explicit catalogue group"},{"node":"group:all","label":"All pollutants","kind":"group","p":0.0,"selected":false,"by":"explicit catalogue group"},{"node":"group:ghg","label":"Greenhouse gases","kind":"group","p":0.0,"selected":false,"by":"explicit catalogue group"},{"node":"group:metal","label":"Heavy metals","kind":"group","p":0.0,"selected":false,"by":"explicit catalogue group"},{"node":"pollutant:PM2_5","label":"PM2.5","kind":"pollutant","p":1.0,"selected":true,"by":"model"},{"node":"pollutant:PM10","label":"PM10","kind":"pollutant","p":1.0,"selected":true,"by":"model"}],"groups_expanded":["pm"],"pollutants":["PM2_5","PM10"]},{"stage":"query","title":"Breakdown","answers":{"GEO":{"type":"choice","probabilities":{"region":1.0,"country":0.0,"continent":0.0,"none":0.0},"top":"region","confidence":1.0},"TIME":{"type":"choice","probabilities":{"month":0.0,"year":0.0,"none":1.0},"top":"none","confidence":1.0},"SECTOR":{"type":"choice","probabilities":{"subsector":0.0,"macrosector":0.0,"none":1.0},"top":"none","confidence":1.0}},"query":{"pollutants":["PM2_5","PM10"],"levels":{"GEO":"region"},"filters":{}}},{"stage":"discover","title":"Discovery","solutions":1},{"stage":"rank","title":"Ranking","by":"coverage (no preference given)","order":["A"]},{"stage":"explain","title":"Explanation","by":"template"}],"policy":{"min_confidence":0.8,"validated_risk_guarantee":false,"calibration_loaded":false,"time_anchor":null},"query":{"pollutants":["PM2_5","PM10"],"levels":{"GEO":"region"},"filters":{}},"solutions":[{"sources":[{"id":"S4","name":"Regional Air Monitoring Network","publisher":"official","updated":2025,"provides":["PM10","PM2_5"]}],"profiles":{"GEO.region":{"Veneto":54,"Lazio":56,"Lombardy":60,"Ile-de-France":35,"Auvergne-Rhone-Alpes":26,"Brandenburg":58,"Bavaria":58,"Andalusia":54}},"cells":8,"notes":["Regional Air Monitoring Network is summed over time but covers only 80 of 132 months"],"id":"A","summary":"Sources: Regional Air Monitoring Network (official, updated 2025). 8 regions: Veneto, Lazio, Lombardy, Ile-de-France, Auvergne-Rhone-Alpes, Brandenburg, Bavaria, Andalusia. Pollutants: PM2.5, PM10. Regional Air Monitoring Network is summed over time but covers only 80 of 132 months."}],"subgraph":{"nodes":[{"id":"P:PM2_5","kind":"pollutant","label":"PM2.5"},{"id":"P:PM10","kind":"pollutant","label":"PM10"},{"id":"L:GEO.region","kind":"level","label":"region","dim":"GEO","selected":true},{"id":"L:GEO.country","kind":"level","label":"country","dim":"GEO","selected":false},{"id":"L:GEO.continent","kind":"level","label":"continent","dim":"GEO","selected":false},{"id":"L:TIME.month","kind":"level","label":"month","dim":"TIME","selected":false},{"id":"L:TIME.year","kind":"level","label":"year","dim":"TIME","selected":false},{"id":"L:SECTOR.subsector","kind":"level","label":"subsector","dim":"SECTOR","selected":false},{"id":"L:SECTOR.macrosector","kind":"level","label":"macrosector","dim":"SECTOR","selected":false},{"id":"S:S4","kind":"source","label":"Regional Air Monitoring Network"}],"edges":[{"from":"L:GEO.region","to":"L:GEO.country","kind":"rolls_up"},{"from":"L:GEO.country","to":"L:GEO.continent","kind":"rolls_up"},{"from":"L:TIME.month","to":"L:TIME.year","kind":"rolls_up"},{"from":"L:SECTOR.subsector","to":"L:SECTOR.macrosector","kind":"rolls_up"},{"from":"S:S4","to":"P:PM2_5","kind":"provides"},{"from":"S:S4","to":"P:PM10","kind":"provides"},{"from":"S:S4","to":"L:GEO.region","kind":"at_level"},{"from":"S:S4","to":"L:TIME.month","kind":"at_level"}]},"explanation":{"by":"template","text":"1 solution hold PM2.5, PM10 by region.\nFirst: A, Regional Air Monitoring Network, covering 8 regions. Caution: Regional Air Monitoring Network is summed over time but covers only 80 of 132 months."},"outcome":"answer","message":null,"totals":{"decisions":10,"calls":3,"latency_ms":0.0,"cost_usd":0.0,"wall_ms":0.0}},
    {"request":"Emissions","preference":"","backend":"catalogue","backend_label":"Catalogue rules","model":"catalogue-rules-v1","residency":"local","stages":[{"stage":"gate","title":"Answer, clarify or reject","answer":{"type":"choice","probabilities":{"answer":0.0,"clarify":1.0,"reject":0.0},"top":"clarify","confidence":1.0}}],"policy":{"min_confidence":0.8,"validated_risk_guarantee":false,"calibration_loaded":false,"time_anchor":null},"outcome":"clarify","message":"The request needs more detail: name a pollutant (or ask for emissions in general) and a breakdown such as country and year.","totals":{"decisions":1,"calls":1,"latency_ms":0.0,"cost_usd":0.0,"wall_ms":0.0}},
    {"request":"Hourly ozone by street in 1990","preference":"","backend":"catalogue","backend_label":"Catalogue rules","model":"catalogue-rules-v1","residency":"local","stages":[{"stage":"gate","title":"Answer, clarify or reject","answer":{"type":"choice","probabilities":{"answer":0.0,"clarify":0.0,"reject":1.0},"top":"reject","confidence":1.0}}],"policy":{"min_confidence":0.8,"validated_risk_guarantee":false,"calibration_loaded":false,"time_anchor":null},"outcome":"reject","message":"Sorry, I can't help with that one. This catalogue covers air pollutants and emissions for 2015-2025 at region, country or continent level, so hourly or city-level data, other years and other topics are out of scope. Try \"Annual PM2.5 by country, 2019 to 2024\".","totals":{"decisions":1,"calls":1,"latency_ms":0.0,"cost_usd":0.0,"wall_ms":0.0}}
  ];

  const BROWSER = {backend_label: "In-browser catalogue rules", residency: "browser"};
  const MIN_CONFIDENCE = 0.8;
  const FIRST_YEAR = 2015, LAST_YEAR = 2025;
  const LEGEND = {"0": "none", "1": "part", "2": "most", "3": "full"};
  const LEVELS = {GEO: ["region", "country", "continent"], TIME: ["month", "year"], SECTOR: ["subsector", "macrosector"]};
  const CLARIFY = "The request needs more detail: name a pollutant (or ask for emissions in general) and a breakdown such as country and year.";
  const REJECT = "Sorry, I can't help with that one. This catalogue covers air pollutants and emissions for 2015-2025 at region, country or continent level, so hourly or city-level data, other years and other topics are out of scope. Try \"Annual PM2.5 by country, 2019 to 2024\".";
  const GREETING = "Hello! I can help you find air-pollution and emissions datasets. Name a pollutant (or ask for emissions in general) and a breakdown such as country and year, for example \"Annual PM2.5 by country, 2019 to 2024\".";
  const THANKS = "You're welcome! Ask about another pollutant or breakdown whenever you like.";
  const GOODBYE = "Goodbye! Come back any time you need air-pollution or emissions data.";
  const READY = "Ready when you are. Name a pollutant (or ask for emissions in general) and a breakdown such as country and year, for example \"Annual PM2.5 by country, 2019 to 2024\".";
  const UNCERTAIN = "Preference scores are uncertain. Candidate datasets are returned for review, not approved automatically.";
  const ALLOWED = new Set(("official sources source only the most recent latest newest data available updated prefer please " +
    "in for across from and with by per all each every last past one two three four five to through until between " +
    "countries regions continents").split(" "));

  const response = body => ({ok: true, status: 200, json: async () => body});
  const clone = value => JSON.parse(JSON.stringify(value));
  const key = text => text.toLowerCase().replace(/\s+/g, " ").trim().replace(/\.$/, "");
  const escape = text => text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const word = (text, term, flags) => new RegExp("(?<![a-z0-9])" + escape(term) + "(?![a-z0-9])", flags).test(text);
  const choice = (labels, top) => ({type: "choice", probabilities: Object.fromEntries(labels.map(l => [l, l === top ? 1.0 : 0.0])), top, confidence: 1.0});
  const captured = Object.fromEntries(CAPTURED.map(body => [key(body.request), body]));
  const greeting = /^(?:(?:hi|hello|hey|hiya|howdy|yo|greetings)|good (?:morning|afternoon|evening|day))(?:\s+(?:there|jev|all|everyone|team))?(?:[\s,!.:;-]+|$)/i;
  const stripGreeting = text => text.replace(greeting, "").trim();
  // Mirrors SMALL_TALK in src/request.rs: each pattern must cover the whole request.
  const OPENERS = "(?:(?:hi|hello|hey|hiya|howdy|yo|greetings|good (?:morning|afternoon|evening|day)|ok|okay|oh|so|well|great|cool|nice|perfect|awesome|alright|right|sure)(?: (?:there|jev|all|everyone|team))? )*";
  const CLOSERS = "(?: (?:jev|please|then|again|mate|there|everyone|team))*";
  const SMALL_TALK = [
    ["(?:(?:thanks|thank you|cheers) )?(?:bye|bye bye|goodbye|good bye|see you|see ya|cya|see you later|thats all|that is all|thats it|im done|i am done|all done)(?: (?:thanks|thank you|cheers))?", GOODBYE],
    ["thanks|thank you|thankyou|thx|ty|cheers|ta|many thanks|much appreciated|appreciate it|(?:thanks|thank you) (?:very much|so much|a lot|heaps|again|for that|for your help)", THANKS],
    ["hi|hello|hey|hiya|howdy|yo|greetings|good (?:morning|afternoon|evening|day)|morning|afternoon|evening|test|testing|anyone there|is anyone there|are you there", GREETING],
    ["help|help me|help please|i need help|can you help|can you help me|could you help me|what can you do|what do you do|what are you|who are you|what is this|whats this|what is jev|who is jev|how does this work|how does it work|how do i use this|how do i use it|how do i start|where do i start|what can i ask|what should i ask|what do i type|what data do you have|what data is there|what data is available|what do you have|what is available|whats available|how are you|how are you doing|hows it going|how is it going", GREETING],
    ["ok|okay|cool|great|nice|perfect|awesome|alright|all right|sure|got it|sounds good|right|yes|yep|yeah", READY],
  ].map(([talk, reply]) => [new RegExp("^" + OPENERS + "(?:" + talk + ")" + CLOSERS + "$"), reply]);
  const smallTalk = text => {
    const words = text.normalize("NFKD").replace(/\p{M}/gu, "").toLowerCase().replace(/['\u2019]/g, "").replace(/[^a-z0-9]+/g, " ").trim();
    const hit = SMALL_TALK.find(([pattern]) => pattern.test(words));
    return hit ? hit[1] : null;
  };

  // ponytail: mirrors preference_score in src/request.rs without its catalogue-term masking or member
  // filters, so a preference naming places or years scores as unrecognised (review) here.
  function preferenceScore(preference, solution) {
    const text = preference.toLowerCase();
    if (text.includes("as many") || text.includes("coverage")) return null;
    if ((text.match(/[a-z]+/g) || []).some(w => !ALLOWED.has(w))) return null;
    const scores = [];
    if (word(text, "official")) {
      const official = solution.sources.map(s => s.publisher === "official");
      scores.push(official.every(Boolean) ? 3 : official.some(Boolean) ? 1 : 0);
    }
    if (/\b(recent|latest|newest)\b/.test(text)) {
      scores.push(Math.max(0, 3 - (LAST_YEAR - Math.min(...solution.sources.map(s => s.updated)))));
    }
    return scores.length ? Math.min(...scores) : null;
  }

  function fit(score) {
    if (score == null) return {type: "score", probabilities: {"0": 0.25, "1": 0.25, "2": 0.25, "3": 0.25}, top: "3", confidence: 0.25, score: 1.5, legend: LEGEND};
    return {type: "score", probabilities: Object.fromEntries(Object.keys(LEGEND).map(k => [k, Number(k) === score ? 1.0 : 0.0])), top: String(score), confidence: 1.0, score, legend: LEGEND};
  }

  // The Rust rank stage: score each solution, sort by fit then cells, review if any score is uncertain.
  function rank(out, preference) {
    if (!preference) return out;
    const solutions = out.solutions.slice().sort((a, b) => a.id.localeCompare(b.id));
    solutions.forEach(s => { s.fit = fit(preferenceScore(preference, s)); });
    solutions.sort((a, b) => b.fit.score - a.fit.score || (b.cells || 0) - (a.cells || 0));
    out.solutions = solutions;
    out.totals.decisions += solutions.length;
    out.totals.calls += solutions.length;
    Object.assign(out.stages.find(s => s.stage === "rank"), {by: "preference fit", order: solutions.map(s => s.id)});
    if (solutions.some(s => s.fit.confidence < MIN_CONFIDENCE)) {
      out.stages = out.stages.filter(s => s.stage !== "explain");
      delete out.explanation;
      return Object.assign(out, {outcome: "review", message: UNCERTAIN});
    }
    const [head, ...lines] = out.explanation.text.split("\n");
    const byId = Object.fromEntries(lines.map(l => [l.split(", ")[0].split(": ")[1], l.slice(l.indexOf(": ") + 2)]));
    out.explanation.text = [head, ...solutions.map((s, i) => {
      const [base, caution] = byId[s.id].split(" Caution: ");
      return (i ? "Then: " : "First: ") + base + " Preference fit " + s.fit.score.toFixed(1) + " of 3 (" +
        Math.round(s.fit.confidence * 100) + "% confident in its most likely level)." + (caution ? " Caution: " + caution : "");
    })].join("\n");
    return out;
  }

  function stop(out, outcome, message) {
    out.stages = [{stage: "gate", title: "Answer, clarify or reject", answer: choice(["answer", "clarify", "reject"], outcome)}];
    out.totals = {decisions: 1, calls: 1, latency_ms: 0.0, cost_usd: 0.0};
    return Object.assign(out, {outcome, message});
  }

  // ponytail: single-source matching by pollutant and level only. The Rust service also combines
  // sources, checks requested members and counts covered cells; the note on each result says so.
  function rules(request, preference) {
    const out = {request, preference, backend: "catalogue", ...BROWSER, model: "catalogue-fixture-v1", stages: [],
      policy: {min_confidence: MIN_CONFIDENCE, validated_risk_guarantee: false, calibration_loaded: false, time_anchor: null}};
    const chat = smallTalk(request);
    if (chat) return stop(out, "clarify", chat);
    const stripped = stripGreeting(request);
    const text = stripped.toLowerCase().replace(/pm\s*2[.,]?\s*5/g, "pm2.5").replace(/sulfur/g, "sulphur");
    const years = (stripped.match(/\b(19|20)\d{2}\b/g) || []).map(Number);
    if (/\b(hourly|daily|weekly|street|streets|city|cities|postcode|ozone|o3|stations?|days?|hours?|weeks?|forecasts?|water|noise|radon|poem|world cup)\b/.test(text) ||
        years.some(y => y < FIRST_YEAR || y > LAST_YEAR)) return stop(out, "reject", REJECT);

    const groups = Object.entries({pm: /particulate/, ghg: /greenhouse|\bghg/, metal: /heavy metal|\bmetals\b/, all: /all pollutants/})
      .filter(([, re]) => re.test(text)).map(([g]) => g);
    const named = CATALOGUE.pollutants.filter(p => word(text, p.name) ||
      (p.label.length <= 2 ? word(stripped, p.label) : word(text, p.label.toLowerCase())));
    const pollutants = CATALOGUE.pollutants
      .filter(p => named.includes(p) || groups.some(g => g === "all" || p.groups.includes(g))).map(p => p.id);
    const levels = {};
    if (/\bregion(s|al)?\b/.test(text)) levels.GEO = "region";
    else if (/\bcountr(y|ies)\b|\bnational\b/.test(text)) levels.GEO = "country";
    else if (/\bcontinent(s|al)?\b/.test(text)) levels.GEO = "continent";
    if (/\bmonth(s|ly)?\b/.test(text)) levels.TIME = "month";
    else if (/\b(year|years|yearly|annual)\b/.test(text)) levels.TIME = "year";
    if (/\bsubsector/.test(text)) levels.SECTOR = "subsector";
    else if (/\b(macro)?sector/.test(text)) levels.SECTOR = "macrosector";
    const broadDomain = /\b(pollution|emissions?|data|everything)\b/.test(text);
    if (!pollutants.length && !Object.keys(levels).length && !broadDomain) return stop(out, "reject", REJECT);
    if (!pollutants.length || !Object.keys(levels).length) return stop(out, "clarify", CLARIFY);

    const query = {pollutants, levels, filters: {}};
    out.stages = [
      {stage: "gate", title: "Answer, clarify or reject", answer: choice(["answer", "clarify", "reject"], "answer")},
      {stage: "enrich", title: "Pollutants", groups_expanded: groups, pollutants},
      {stage: "query", title: "Breakdown", query}
    ];
    out.query = query;
    out.totals = {decisions: 1 + Object.keys(CATALOGUE.groups).length + named.length + 3, calls: 3, latency_ms: 0.0, cost_usd: 0.0};
    const fits = s => Object.entries(levels).every(([dim, lvl]) => s.levels[dim] && LEVELS[dim].indexOf(s.levels[dim]) <= LEVELS[dim].indexOf(lvl));
    const sources = CATALOGUE.sources.filter(s => pollutants.every(p => s.pollutants.includes(p)) && fits(s));
    out.stages.push({stage: "discover", title: "Discovery", solutions: sources.length});
    if (!sources.length) return Object.assign(out, {outcome: "no_data", message: "No single source holds these pollutants at these levels."});

    out.solutions = sources.map((s, i) => ({
      id: String.fromCharCode(65 + i),
      sources: [{id: s.id, name: s.name, publisher: s.publisher, updated: s.updated, provides: pollutants}],
      notes: ["In-browser rules match pollutant and breakdown only; the Rust service also checks members, combines sources and counts covered cells"]
    }));
    const labels = CATALOGUE.pollutants.filter(p => pollutants.includes(p.id)).map(p => p.label).join(", ");
    const by = Object.values(levels).join(" and ");
    out.explanation = {by: "template", text: [out.solutions.length + " source" + (out.solutions.length === 1 ? "" : "s") + " hold " + labels + " by " + by + ".",
      ...out.solutions.map((s, i) => (i ? "Then: " : "First: ") + s.id + ", " + s.sources[0].name + ".")].join("\n")};
    out.stages.push({stage: "rank", title: "Ranking", by: "coverage (no preference given)", order: out.solutions.map(s => s.id)},
      {stage: "explain", title: "Explanation", by: "template"});
    out.outcome = "answer";
    out.message = null;
    return rank(out, preference);
  }

  function catalogue({request, preference}) {
    const started = performance.now();
    const text = (request || "").trim(), pref = (preference || "").trim();
    const hit = captured[key(text)];
    const out = hit ? Object.assign(clone(hit), {request: text, preference: pref}, BROWSER) : rules(text, pref);
    if (hit && out.outcome === "answer") rank(out, pref);
    out.totals.wall_ms = Math.round((performance.now() - started) * 10) / 10;
    return response(out);
  }
  catalogue.list = () => response(clone(CATALOGUE));

  window.SITE_FIXTURES = Object.assign(window.SITE_FIXTURES || {}, {catalogue});
})();
