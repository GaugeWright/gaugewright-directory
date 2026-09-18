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

# The word travels with the section, and scripts/section.sh decides what an
# absent tool means. It is in the Buck2 action's key too, so a run that skipped
# a section is never served to a later run that required an answer.

# Stages 2 and 3 of the Buck2 migration (GaugeWright BUILD.md, DR-0124): every
# section of this bar is a Buck2 target declaring what it reads. When this checkout is a
# cell of a materialized workspace, a section runs through Buck2, which spares
# the re-run when nothing the section declares has changed and otherwise runs
# scripts/section.sh exactly as the direct path does. When it is not — a
# worktree, a CI runner, a host without buck2 — the same script runs directly.
# Same order, same command, same output, same verdict; Buck2 is under this bar,
# never beside it. Inside a Buck2 action already running a section, the direct
# path is taken so no nested client meets the daemon.
#
# A section whose answer comes from outside this tree is spared nothing: it is
# a `check_world` target, which refuses to run unless the invocation names the
# run, so it cannot be answered from a cache by accident.
via_buck2=""
if [ -z "${GREEN_BAR_INSIDE_BUCK2:-}" ] && command -v buck2 >/dev/null 2>&1 \
   && buck2 audit cell 2>/dev/null | grep -qx "gaugewright-directory: $(pwd -P)"; then
  via_buck2=1
fi
# One nonce for the whole run: the world-reading sections put it in their
# action's environment and will not run without it, while the cached sections
# do not read the key and are undisturbed by it.
GREEN_BAR_RUN="${GREEN_BAR_RUN:-$(date +%s)-$$}"
section() {
  if [ -n "$via_buck2" ]; then
    log="$(buck2 build "//:$1" -c "green_bar.run=$GREEN_BAR_RUN" \
      -c "green_bar.prerequisites=$prerequisites" --show-full-simple-output)"
    cat "$log"
  else
    prerequisites="$prerequisites" scripts/section.sh "$1"
  fi
}

echo "== production dependency advisories =="
# The audit lives here rather than in a workflow step so that the documented
# local green bar and the enforced gate stay the same command. It reads the
# committed Cargo.lock, so it needs neither the platform submodule nor a build.
section advisories

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
section lints

echo "== tests =="
section tests

# The docs build is the only thing that reads mkdocs.yml, the theme override
# under overrides/, and assets/brand.css. Nothing here read them until now: the
# site composes this build from gaugewright-site/docs-site/build.sh, so the
# first place a broken theme reference could appear was that build, in another
# repository, after the change had already landed here.
echo "== documentation =="
section documentation

echo "== gaugewright-directory green bar PASSED =="
