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

echo "== agent guide =="
node scripts/check-agent-guide.mjs

echo "== product contracts =="
node scripts/check-product-contracts.mjs --enforce-evidence

echo "== formatting =="
cargo fmt --all --check

echo "== lints =="
cargo clippy --all-targets -- -D warnings

echo "== tests =="
cargo test

echo "== gaugewright-directory green bar PASSED =="
