#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
# Preserve the original 0.1 gate, including its oracle, schema and browser run.
sh scripts/run-b002.sh
app_python=${WEFT_PYTHON:-python3}
"$app_python" tests/application/reports.py
"$app_python" tests/application/oracle.py
bun tests/application/schema-check.ts
WEFT_FRONTEND_CORPUS=tests/application/fixtures/cases.json \
WEFT_FRONTEND_REPORTS=target/b002a/reports.json \
WEFT_FRONTEND_BROWSER_SUMMARY=target/b002a/browser-summary.json \
node tests/frontend/browser-check.mjs
