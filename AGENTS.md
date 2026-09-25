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

It covers the product-contract manifest with evidence enforcement, the advisory
audit, formatting, lints, the test suite, and the strict documentation build.

Two of those need a tool a workstation may not have installed — `cargo-audit`
and `mkdocs`. A bare `scripts/check.sh` reports each absence, names the command
that closes it, and runs everything else, so a bar that has answered everything
it could about the change says so rather than failing about the host. The CI job
installs both and runs `scripts/check.sh required`, which refuses to skip, so
what the gate covers is unchanged.

Live verification against a deployed directory is separate and needs a reachable
service:

```sh
GAUGEWRIGHT_DIRECTORY_URL=https://… scripts/directory-check.sh
```

## Authority

- `contracts/product-routes.json` declares every route this service produces,
  with its evidence. `contracts/product-route.schema.json` is a copy of the
  manifest schema owned by `gaugedesk-src`; change it there first.
- `docs/` holds the public explanation of the blind-directory model.
- The shared documentation theme is owned by the `GaugeWright` repository and
  rendered here (DR-0093): the marked block in `docs/assets/brand.css`,
  `overrides/partials/logo.html`, `docs/assets/mark-clear-64.png`, and
  `docs/assets/fonts/`. Change it there with `tools/docs-theme.mjs --write`, not
  here — `scripts/check-docs-theme.mjs`, itself rendered, rejects a local edit.
  Rules for this repository's own documentation belong below the closing marker,
  which is the one part of that stylesheet this repository owns.
- Company-wide policy — repository ownership and secret boundaries — lives in
  the `GaugeWright` repository under `specs/systems.md`. The part of it that
  governs day-to-day work in this repository is carried below.

<!-- BEGIN GAUGEWRIGHT SHARED AGENT GUIDE v1 sha256:43c61f1c92ce081b02eb3f3a56f55d90046dd0ddaf86049618b0d5d12af4b01e -->

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

That promise is about *meaning*, not about every host answering every question.
A section whose prerequisite the machine genuinely cannot supply — Debian
archive tooling on a Mac, a Windows cross-compile off Windows — skips there and
says so, naming the job that does run it. A section whose prerequisite the
machine simply has not installed is the harder case, and the rule is the same
one the native shells already follow: `scripts/check.sh` skips it, printing the
exact command that closes the gap, while the invocation the gate runs refuses.
What the gate covers never changes; only where the answer is available does.

Do not write that split by hand. `scripts/prerequisite.sh` states it once, is
rendered from the GaugeWright repository, and is sourced by the bar that uses
it; a repository sets `prerequisites` from its own argument and calls
`prerequisite <tool> <what it gates> <install command>`. The gate passes the
word that makes the install steps above it load-bearing — `scripts/check.sh
required`, or a section's own `required` — so a dropped install reddens the gate
rather than quietly buying itself a skip.

A gate must never fail with a message about the host when it has nothing to say
about the change. That failure teaches the reader to wave red through, and it
costs more than the check was worth — `stat -c%s` reported as a missing module,
a preview bound to `::1` reported as a thirty-second timeout, an absent `mkdocs`
failing a bar that had already passed everything it could answer. That last one
is why the helper exists rather than the advice alone: the rule was written
against it and it stayed in place, and a sweep then found the same hard exit
around `cargo-audit`, `cargo-deny`, `wasm-bindgen`, `wrangler` and `mkdocs` in
four repositories, one of them guarding the first step of a bar so that a
workstation without the tool learned nothing whatever about its change
(DR-0127). Prose that every repository has to re-implement gets
re-implemented four ways.

If a prerequisite is genuinely required — a step that cannot proceed without it
rather than one that can be reported and skipped — name it, name its remedy for
the host in front of you, and fail on the section that needs it rather than on
the whole run. A hard failure trains the reflex, and a silent skip is a gate
that has quietly stopped gating; a bar that closes by saying what it could not
establish is neither.

### Landing a change

Work on a branch cut from `origin/main`, in your own worktree. Use atomic
commits with direct messages and keep unrelated changes out.

A pull request carries a substantive, cohesive body of work — it is not the
unit of every change. When the work is that, open the pull request and merge it
on the fleet's verdict; an implementation request already authorizes it, so do
not ask first. A small fix is not that: for a UI or UX tweak, a little
correction, or one step in a chain of related changes, commit on your branch
and stop — the founder says when accumulated commits become a pull request.

Do not run the whole bar locally first. The fleet answers a push on a machine
built for it, warm, in a couple of minutes, and it is the verdict that counts;
running the same bar on the workstation beforehand does the work twice and does
the second copy on the machine somebody is trying to use. Measured on
2026-09-22: a dozen sessions on one laptop, each running a full bar before
pushing, took the load average past 140, and two separate failures were
misdiagnosed that day because a wall-clock number read under that load
described the machine rather than the change.

Run locally what you are actually working on — the section you changed, the
test you are iterating on, `scripts/section.sh <name>` — which is fast and
tells you something the fleet's verdict would tell you slower. Run the whole
bar when you have a reason: no fleet host can answer your repository, you are
changing the bar itself, or you are about to land something whose failure would
be expensive to discover on `main`. The one check command has not changed and
neither has what the gate runs; what changed is that a machine exists whose job
is to run it, and yours is not it. Likely follow-up work is reason to hold whether or not you can see the
effort it belongs to. Holding is a smaller unit of delivery, not a request for
review: nothing is waiting on a reading of your diff. Open before the work is
complete only when you genuinely need a verdict you cannot get otherwise.

A change whose result is visual is shown to the founder on a running local
instance before its pull request. The gates report structure and behaviour, and
can all pass while the founder's read of the interface says the change is wrong.

Merge your own pull request once the gate is green on its head commit. Do not
wait for a review — GaugeWright has no independent reviewer, and the gate plus
the commit trail are the approval evidence. Hold only when you have a specific reason that
particular change should not land, and say what the reason is. A change that
ought to have been caught before landing is evidence the gate set is short; the
repair is the missing check, not a human reading of the diff.

A RED `main` is shared, so before repairing one, look for an open pull request
already repairing it. Several sessions run at once and each meets the same red
gate at the same moment, so the obvious repair gets written more than once — two
sessions independently bumped the same two advisory-flagged lockfile entries on
2026-09-02, and the second learned it only when its rebase conflicted. The waste
is not the duplicate branch, which is cheap to close; it is that the second
session spent its time believing it was unblocking work already unblocked. This
is the collision the decision-record allocator exists to prevent, arriving
through a door that has no allocator.

The gate is the company's own fleet, not the forge (DR-0131). It posts a
commit status per verdict, and a repository has more than one.
`gaugewright/bar` is its `scripts/check.sh required`. Each entry in its
manifest `gate.jobs` adds one more under its own context, and the entry says
which of three kinds it is. A job with neither `every` nor `pulls` is
per-change and posts on a pull request head, like whipplescript-src's
`gaugewright/bar/new-refusals`. A job with `pulls: false` is per-change on the
default branch only: it answers every head `main` reaches and never posts on a
pull request, like the platform compiles, gaugedesk-src's
`gaugewright/bar/macos` and whipplescript-src's `gaugewright/bar/windows`
(GaugeWright DR-0146). A red there names the commit that broke that platform
and is repaired like any red `main`, before the release lane, which builds the
platform again, would ship it. A job with `every` is a cadence check due on
the default branch and never posts on a pull request at all, like the
GaugeWright repository's seven 24h and 7d contexts.
`gaugewright/host/<name>` is not a job and is not about any change: a host's
own proof that it can run work, which is where to look when every pull request
from one host has gone red.

So the combined `.state` is not the verdict, and it misleads in both
directions. It covers only the contexts that have POSTED, so `success` can
mean no more than that nothing which reported said no while a per-change job
is still unclaimed — on 2026-09-22 gaugedesk-src#708's `bar/macos`, then
owed by pull requests, posted twenty minutes after its `bar`, and merging on
the combined state would have skipped the desktop compile. And a host proof turns it red while every bar is
green: GaugeWright's `main` read `failure` on `gaugewright/host/winworker`,
which had failed its own proof and stopped claiming, with `gaugewright/bar`
green beside it. Read `gate.jobs` from `tools/vmr/manifest.json` to learn
which contexts a change is owed, list every status the commit carries, and
judge those; never read `.state` alone, and never read `.statuses[0]`, which
picks an arbitrary context. A red default branch is not necessarily a red bar.

A pull request is answered at the forge's
test merge of its head onto `main`, so the verdict is about what would land,
and it is bound to the head commit that was pushed. `gh pr checks` shows it.
The description says which host answered and against which `main`; `pending`
means a host has claimed the commit and is running it; `error` means the gate
did not answer — the bar could not be run or exceeded its ceiling — which is a
different fact from `failure` and is never waved through as either. A red pull
request is answered again when `main` moves, so a fix on `main` reaches it at
the next sweep without a rebase. A verdict that
arrives in seconds is the fleet's action cache answering for sections whose
inputs the change did not touch, and its transcript is the original run's.

A green is not re-run on every push to `main`, but it does expire. The verdict
was about a test merge onto a base, and once that base is no longer the one
the change would land on, the green describes a merge that no longer exists —
while reading exactly like one taken a minute ago, because nothing on the
commit records how far behind it is. So the bridge answers a green pull
request again once its base has moved and a day has passed. Two changes here
were merged on greens taken against bases three days and six weeks old before
that rule existed. Until a re-answer arrives, the description says which
`main` the verdict was computed against: if that is not the current one, what
the fleet approved is not what you are about to merge.

The status links to that transcript: `target_url` on it is the whole bar as
the host that ran it saw it, served by that host over the fleet's network.
Read it before reproducing anything — a red names its section there, and the
skips a green took are the other half of what the bar actually established.
It is a link to a machine's disk and not an archive: the host that wrote it
serves it, transcripts are pruned, and a host that is off serves none. A
verdict with no link is a host that advertises none, which is what every
verdict carried before this.

GitHub Actions runs no gate. A workflow that cannot start is noise, not a
verdict; do not dispatch, rerun, enable or disable one, and do not read its
absence as a passing gate. The lanes that remain there — release, deploy,
canary, scheduled — are not gates and are being rehomed on the fleet.

### Stale documentation

When you find documentation that no longer describes what is true — a README,
a runbook, a comment, this guide, a specification that lags a decision already
folded or a system as it now runs — correct it. Do not ask whether to. Make
the correction in the change you are already making when it is in the same
repository, or as its own commit on a branch in the repository that owns it,
and say in your summary what you corrected. Where the repository's governance
needs a record for a specification edit, a correction to match what is already
true or already decided is a routine decision; write it and land it (GaugeWright
DR-0138).

This covers bringing text into line with the truth, not choosing the truth.
When the documentation and the system disagree and it is not settled which one
is right, that is a question, not a stale page: ask it, plainly, and correct
whichever side the answer says is wrong. The founder was being asked, over and
over, whether to update documentation that was plainly out of date, and the
answer was always yes.

### Waiting on a gate

A gate is waited on, never polled from the conversation. Arm one watch and let
it report. For a question with one answer — has this run finished — use a
backgrounded wait that exits when the run is terminal, which delivers exactly
one notification; use a streaming monitor only when every occurrence is wanted.
The watch matches every terminal state — success, failure, cancellation, and a
run that never started — because a filter that reports only success is
indistinguishable from a hung run, and a poll that was never issued is
indistinguishable from a passing gate.

For the fleet's verdict the answer is the commit statuses: wait on
`gh api repos/<owner>/<repo>/commits/<sha>/status` until every per-change
context the change is owed — `gaugewright/bar` and, on a pull request, each
`gate.jobs` entry with neither `every` nor `pulls: false` — has posted and is terminal, and read those states from
the reply, not the combined one and not a workflow list. Waiting on the
combined state alone returns early, because a context that has not posted yet
is not holding it back. Size the wait for the gate's
real cost: a cold bar on a busy host is minutes, and a queue with several
hosts drains in the order of what only one platform can answer first, then
age, so a round-number ceiling reports a working queue as stuck. And read a
check's exit status from the check: a pipeline returns its last command's status, so `scripts/check.sh |
grep` under `set -e` commits a red bar. Write the transcript to a file and read
it afterwards. The founder pinged agents whose polling had silently stopped on
gates long finished; the session that then repaired three gates for reporting
success while checking nothing hand-polled, over-notified, mis-sized a
merge-queue wait, and piped its own green bar into `grep` (DR-0123).

### Decision records

A decision record is called that — `DR`, cited `DR-NNNN` — in every
repository. Do not write `ADR` in new text; correct an old citation when you
next edit the text around it, not in a sweep.

Every record declares its class. A **founder decision** introduces a plan or
direction not already settled, makes an architectural or design choice with
serious tradeoffs, touches scope, money, legal posture, a customer commitment,
or governance, or reverses a decision the founder accepted; only the founder
decides it. A **routine decision** is one
whose answer follows from settled policy and specifications, or is obvious and
reversible with no serious tradeoff; its author accepts it, marks it routine,
and writes one sentence saying why. The founder reads routine decisions when
they choose and reverses one they disagree with. When in doubt the record is a
founder decision — but doubt means a real tradeoff you cannot resolve from what
is settled, not the reflex to ask. The founder's time was going to "yes,
approve" on records they had not read because the answer was obvious, and a
gate that is waved through is not a gate (GaugeWright DR-0130).

The founder decides in the conversation, not by reading the record. When they
have chosen the direction in the conversation that produced the work, that
choice is the acceptance. Write the record, mark it accepted by the founder
with a rationale saying what they chose, and land it in one commit. Never ask
the founder to review or accept a record. If a founder-class question is still
open, ask the question itself in the conversation, with the choice and its
tradeoff in plain terms, and record the answer. Even after DR-0130 the founder
was still accepting founder-class records without reading them, because all of
their alignment happens in the chat (GaugeWright DR-0136).

A record's text may be revised in place to correct or clarify it. A record is
never deleted or renumbered, and a reversal is a new record or a status change
on the old one, never its removal.

A repository that keeps numbered decision records assigns a record's number
when it lands, never when it is drafted. Draft under the repository's
placeholder — the four digits replaced by `XXXX`, one drafted decision per
branch — and cite the placeholder in prose. At landing, run the repository's
allocator: it reads freshly fetched `origin/main`, takes the number after the
highest one there, and rewrites the placeholder across the branch's changes.
The check refuses a placeholder and any newly originated number that does not
exceed every number on the current base, so two sessions racing for one number
produce a red gate and one rerun of the allocator on the loser, never a merged
duplicate — the failure that issued one number to two decisions in three
repositories in one month. A merged number is never reassigned. A citation
that crosses a repository boundary names the owning repository beside the
number, because each repository runs its own sequence.

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
