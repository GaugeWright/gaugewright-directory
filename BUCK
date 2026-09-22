# This repository's Buck2 targets (GaugeWright BUILD.md, DR-0124).
#
# Stage 1 gave this cell one target, `:check`, the whole bar as a single opaque
# action over the whole checkout. It was scaffolding, and stage 3 retires it:
# with every section a target it would run the bar a second time inside
# `buck2 build //...`, and `//...` is now the bar.
#
# Stage 2: the bar's pure sections as their own targets, each declaring exactly
# what it reads, so a change that touches none of a section's inputs does not
# re-run it. Stage 3: the two cargo units, and the advisory audit, whose answer
# comes from a live database and is therefore never served from a cache.
#
# scripts/check.sh runs these in its usual order when this checkout is a cell of
# a materialized workspace, and runs the same commands directly when it is not.
# The command for each is stated once, in scripts/section.sh.
load("@gaugewright//tools/buck:check.bzl", "check_section", "check_unit", "check_world")

# The cargo workspace, twice: what clippy says about it and what its tests say.
# Both read the same unit — the manifest, the lockfile, the toolchain pin and
# the sources — and both are what `covers` claims to the build-coverage check.
# The platform submodule is excluded from the workspace, so it is not here.
CARGO_UNIT = ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "scripts/section.sh"] \
    + glob(["src/**/*.rs", "tests/**/*.rs"])

# Both are cargo, so both keep the cores busy and both take this repository's
# target directory lock while they run (the gaugewright cell's
# tools/buck/check.bzl). `cores` is what they hold of the machine; it is two at
# a time on a sixteen-core workstation and one on anything smaller.
check_unit(
    name = "lints",
    checkout = "gaugewright-directory",
    command = "scripts/section.sh lints",
    cores = 8,
    covers = ["rust-build:."],
    srcs = CARGO_UNIT,
)

check_unit(
    name = "tests",
    checkout = "gaugewright-directory",
    command = "scripts/section.sh tests",
    cores = 8,
    covers = ["rust-build:."],
    srcs = CARGO_UNIT,
)

# Reads the live RustSec database, so it is never answered from a cache.
check_world(
    name = "advisories",
    checkout = "gaugewright-directory",
    command = "scripts/section.sh advisories",
    covers = ["rust-audit:Cargo.lock"],
    srcs = ["Cargo.lock", "scripts/section.sh"],
)

check_section(
    name = "build-coverage",
    checkout = "gaugewright-directory",
    command = "scripts/section.sh build-coverage",
    # The coverage claim is read out of this BUCK file itself; the read sandbox
    # (GaugeWright FLEET.md stage 0) found it undeclared on 2026-09-22.
    srcs = ["BUCK", "Cargo.toml", "Cargo.lock", "scripts/check.sh", "scripts/section.sh", "scripts/check-build-coverage.mjs"]
        + glob([".github/workflows/*.yml", "scripts/build-coverage.policy.json"]),
)

check_section(
    name = "agent-guide",
    checkout = "gaugewright-directory",
    command = "scripts/section.sh agent-guide",
    # The checker also sweeps the tree for a second guide file anywhere, so a
    # new AGENTS.md or CLAUDE.md appearing in any directory is an input.
    srcs = ["AGENTS.md", "CLAUDE.md", "scripts/section.sh", "scripts/check-agent-guide.mjs"]
        + glob(["**/[Aa][Gg][Ee][Nn][Tt][Ss].[Mm][Dd]", "**/[Cc][Ll][Aa][Uu][Dd][Ee].[Mm][Dd]"], exclude = ["AGENTS.md", "CLAUDE.md"]),
)

check_section(
    name = "product-contracts",
    checkout = "gaugewright-directory",
    command = "scripts/section.sh product-contracts",
    # Every evidence id in the contract names a file, and the check asks that
    # each exists — the integration tests today. The read sandbox (GaugeWright
    # FLEET.md stage 0) found tests/directory_wiring.rs undeclared on 2026-09-22.
    srcs = ["contracts/product-routes.json", "src/lib.rs", "scripts/section.sh", "scripts/check-product-contracts.mjs"]
        + glob(["tests/**/*"]),
)

check_section(
    name = "formatting",
    checkout = "gaugewright-directory",
    command = "scripts/section.sh formatting",
    # `cargo fmt --all` covers this package alone: the platform submodule is
    # excluded from the workspace, so it is not an input here.
    srcs = ["Cargo.toml", "rust-toolchain.toml", "scripts/section.sh"] + glob(["src/**/*.rs", "tests/**/*.rs"]),
)

check_section(
    name = "documentation",
    checkout = "gaugewright-directory",
    command = "scripts/section.sh documentation",
    srcs = ["mkdocs.yml", "scripts/section.sh", "scripts/check-docs-theme.mjs"]
        + glob(["docs/**/*", "overrides/**/*"]),
)
