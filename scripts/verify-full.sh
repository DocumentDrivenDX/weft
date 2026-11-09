#!/usr/bin/env bash
# verify-full.sh - run the full ("heavy") suite locally and record evidence.
#
# Suite: weft-full
# Prereqs: bun, rust toolchain (rust-toolchain.toml), python3.12, jq
#
# Usage:
#   scripts/verify-full.sh              run the suite; on success write
#                                       .ci/evidence/full-suite.json
#   scripts/verify-full.sh --hash-only  print the content hash (used by the
#                                       `evidence-gate` CI job)
#
# Commit your change first, run this, then commit .ci/evidence/full-suite.json
# as the last commit. The hash covers every tracked file except .ci/evidence,
# so any later edit invalidates the evidence and requires a re-run.
# Release/tag/publish workflows ignore evidence and always run the heavy suite.
# Override the command with VERIFY_FULL_CMD if needed.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

content_hash() {
  if command -v sha256sum >/dev/null; then H=(sha256sum); else H=(shasum -a 256); fi
  git ls-files -z -- . ':!.ci/evidence' | xargs -0 "${H[@]}" | "${H[@]}" | cut -d' ' -f1
}

if [[ "${1:-}" == "--hash-only" ]]; then content_hash; exit 0; fi

command -v jq >/dev/null || { echo "jq is required" >&2; exit 2; }

SUITE='weft-full'
COMMAND="${VERIFY_FULL_CMD:-bun install --frozen-lockfile && bun run specs:check && bun run modules:check && cargo test --workspace --locked}"

if [[ -n "$(git status --porcelain --untracked-files=no -- . ':!.ci/evidence')" ]]; then
  echo "Tracked files have uncommitted changes; commit first so the hash matches the PR head." >&2
  exit 1
fi

before=$(content_hash)
echo ">> running full suite: $SUITE"
echo "+ $COMMAND"
bash -c "$COMMAND"
after=$(content_hash)
if [[ "$before" != "$after" ]]; then
  echo "The suite modified tracked files; restore or commit them and re-run." >&2
  exit 1
fi

tools=$(for t in bun node python3 cargo rustc uv; do
  if command -v "$t" >/dev/null; then echo "$t $("$t" --version 2>&1 | head -1)"; fi
done | jq -R -s -c 'split("\n") | map(select(length>0))')
mkdir -p .ci/evidence
jq -n \
  --arg suite "$SUITE" --arg hash "$after" --arg ts "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg cmd "$COMMAND" --argjson tools "$tools" \
  '{suite:$suite, tool_versions:$tools, content_hash:$hash, timestamp:$ts, command:$cmd}' \
  > .ci/evidence/full-suite.json
echo ">> wrote .ci/evidence/full-suite.json (commit it as the last commit)"
