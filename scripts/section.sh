#!/usr/bin/env bash
# Every section of the green bar, each stated once (GaugeWright BUILD.md, stages 2 and 3).
#
#   scripts/section.sh <name>
#
# scripts/check.sh runs a section either directly, through this script, or as
# the Buck2 target of the same name — which also runs this script, in this
# checkout, and re-runs it only when a file the target declares has changed.
# The command lives here and nowhere else, so the two paths cannot state it
# differently. Stage 3 brought in the last three: the two cargo units, and the
# advisory audit, whose answer comes from a live database rather than from this
# tree. The audit is a `check_world` target, which refuses to run without a
# nonce naming the run, so it is in the graph without ever being answered from
# it.
set -euo pipefail
cd "$(dirname "$0")/.."

# A step whose tool is absent. Returns non-zero when the caller must skip, so a
# guarded step reads `if prerequisite …; then`; under `required` it never
# returns at all. `prerequisites` arrives from scripts/check.sh — from its
# argument on the direct path, and from the action's environment on the Buck2
# path, where it is part of the action's key so that a skip is never served to
# an invocation that required an answer. A direct invocation of this script
# carrying no word is a gate, so it defaults to required.
#
#   $1 the tool, $2 what it gates, $3 the command that installs it
prerequisite() {
  command -v "$1" >/dev/null 2>&1 && return 0
  if [ "${prerequisites:-required}" = required ]; then
    echo "$2 requires $1." >&2
    echo "install: $3" >&2
    exit 1
  fi
  echo "-- $2 SKIPPED: $1 is not installed --" >&2
  echo "   the ci.yml check job installs it and runs this on every pull request." >&2
  echo "   To close the gap locally: $3" >&2
  return 1
}

case "${1:-}" in
  advisories)
    if prerequisite cargo-audit "the dependency advisory audit" "cargo install cargo-audit"; then
      cargo audit
    fi ;;
  build-coverage)    node scripts/check-build-coverage.mjs ;;
  agent-guide)       node scripts/check-agent-guide.mjs ;;
  carries-agent-guide|carries-agent-guide-checker)
    # The cross-repository edge (GaugeWright DR-0124 stage 4). In a workspace
    # the bar builds the `carries` target and never reaches here; reaching here
    # means there is no `gaugewright` cell, so the question cannot be asked.
    echo "#unasserted: $1 needs a materialized workspace; the digest check answered instead"
    echo "-- $1 SKIPPED: no gaugewright cell outside a workspace --" >&2 ;;
  product-contracts) node scripts/check-product-contracts.mjs --enforce-evidence ;;
  formatting)        cargo fmt --all --check ;;
  lints)             cargo clippy --all-targets -- -D warnings ;;
  tests)             cargo test ;;
  # The theme check is inside the branch rather than after it because it reads
  # the built site: with no `site/` it fails for want of a build that did not
  # run, and with a stale one it answers about an older tree. Its other half —
  # whether this repository still carries the theme it was given — is asked
  # across every repository at once by `tools/docs-theme.mjs --check` in the
  # GaugeWright repository.
  documentation)
    if prerequisite mkdocs "the strict documentation build" "python3 -m pip install -r docs/requirements.txt"; then
      mkdocs build --strict
      node scripts/check-docs-theme.mjs
    fi ;;
  *) echo "section.sh: unknown section '${1:-}'" >&2; exit 2 ;;
esac
