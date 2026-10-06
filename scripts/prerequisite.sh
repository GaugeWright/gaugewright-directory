# The one statement of what a green bar does about a tool the host has not
# installed. GENERATED into each adopting repository as `scripts/prerequisite.sh`
# from `tools/shared-checks/prerequisite.sh` in the GaugeWright repository, which
# owns it. Do not edit a rendered copy: `scripts/check-shared-harness.mjs`
# verifies it against the digest it was rendered with, so a local edit fails that
# repository's own gate. Change it here and re-render.
#
# A repository sources this from `scripts/check.sh` and sets `prerequisites`
# before the first call:
#
#   case "${1:-best-effort}" in
#     best-effort) prerequisites=best-effort ;;
#     *)           prerequisites=required ;;
#   esac
#   . scripts/prerequisite.sh
#
#   if prerequisite cargo-deny "the supply-chain policy check" \
#       "cargo install cargo-deny --locked"; then
#       cargo deny check licenses bans sources
#   fi
#
# Bare is the bar; any word at all — `required`, or a mistyped one — is the gate,
# so a typo can never quietly buy a skip.
#
# Why the split exists, rather than a hard exit: a tool the host *cannot* supply
# and a tool it simply has not installed are different facts, and a gate that
# fails with a message about the host when it has nothing to say about the change
# teaches the reader to wave red through. Four repositories each wrote their own
# version of this after an absent `mkdocs` failed a bar that had already passed
# everything it could answer; four copies of one idea is how they come to
# disagree, so there is now one.

# Steps that did not run for want of a tool, in the order they were skipped.
# A repository whose bar closes with a line about what it could not establish
# folds this into that sentence; one that does not may ignore it.
skipped_prerequisites=""

# Say, on stdout, that this bar did not establish something.
#
# A skip exits zero and the commit goes green, which is right — a gate that
# fails with a message about the host when it has nothing to say about the
# change teaches the reader to wave red through. What it leaves behind is a
# check nobody can distinguish from one that passed. On 2026-09-22 two were
# found in an afternoon: a verifier that had skipped on every host that ever
# ran it, for the life of its repository, and a section answered from the cache
# on machines where running it would have failed.
#
# So a skip says so in a fixed shape, on stdout, and the fleet keeps a ledger
# of them across hosts (GaugeWright tools/gate-unasserted.mjs). The question it
# answers is not "did this skip" — a Linux-only verifier skipping on a Mac is
# exactly what a skip is for — but "did EVERY host that ran this bar skip it",
# which means no machine has ever answered it.
#
# Name the section in the text. An item is identified by what it says, so two
# hosts phrasing one gap differently are two gaps to anything reading them.
#
#   unasserted "verify: bin/whip is a Linux binary and this host is $(uname -s)"
unasserted() {
    echo "#unasserted: $1"
    skipped_prerequisites="${skipped_prerequisites:+$skipped_prerequisites; }$1"
}

# $1 the tool, $2 what it gates, $3 the command that installs it,
# $4 (optional) the CI job that runs it anyway, default "the CI check job".
#
# Returns non-zero when the caller must skip, so a guarded step reads
# `if prerequisite …; then`. Under `required` it does not return at all.
prerequisite() {
    command -v "$1" >/dev/null 2>&1 && return 0

    if [ "${prerequisites:-required}" = required ]; then
        echo "$2 requires $1." >&2
        echo "install: $3" >&2
        exit 1
    fi

    echo "-- $2 SKIPPED: $1 is not installed --" >&2
    echo "   ${4:-the CI check job} installs it and runs this on every pull request." >&2
    echo "   To close the gap locally: $3" >&2
    unasserted "$2 not run: $1 is not installed"
    return 1
}
