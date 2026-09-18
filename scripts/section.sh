#!/usr/bin/env bash
# The green bar's pure sections, each stated once (GaugeWright BUILD.md, stage 2).
#
#   scripts/section.sh <name>
#
# scripts/check.sh runs a section either directly, through this script, or as
# the Buck2 target of the same name — which also runs this script, in this
# checkout, and re-runs it only when a file the target declares has changed.
# The command lives here and nowhere else, so the two paths cannot state it
# differently. A section that reads the world — the advisory audit — is not
# here: it is always run, and it is never a target.
set -euo pipefail
cd "$(dirname "$0")/.."
case "${1:-}" in
  build-coverage)    node scripts/check-build-coverage.mjs ;;
  agent-guide)       node scripts/check-agent-guide.mjs ;;
  product-contracts) node scripts/check-product-contracts.mjs --enforce-evidence ;;
  formatting)        cargo fmt --all --check ;;
  documentation)
    command -v mkdocs >/dev/null || {
      echo "mkdocs is not installed; run: python3 -m pip install -r docs/requirements.txt" >&2
      exit 1
    }
    mkdocs build --strict
    node scripts/check-docs-theme.mjs ;;
  *) echo "section.sh: unknown section '${1:-}'" >&2; exit 2 ;;
esac
