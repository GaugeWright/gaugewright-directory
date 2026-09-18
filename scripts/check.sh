#!/usr/bin/env bash
# The green bar for gaugewright-directory. This is the complete required check
# set: the configured CI gate runs this same script, so a passing run here and a
# passing gate cannot mean different things.
#
#   scripts/check.sh
#
# Live checks against a deployed directory are deliberately not here; they need
# a reachable service and run from scripts/directory-check.sh.
set -euo pipefail
cd "$(dirname "$0")/.."

# Stage 2 of the Buck2 migration (GaugeWright BUILD.md, DR-0124): each pure
# section is a Buck2 target declaring what it reads. When this checkout is a
# cell of a materialized workspace, a section runs through Buck2, which spares
# the re-run when nothing the section declares has changed and otherwise runs
# scripts/section.sh exactly as the direct path does. When it is not — a
# worktree, a CI runner, a host without buck2 — the same script runs directly.
# Same order, same command, same output, same verdict; Buck2 is under this bar,
# never beside it. Inside a Buck2 action already running the whole bar, the
# direct path is taken so no nested client meets the daemon.
via_buck2=""
if [ -z "${GREEN_BAR_INSIDE_BUCK2:-}" ] && command -v buck2 >/dev/null 2>&1 \
   && buck2 audit cell 2>/dev/null | grep -qx "gaugewright-directory: $(pwd -P)"; then
  via_buck2=1
fi
section() {
  if [ -n "$via_buck2" ]; then
    log="$(buck2 build "//:$1" --show-full-simple-output)"
    cat "$log"
  else
    scripts/section.sh "$1"
  fi
}

echo "== production dependency advisories =="
# The audit lives here rather than in a workflow step so that the documented
# local green bar and the enforced gate stay the same command. It reads the
# committed Cargo.lock, so it needs neither the platform submodule nor a build.
command -v cargo-audit >/dev/null || {
    echo "cargo-audit is not installed; run: cargo install cargo-audit" >&2
    exit 1
}
cargo audit

# Rendered from tools/shared-checks/build-coverage.mjs in the GaugeWright
# repository, which owns it. It fails when a cargo workspace or a lockfile is
# watched by nothing. Edit it there and re-render; a local edit fails here.
echo "== build coverage =="
section build-coverage

echo "== agent guide =="
section agent-guide

echo "== product contracts =="
section product-contracts

echo "== formatting =="
section formatting

echo "== lints =="
cargo clippy --all-targets -- -D warnings

echo "== tests =="
cargo test

# The docs build is the only thing that reads mkdocs.yml, the theme override
# under overrides/, and assets/brand.css. Nothing here read them until now: the
# site composes this build from gaugewright-site/docs-site/build.sh, so the
# first place a broken theme reference could appear was that build, in another
# repository, after the change had already landed here.
echo "== documentation =="
section documentation

echo "== gaugewright-directory green bar PASSED =="
