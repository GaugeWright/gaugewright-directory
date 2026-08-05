# gaugewright-directory — Agent Guide

This repository is the GaugeWright blind account directory: the readable
routing layer plus opaque sealed account blobs. Its trust story is that it is
blind by construction, so a change that lets this service read, derive, or
retain account content is a defect regardless of what it enables.

## Build

The platform crates are consumed through the `platform/` submodule, pinned to a
known-good commit of the public GaugeDesk mirror:

```sh
git submodule update --init
cargo build
```

## Check

One command runs the complete required check set, and the CI gate runs the same
script:

```sh
scripts/check.sh
```

It covers the product-contract manifest with evidence enforcement, formatting,
lints, and the test suite. Live verification against a deployed directory is
separate and needs a reachable service:

```sh
GAUGEWRIGHT_DIRECTORY_URL=https://… scripts/directory-check.sh
```

## Authority

- `contracts/product-routes.json` declares every route this service produces,
  with its evidence. `contracts/product-route.schema.json` is a copy of the
  manifest schema owned by `gaugedesk-src`; change it there first.
- `docs/` holds the public explanation of the blind-directory model.
- Company-wide policy — repository ownership, secret boundaries, and the
  development standard this guide implements — lives in the `GaugeWright`
  repository under `specs/systems.md`. This guide never restates it.

## Landing a change

Work on a branch cut from `origin/main`, run `scripts/check.sh`, and open a pull
request. Do not manually dispatch, rerun, enable, or disable GitHub Actions
workflows; the configured gates run on their own triggers, and a failed gate is
diagnosed locally and corrected in one coherent follow-up.

Toolchains are pinned in-repo: `rust-toolchain.toml` and `.node-version`. Change
the pin in the file rather than in a workflow.

## Workspace conventions

Worktrees live under one root per machine (`~/code/.worktrees/`), never in a
temporary directory that a reboot can remove. Primary checkouts rest on `main`
so a working tree is never mistaken for current truth. The Rust repositories
share one `CARGO_TARGET_DIR` so a second worktree does not rebuild the world.
