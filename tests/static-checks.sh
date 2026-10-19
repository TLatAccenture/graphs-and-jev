#!/usr/bin/env bash
set -uo pipefail
node tests/catalogue-fixture.test.js || exit 1
fail=0
check() { local name="$1"; shift; local out; out="$("$@" 2>/dev/null)"; if [[ -n "$out" ]]; then printf 'FAIL %s\n%s\n' "$name" "$out"; fail=1; fi; }
check "sub-path prefix"   grep -rn "/graphs-and-jev/" static
check "removed topics"    bash -c 'grep -rniE "dcceew|department of climate|parkinson|levodopa|clinical|RLCD" static README.md docs/design.md src | grep -vE "^src/lib.rs:[0-9]+: +\(\"/(dcceew|parkinsons)\", \"/"'
check "em dash"           grep -rn "—" static README.md docs/design.md docs/api-contract.md
check "sub-100 claim"     grep -rn "sub-100" static README.md
check "old brand"         grep -rn "Graphs + Jev" static README.md docs/design.md
check "fetch outside api" bash -c 'grep -rln "fetch(" static | grep -v "^static/js/api.js$"'
check "US spelling"       bash -c 'grep -rniE "\b(organiz|moderniz|optimiz|behavior|catalog\b)" static --include=*.html | grep -v "behavior:"'
exit "$fail"
