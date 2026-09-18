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
  # `prerequisites` arrives from scripts/check.sh, which defaults it to
  # best-effort for the bar and to required for the gate. A direct invocation of
  # this script — the Buck2 target's path — carries no word and is a gate, so it
  # defaults to required here.
  #
  # The theme check is inside the branch rather than after it because it reads
  # the built site: with no `site/` it fails for want of a build that did not
  # run, and with a stale one it answers about an older tree. Its other half —
  # whether this repository still carries the theme it was given — is asked
  # across every repository at once by `tools/docs-theme.mjs --check` in the
  # GaugeWright repository.
  documentation)
    if command -v mkdocs >/dev/null 2>&1; then
      mkdocs build --strict
      node scripts/check-docs-theme.mjs
    elif [ "${prerequisites:-required}" = required ]; then
      echo "the strict documentation build requires mkdocs." >&2
      echo "install: python3 -m pip install -r docs/requirements.txt" >&2
      exit 1
    else
      echo "-- documentation SKIPPED: mkdocs is not installed --" >&2
      echo "   the ci.yml check job installs the docs toolchain and runs this, and the theme" >&2
      echo "   check that reads what it builds, on every pull request. To close the gap" >&2
      echo "   locally: python3 -m pip install -r docs/requirements.txt" >&2
    fi ;;
  *) echo "section.sh: unknown section '${1:-}'" >&2; exit 2 ;;
esac
