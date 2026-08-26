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
- `docs/assets/brand.css`, `docs/assets/fonts/`, `docs/assets/mark-clear-64.png`
  and `overrides/partials/logo.html` are the shared documentation theme: the
  masthead signature, the palette, and the self-hosted brand faces that
  `docs.gaugewright.com` carries on both halves. They are a copy of what
  `gaugedesk-src` ships under the same paths, minus its capability badges;
  change the theme there first and re-copy. Nothing checks the two for
  divergence yet — the brand tokens they mirror are owned by the `GaugeWright`
  repository (DR-0077), and a projector for the rest of the theme is not built.
- Company-wide policy — repository ownership and secret boundaries — lives in
  the `GaugeWright` repository under `specs/systems.md`. The part of it that
  governs day-to-day work in this repository is carried below.

<!-- BEGIN GAUGEWRIGHT SHARED AGENT GUIDE v1 sha256:4a3bda1f2115ca679599c0e012737f1317d2ab2d0551cead28a2dce26329a401 -->

## Working in a GaugeWright repository

These rules hold in every active GaugeWright repository. This section is
generated from `tools/agent-guide/shared.md` in the `GaugeWright` repository,
which owns it. Edit it there and re-render; a local edit fails the check.

### The one check command

Each repository exposes exactly one command that runs its complete required
check set, and its configured gate invokes that same command:

```sh
scripts/check.sh
```

A documented local check and an enforced remote check therefore cannot mean
different things. Deep suites that need Maude, Nix, Playwright, built bundles,
or live providers are deliberately outside it and run on their own schedule or
dispatch; run the deep script covering the area you changed, not the whole set
out of habit.

### Landing a change

Work on a branch cut from `origin/main`, in your own worktree. Use atomic
commits with direct messages and keep unrelated changes out.

A pull request carries a substantive, cohesive body of work — it is not the
unit of every change. When the work is that, run `scripts/check.sh`, open the
pull request, and merge it; an implementation request already authorizes it, so
do not ask first. A small fix is not that: for a UI or UX tweak, a little
correction, or one step in a chain of related changes, run the check, commit on
your branch, and stop — the founder says when accumulated commits become a pull
request. Likely follow-up work is reason to hold whether or not you can see the
effort it belongs to. Holding is a smaller unit of delivery, not a request for
review: nothing is waiting on a reading of your diff. Open before the work is
complete only when you genuinely need the hosted gate's output, which the local
check cannot give you.

A change whose result is visual is shown to the founder on a running local
instance before its pull request. The gates report structure and behaviour, and
can all pass while the founder's read of the interface says the change is wrong.

Merge your own pull request once the required gates pass. Do not wait for a
review — GaugeWright has no independent reviewer, and the gates plus the commit
trail are the approval evidence. Hold only when you have a specific reason that
particular change should not land, and say what the reason is. A change that
ought to have been caught before landing is evidence the gate set is short; the
repair is the missing check, not a human reading of the diff.

Do not manually dispatch, rerun, enable, or disable GitHub Actions workflows.
The configured gates run on their own triggers. A failed gate is diagnosed
locally and corrected in one coherent follow-up commit rather than by hosted
reruns. One exception (DR-0084): a scheduler-dropped run — still queued thirty
minutes after creation with zero jobs started while sibling runs on the same
commit completed — may be re-run without a founder request, because a run that
never started carries no evidence a rerun could destroy. When the rerun API
fails on such a run too, amend the head commit and push with
`--force-with-lease` to mint fresh gating runs. Any other manual Actions
invocation needs an explicit founder request in the current conversation.

### Toolchains

Language toolchain versions are pinned in-repo — `rust-toolchain.toml`,
`.node-version` — and the automated gate derives its versions from those files.
Change the pin in the file rather than in a workflow.

### Worktrees and build output

Worktrees live under one root per machine, `~/code/.worktrees/`, never in a
temporary directory a reboot can remove. Primary checkouts rest on the default
branch so a working tree is never mistaken for current truth; a stale peer
checkout makes cross-repository checks fail on files that exist upstream.

Give each worktree its own build output — `CARGO_TARGET_DIR=<worktree>/target`
— and accept the cold build. Do not point a worktree at another checkout's
`target/`. Cargo keys some cached build-script and test binaries in a way that
survives the source change, so a shared directory lets one worktree rerun
another's `build.rs`, and lets a *deleted* worktree's cached test binary fail on
fixtures that are present and committed. Both failure modes read as a broken
`main` and describe neither tree. To tell stale cache from broken code:

```sh
strings target/debug/deps/<test>-* | grep -oE '/home/jack/code/[^ ]*<crate>'
```

Several worktree paths there means stale cache; `cargo clean -p <crate>` clears
it. The npm equivalent is real too: a symlinked `node_modules` silently
typechecks the main checkout, so re-point per-worktree links.

### Naming

Product repositories name their crates, binaries, environment variables, and
other operator-visible identifiers after the product they implement. The
company name identifies the company, shared company infrastructure, and
repositories that genuinely act for the company. No identifier may carry two
meanings for two processes composed in one environment.

### Cross-repository artifacts

Contract schemas, shared validators, and shared harness code have exactly one
owning source. Consuming repositories pin or digest-verify them rather than
copying, and divergence fails a check rather than passing silently.

### This guide

Each repository carries one agent guide, `AGENTS.md`, at its root. `CLAUDE.md`
is a symbolic link to it so that every tool reads the same file. Never replace
that link with a copy, and never write guidance into one that is not in the
other — a second filename may alias this guide but may not restate or
contradict it.

Guidance that applies to every repository belongs in the shared source above,
not in a repository's own sections. Repository-specific guidance — how to build
this repository, what its authority chain is, what its checks cover — belongs
outside the generated block and is never duplicated between repositories.

### Never commit

Secrets, tokens, passwords, private keys, tax identifiers, bank details,
signatures, private legal documents, customer-confidential material, or
personal data. Tracked automation configuration may carry variable names,
project identifiers, and paths — never resolved secret values.

<!-- END GAUGEWRIGHT SHARED AGENT GUIDE -->
