/* In-browser fixture for the lineage-migration demo. Stub until WS4 builds the rules. */
"use strict";
(() => {
  const lineage = () => ({ok: true, status: 200, json: async () => ({outcome: "no_data", message: "Fixture not built yet."})});
  window.SITE_FIXTURES = Object.assign(window.SITE_FIXTURES || {}, {lineage});
})();
