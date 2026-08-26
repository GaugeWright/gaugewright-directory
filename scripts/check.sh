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
node scripts/check-build-coverage.mjs

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

# The docs build is the only thing that reads mkdocs.yml, the theme override
# under overrides/, and assets/brand.css. Nothing here read them until now: the
# site composes this build from gaugewright-site/docs-site/build.sh, so the
# first place a broken theme reference could appear was that build, in another
# repository, after the change had already landed here.
echo "== documentation =="
command -v mkdocs >/dev/null || {
    echo "mkdocs is not installed; run: python3 -m pip install -r docs/requirements.txt" >&2
    exit 1
}
# Output goes to the gitignored target/, the same site_dir mkdocs.yml names.
mkdocs build --strict
# Rendered from tools/docs-theme/repo-check.mjs in the GaugeWright repository,
# which owns the documentation theme (DR-0093). It verifies both that this
# repository still carries what was rendered into it and that the theme reached
# the built page: --strict fails on a missing custom_dir but resolves neither
# extra_css nor a template's own references, so a build that lost its brand
# stylesheet, mark, or faces exits zero.
node scripts/check-docs-theme.mjs

echo "== gaugewright-directory green bar PASSED =="
