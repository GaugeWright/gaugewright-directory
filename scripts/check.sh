#!/usr/bin/env bash
# The green bar for gaugewright-directory. This is the complete required check
# set: the configured CI gate runs this same script, so a passing run here and a
# passing gate cannot mean different things.
#
#   scripts/check.sh              the bar
#   scripts/check.sh required     the gate (what ci.yml runs)
#
# The word decides what an absent prerequisite means. A tool this host has not
# installed is not a tool it cannot supply, and GaugeWright's shared agent guide
# draws the line between them: the bar reports the gap and names the command
# that closes it, so a run that has answered everything else about the change
# still says so, while the invocation the gate runs refuses. A gate that fails
# with a message about the host when it has nothing to say about the change is
# how a reader learns to wave red through.
#
# What the gate covers is unchanged, because ci.yml installs every one of these
# in the steps above the invocation and then asks for `required` — so a dropped
# install step reddens the gate rather than quietly buying itself a skip.
#
# Bare is the bar; any word at all — `required`, or a mistyped one — is the
# gate, so a typo can never quietly buy a skip.
#
# Live checks against a deployed directory are deliberately not here; they need
# a reachable service and run from scripts/directory-check.sh.
set -euo pipefail
cd "$(dirname "$0")/.."

case "${1:-best-effort}" in
  best-effort) prerequisites=best-effort ;;
  *)           prerequisites=required ;;
esac

# A step whose tool is absent. Returns non-zero when the caller must skip, so a
# guarded step reads `if prerequisite …; then`; under `required` it never
# returns at all.
#
#   $1 the tool, $2 what it gates, $3 the command that installs it
prerequisite() {
  command -v "$1" >/dev/null 2>&1 && return 0
  if [ "$prerequisites" = required ]; then
    echo "$2 requires $1." >&2
    echo "install: $3" >&2
    exit 1
  fi
  echo "-- $2 SKIPPED: $1 is not installed --" >&2
  echo "   the ci.yml check job installs it and runs this on every pull request." >&2
  echo "   To close the gap locally: $3" >&2
  return 1
}

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
    prerequisites="$prerequisites" scripts/section.sh "$1"
  fi
}

echo "== production dependency advisories =="
# The audit lives here rather than in a workflow step so that the documented
# local green bar and the enforced gate stay the same command. It reads the
# committed Cargo.lock, so it needs neither the platform submodule nor a build.
if prerequisite cargo-audit "the dependency advisory audit" "cargo install cargo-audit"; then
    cargo audit
fi

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
