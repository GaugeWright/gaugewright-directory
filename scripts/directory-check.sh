#!/usr/bin/env bash
# directory-check.sh — live verification of the ACCT-2 blind directory against the deployed
# service. Drives the real gaugewright-directory over HTTP: a signed publish (204), a public
# fetch (round-trips the readable record + opaque sealed blob), and a forged write (401).
#   GAUGEWRIGHT_DIRECTORY_URL=http://HOST:7901 scripts/directory-check.sh
set -euo pipefail
cd "$(dirname "$0")/.."
: "${GAUGEWRIGHT_DIRECTORY_URL:?set GAUGEWRIGHT_DIRECTORY_URL to the deployed directory (e.g. http://20.57.147.64:7901)}"
echo "== verifying the blind directory at $GAUGEWRIGHT_DIRECTORY_URL =="
GAUGEWRIGHT_DIRECTORY_URL="$GAUGEWRIGHT_DIRECTORY_URL" \
  cargo test -p gaugewright-directory --test directory_live -- --ignored --nocapture
echo "== ACCT-2 live directory check PASSED =="
