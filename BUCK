# This repository's Buck2 targets (GaugeWright BUILD.md, DR-0124).
#
# Stage 1: `:check`, the whole green bar as one opaque action.
# Stage 2: the bar's pure sections as their own targets, each declaring
# exactly what it reads, so a change that touches none of a section's inputs
# does not re-run it. scripts/check.sh runs these in its usual order when this
# checkout is a cell of a materialized workspace, and runs the same commands
# directly when it is not. The command for each is stated once, in
# scripts/section.sh. The advisory audit, clippy and the tests are not here:
# the first reads the world and is never cached; the other two are language
# units, which stage 3 owns.
load("@gaugewright//tools/buck:check.bzl", "check", "check_section")

check(
    name = "check",
    checkout = "gaugewright-directory",
    srcs = glob(["**/*"]),
)

check_section(
    name = "build-coverage",
    checkout = "gaugewright-directory",
    command = "scripts/section.sh build-coverage",
    srcs = ["Cargo.toml", "Cargo.lock", "scripts/check.sh", "scripts/section.sh", "scripts/check-build-coverage.mjs"]
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
    srcs = ["contracts/product-routes.json", "src/lib.rs", "scripts/section.sh", "scripts/check-product-contracts.mjs"],
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
