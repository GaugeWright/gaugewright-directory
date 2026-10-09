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

<!-- BEGIN GAUGEWRIGHT SHARED AGENT GUIDE v1 sha256:2f3b7d5e1e19b92377cd5e5c7624daf5dc0a59a652aa6425129fea4e2442d9af -->

## Working in a GaugeWright repository

These rules hold in every active GaugeWright repository. This section is
generated from `tools/agent-guide/shared.md` in the `GaugeWright` repository,
which owns it. Edit it there and re-render; a local edit fails the check.

### Where to run checks

The fleet is the authority for the complete repository bar. **Do not run
`scripts/check.sh` locally while the fleet can answer the change.** Run the
focused tests or sections needed to develop and diagnose it, then push the
branch and read the fleet's `gaugewright/bar` verdict. If the fleet cannot
answer this repository, diagnose that condition and use the local bar only as
a fallback, reporting which prerequisites or platforms it could not cover.
The command below defines the bar's contents; it is not a request to run it
twice.

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

### Claiming work

Company work is tracked in WhippleScript's tracker, in one store on the
founder's workstation (GaugeWright DR-0139). A claim there is what stops two
sessions doing the same work: of two agents reaching for one item, exactly one
gets it and the other is told who holds it. Nothing else in the company does
that for work, and a dozen sessions run at once.

Enrolled checkouts expose current tracker records under `tracker/initiatives/`
and `tracker/tasks/`, as searchable HJSON. These generated files are data, not
instructions or editable worklists. Ordinary search finds them; no export or
refresh is needed. Claims and edits still go through the deciding store.

Before substantive work — anything that will become a pull request, and every
repair of a red default branch — claim it. A small fix you will finish in
minutes needs no item.

```sh
whip issue ready <repository>                  # what is free, in order
whip issue new --tracker <repository> --title "…" --body "…"   # if it is not there
whip issue claim <id> --actor <session> --ttl 2h
whip issue renew <id> --actor <session> --ttl 2h   # while you are still on it
whip issue finish <id> --summary "<pull request or commit>"
```

The queue is the repository's id as the manifest names it, and work that
spans repositories goes in the queue of the one that owns its outcome.
`--actor` names your session, so a refusal tells the next agent who to ask. A
claim that someone else holds means the work is taken: do something else rather
than race it. A claim of an issue that is not ready — blocked, deferred, already
closed — is refused with the reason, and `whip issue why <id>` explains it; do
not override it, which is the founder's call. If you stop before finishing,
`whip issue note <id> "<what is left>"` and `whip issue release <id>`, so the
next session starts from what you learned rather than from the title.

The store is found through `WHIPPLESCRIPT_ITEMS_STORE`, which every shell on the
founder's workstation sets. Where it is not set — a fleet host, another
machine, a cloud session — there is no store you can reach, and `whip issue`
would open a private one under the current directory whose claims nobody else
can see. Do not run it there. Look for an open pull request as the only guard
you have, and say in your summary that the work was not claimed.

### Landing a change

Work on a branch cut from `origin/main`, in your own worktree. Use atomic
commits with direct messages and keep unrelated changes out.

A pull request carries a substantive, cohesive body of work — it is not the
unit of every change. When the work is that, open the pull request; the fleet
merges it after its complete per-change gate passes. An implementation request
already authorizes it, so do not ask first. A small fix is not that: for a UI or UX tweak, a little
correction, or one step in a chain of related changes, commit on your branch
and stop — the founder says when accumulated commits become a pull request.

Do not run the whole bar locally first. The fleet answers a pull request's head
or a default-branch push on a machine built for it, warm, and its verdict is
the one that counts. Running the same bar on the workstation beforehand does
the work twice and puts the second copy on the machine somebody is trying to
use. Measured on
2026-09-22: a dozen sessions on one laptop, each running a full bar before
pushing, took the load average past 140, and two separate failures were
misdiagnosed that day because a wall-clock number read under that load
described the machine rather than the change.

Before a pull request, an agent in a materialized workspace can request an
early fleet answer for a staged snapshot: run
`node "${GAUGEWRIGHT_WORKSPACE:-$HOME/code}/GaugeWright/tools/agent-check.mjs" --checkout=. --job=bar`
from the repository under test. Stage exactly what should be checked first;
the command refuses to omit unstaged or untracked files unless
`--allow-excluded` acknowledges them. It leaves HEAD, the index, and the
working tree in place, waits for the answer, and removes the temporary remote
ref. The result is under `gaugewright/agent/bar`: useful feedback, never a
required gate verdict. The repository manifest lists which jobs may be asked
for this way; WhippleScript also allows `--job=new-refusals`. A normal pull
request still receives its own required checks.

Run locally what you are actually working on — the section you changed, the
test you are iterating on, `scripts/section.sh <name>` — which is fast and
tells you something the fleet's verdict would tell you slower. Run the whole
bar locally only when the fleet cannot answer the repository. The one check command has not changed and
neither has what the gate runs; what changed is that a machine exists whose job
is to run it, and yours is not it. Likely follow-up work is reason to hold whether or not you can see the
effort it belongs to. Holding is a smaller unit of delivery, not a request for
review: nothing is waiting on a reading of your diff. Open before the work is
complete only when you genuinely need a verdict you cannot get otherwise.

A change whose result is visual is shown to the founder on a running local
instance before its pull request. The gates report structure and behaviour, and
can all pass while the founder's read of the interface says the change is wrong.

The fleet merges each non-draft pull request targeting a declared default
branch once all required contexts are green on its head and current base.
Draft the pull request when it needs to remain open. GaugeWright has no
independent reviewer, and the gate plus the commit trail are the approval
evidence. A change that
ought to have been caught before landing is evidence the gate set is short; the
repair is the missing check, not a human reading of the diff.

A RED `main` is shared, so its repair is claimed like any other work. Several
sessions run at once and each meets the same red gate at the same moment, so the
obvious repair gets written more than once — two sessions independently bumped
the same two advisory-flagged lockfile entries on 2026-09-02, and the second
learned it only when its rebase conflicted. The waste is not the duplicate
branch, which is cheap to close; it is that the second session spent its time
believing it was unblocking work already unblocked. Looking for an open pull
request did not prevent it, because both sessions looked before either had
opened one. So before repairing, find the item — `whip issue list --tracker
<repository>`, titled `red main: <repository>` — and claim it, filing it if it
is absent. A refused claim means someone is already on it.

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
the next sweep without a rebase. A commit is answered once, so to have an
unchanged tree answered again, give it a new commit
(`git commit --amend --no-edit --date=now`). A verdict that
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

If you merge a green on an older base anyway, check what `main`'s new commits
*test* as well as what they touch. A test that enumerates a shape — a bundled
vocabulary, a registry, a route manifest, a pinned count — collides with a
change that extends that shape while sharing no file with it. whipplescript-src
#633 was green 38 commits behind, touched nothing #654 had touched, and turned
`main` red on #654's new inventory test. When one does, rebase and take a
fresh verdict.

A red on a `carries-*` section is about the GaugeWright repository, not your
change. It means a file GaugeWright renders into this repository — this guide,
the documentation theme, the brand tokens — has moved there and not yet here,
so every pull request and `main` go red on it together. GaugeWright's
`carried-artifacts` job opens and merges a `carried-artifacts` pull request
here within about four hours; do not re-render by hand, and do not repair it
in your branch. Once it lands, your red is answered again with `main`. If that
pull request is itself held red by something unrelated, rebase your branch
onto `origin/carried-artifacts` and land it with a merge commit rather than a
squash, which lands both; say so on both pull requests.

The status links to that transcript: `target_url` on it is the whole bar as
the host that ran it saw it, served by that host over the fleet's network.
Read it before reproducing anything — a red names its section there, and the
skips a green took are the other half of what the bar actually established.
It is a link to a machine's disk and not an archive: the host that wrote it
serves it, transcripts are pruned, and a host that is off serves none. A
verdict with no link is a host that advertises none, which is what every
verdict carried before this.

The public names in those links sit behind Cloudflare Access. Never run
`cloudflared access` against one, nor ssh to a host through a `.cf` alias:
each opens an Access sign-in in the founder's browser, once per hostname and
again whenever its 24-hour session lapses. Between 2026-09-25 and 2026-10-01
agents read transcripts with `cloudflared access curl` thirty-four times, and
the founder was asked to approve sign-ins they had not started. On the
founder's workstation read one through the GaugeWright reader instead, rather
than giving up on it or reproducing the bar blind:

```sh
node "${GAUGEWRIGHT_WORKSPACE:-$HOME/code}/GaugeWright/tools/gate-log.mjs" <repository> <commit|#pull> --out=<file>
node "${GAUGEWRIGHT_WORKSPACE:-$HOME/code}/GaugeWright/tools/gate-log.mjs" <target_url> --out=<file>
```

It reaches the host over the ssh configuration the workstation already holds
and reads the transcript from the host's own server, so it needs no credential
of its own. `--context=` names a job's status other than `gaugewright/bar`.
It reads a host's records the same way, given
`https://<host>.gaugewright.com/events.jsonl` or `measurements.jsonl`. When it
cannot reach a host, say so; do not sign in instead.

Whether the fleet itself is well is a different question from any verdict.
It asks whether every host is still sampling, has room on its disk, passed its
newest proof, and whether the forge budget they share is above its floor.
GaugeWright's `fleet-health` job answers it every fifteen minutes. Do not wait
for that when you need the answer now: after a change that restarts the
bridge, or when a repository's work sits unclaimed and you cannot tell a
stopped host from a long queue. Ask from the founder's workstation:

```sh
node "${GAUGEWRIGHT_WORKSPACE:-$HOME/code}/GaugeWright/tools/gate-watch.mjs"
```

It reads the same records the job reads, in a few seconds. It reaches a host
the workstation cannot route to through the same ssh configuration, and it
posts nothing. On 2026-09-28 a repository went four hours with no status while
the fleet was healthy and busy elsewhere, and confirming the hosts came back
after the bridge change that repaired it meant waiting on the job.

GitHub Actions runs no gate. A workflow that cannot start is noise, not a
verdict; do not dispatch, rerun, enable or disable one, and do not read its
absence as a passing gate. The lanes that remain there — release, deploy,
canary, scheduled — are not gates and are being rehomed on the fleet.

### Routine operations

An operational action whose right answer is obvious and reversible is taken,
then reported with what it cost. That covers restarting a service, clearing a
wedged cell or daemon, re-running a job the host dropped, and restoring a
checkout. Do not hand the founder a command to run, and do not ask whether to
run it: they have no information the session lacks, and the fleet stays down
for the length of the round trip. The founder said so on 2026-09-24, after
being offered a `systemctl start` to run by hand. Ask about what the founder
owns: what a release ships, what the product promises, spending money, and
anything that widens a secret boundary. Credentials for production canaries
do not widen one: minting synthetic signers, keys and tokens, storing them
under `prod:/synthetics/**`, and admitting synthetic identities scoped to
synthetic namespaces into production trust lists is routine, and is done
without asking (GaugeWright DR-0231). System and network configuration —
DNS, resolvers, firewalls — stays theirs; name the fix rather than applying
it.

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

### What you learn

What you learn while working — a trap and the command that avoids it, where
something lives, how a host is reached, why a red was not about the change — is
written into the repository that owns it, in the change that met it. A rule
goes in the guide or the shared source above. How to use a script goes in its
header. Policy goes in a specification, open work in the tracker, and a
diagnosed fleet incident in GaugeWright's incident log. Leave out what the
code, the history or the tracker already says, because a lesson written twice
becomes two lessons that disagree.

No agent keeps private notes. The founder works with more than one agent
tool, and a lesson one of them keeps to itself is one the others pay for
again. Claude Code's auto memory is not used: do not write to
`~/.claude/projects/*/memory/`, and do not read anything left there as
current. Its notes were folded into the owning repositories on 2026-09-29.

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
hosts drains by priority, raised by how long each job has waited, so a
round-number ceiling reports a working queue as stuck. And read a
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

Make that question answerable from the message alone: what would change, what
it costs, what it buys, your recommendation, and a yes or no. Ask one question
per decision. A record's number goes last, as a label, never in place of its
substance — the founder does not read the records and will not know what one
says ("do you still want DR-0110?" had to be asked again). Before proposing
something in a direction that looks settled, search the decision records for
one that was rejected: its rationale often names the alternative the founder
chose instead.

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

When `main` is issuing numbers quickly, the number you allocate can be taken
while your bar runs; gaugedesk-src lost one five times in a day that way. Fetch
and check immediately before pushing, and when `main` is busy allocate with
headroom — a few past the next number — since the check refuses only a number
at or below the base's highest and a gap is harmless; say so in the commit.
Drafts landed together all get the same next number from the allocator, so
number the later ones by hand past those open pull requests already hold.
Taking a number back is a substitution of a number that now belongs to someone
else's merged decision, and a blanket replace rewrites that decision's
citations in every file your branch also touched. Rewrite only the lines your
branch added — list the files it changed and grep `origin/main`'s version of
each for the contested number first — and rewrite your unpushed commit
messages too.

### Toolchains

Language toolchain versions are pinned in-repo — `rust-toolchain.toml`,
`.node-version` — and the automated gate derives its versions from those files.
Change the pin in the file rather than in a workflow.

### Versioning

A product release is numbered `X.Y.Z`, and the number means what Cargo already
assumes (GaugeWright DR-0170). Before 1.0, raise the middle number for a release
that breaks anything a user or consumer relies on — stored data, a client's
protocol with a Home or the Hub, a command line, configuration, a published
crate's or package's interface — and the last number for everything else. A
break numbered in the last place reaches every crates.io dependent without any
of them choosing it. Going to 1.0 is the founder's decision.

A product has one version, declared once in its trunk's workspace manifest.
Every crate, package, bundle and mobile app inherits it or is generated from
it, and the bar checks that they agree; a store's build number is a separate
integer. Change the version only in the commit that cuts the release. Tag
`vX.Y.Z` in the trunk at the source commit and in the mirror at the publishing
commit. Never move or delete a pushed tag; repair a failed cut under the next
number. The cut refuses without a `## [X.Y.Z] — YYYY-MM-DD` entry in
`CHANGELOG.md` that says what changed and, for a breaking release, what a user
or dependent must do.

A repository that deploys continuously has no version: its commit identifies
what it serves, and a version field a toolchain demands is `0.0.0` and
unpublished. A contract, schema or file format you own carries a whole-number
`vN` that rises only for a change a reader of the previous version could not
accept. A format someone else owns keeps their scheme, and an identifier of
ours in another form converges when it next changes, not in a sweep.

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

A worktree nested inside another checkout — Claude Code puts its own under
`<checkout>/.claude/worktrees/` — lets cargo walk up out of a workspace that
excludes a directory and into the enclosing checkout's `Cargo.toml`, so a check
like `native-crates` fails there with "current package believes it's in a
workspace when it's not". Run it from a worktree under `~/code/.worktrees/`.
Remove only the worktrees you created, by exact path and never by pattern: a
cleanup glob once removed two other sessions' worktrees with their uncommitted
edits.

### Declaring what a check reads

Every section of a bar is a Buck2 target whose `srcs` say what it reads. Its
verdict is keyed on those inputs and nothing else. On the fleet's Linux hosts,
a repository whose manifest entry sets `gate.sandbox` runs it in a read
sandbox where an undeclared file does not exist. So `srcs` must cover
everything the section's *command* reads, not only what its question is about:
`cargo metadata` opens every member's crate roots, and `rust-toolchain.toml`
decides which compiler answers. A Mac has no sandbox, so a section that passes
on the workstation can still fail on the fleet.

- `glob(["**/*"])` never matches a dotfile and never enters a dot-directory.
  Name `.gitignore`, `.github/**`, `.cargo/**` and a fixture's `.buckconfig`
  outright, and name a fixture's new dotfile in the same commit that adds it.
- A glob stops at a package boundary. A subdirectory with its own `BUCK` —
  a submodule that carries one — matches nothing, silently. Prove a
  declaration with `buck2 uquery "inputs(<target>)"`: a nonzero count for the
  path is the only evidence that it declares anything.
- The sandbox reports the first undeclared read and says nothing about the
  second, so budget several passes. A failing step prints what the sandbox
  bound, what it noticed and withheld, and a command that re-enters it; read
  that first. An EROFS from a build script is a write, not a read.
- A section that has only ever been answered from the cache has not been shown
  to work. When a section is new, newly moved into the graph, or has only
  skipped on the hosts that ran it, make it run once — change one of its
  `srcs`, or run its command in the cell — before believing it. When a red
  seems caused by an unrelated change, check first whether that change only
  invalidated a cache key.
- A cell's own `.buckconfig` must ignore its build output as `**/target/**`.
  The ignore is per cell, it is read only when a daemon starts, and macOS never
  shows the problem. Without it, cargo's writes flood the file watcher and a
  parallel section fails with "The transaction was cancelled".
- To test a change to what a section sees, `buck2 kill` and build with
  `--no-remote-cache`. The daemon otherwise answers from memory and the cache
  from anyone's run; a different `instance_name` isolates nothing.
- Where Rust builds natively (`scripts/buckify-crates.py` is present), the
  rendering is part of the source. Re-render after any change to a
  `Cargo.toml`, `Cargo.lock` or submodule pin, and again after every rebase,
  because a clean merge of two renderings still leaves stale hashes. A native
  test sees only the files its crate declares in
  `[package.metadata.native-tests] data`, and a path a test builds with
  `.join(..)` at run time is invisible to the renderer, so declare that
  directory. `include!(concat!(env!("CARGO_MANIFEST_DIR"), …))` breaks under
  Buck2; write the path relative to the file. A new build script needs a
  reindeer fixup, and a fixup's opt-level must equal the top-level workspace's
  `[profile.dev.package.*]`.

### The founder's workstation

An agent's shell there is zsh, and it may not be a login shell. When
`command -v node cargo` finds nothing, the profile did not run:

```sh
export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$HOME/.local/bin:$PATH"
eval "$(/opt/homebrew/bin/fnm env --shell bash)"
fnm use    # inside the checkout; reads .node-version
```

zsh does not word-split an unquoted parameter, so `for x in $list` runs once
over the whole string. A status wait written that way never matches its
contexts and waits forever on a green gate; that held a release two days. Write
any loop over a list to a file and run it with `/opt/homebrew/bin/bash`, over
an array. Quote globs, because zsh errors on one that matches nothing. Never
`pgrep -f` or `pkill -f` a pattern that also appears in your own command line —
write `[n]ode tools/gate.mjs` — or the wait blocks on itself and the kill takes
your session.

`/bin/bash` is 3.2 and Homebrew's is 5.3, and a bare `bash` may be either, so
run every shell test under both. Three 3.2 behaviours make a test green while
it checks nothing. `set -e` does not fire on a failing `[[ … ]]`, so give each
assertion its own `|| { echo …; exit 1; }`. An EXIT trap sees `$?` as 0 after
an abort, so set `completed=1` on the last line and have the trap fail when it
is unset. A `case` inside `$( … )` is a parse error, so move it into a
function. Prove a test fails by breaking what it tests, not by reading it.

macOS deletes files under `/private/tmp` that have not been read for three
days, a session scratchpad included. Anything that must outlast a session —
a toolchain, a probe, a virtualenv — goes under `~/.local/opt/`. macOS has no
`setsid`. To run a long job detached, write a launcher that sets the PATH,
redirects its transcript to a file and appends its exit status. Start it with
`nohup <launcher> >/dev/null 2>&1 </dev/null & disown`, and wait on the exit
line.

An agent's command sandbox cannot reach the login keychain. Anything that
reads it — `infisical run`, `gh` on a keychain-stored token — fails there with
`exit status 51`, "The user name or passphrase you entered is not correct",
which reads as a broken login and is not one. `security show-keychain-info
~/Library/Keychains/login.keychain-db` tells the two apart: it fails the same
way inside the sandbox and succeeds outside it. Run the command outside the
sandbox; do not ask the founder to log in again.

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

Nor print one. A value that reaches an agent's transcript has left the
company's custody: it sits unencrypted in the tool's session log and was sent
to a model as context. Nothing about learning a secret's name requires reading
its value. `infisical secrets` prints every value at a path, with or without
`--plain`; on 2026-09-22 it put the Apple signing bundle and the updater key
in a transcript and forced their revocation. Give a process its secrets with
`infisical run … -- <command>`, test for presence under it with
`sh -c '[ -n "$NAME" ] && echo SET'`, and read names from the consuming code,
`gh secret list`, or a provider's metadata. Never read or `cat` a credential
file on a host to see what is in it; list it by name.

<!-- END GAUGEWRIGHT SHARED AGENT GUIDE -->
