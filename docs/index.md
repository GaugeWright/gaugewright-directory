# GaugeWright Blind Account Directory

The directory is the readable routing/identity layer of the GaugeWright
account model (`ACCT-2`, ADR 0054), hosted so your identity, devices, and
placement pointers stay reachable even when your machines are off. It is the
one GaugeWright-operated service that is **free and open source**
(Apache-2.0), because its trust story depends on anyone being able to audit
that it is blind by construction.

## What it stores — and what it cannot see

Per account root, exactly two things:

- a **readable routing record**: the account root public key, device public
  keys, and placement pointers. No secrets, no work content (`INV-10`).
- an **opaque sealed account blob**: hex ciphertext produced by your own
  devices. The directory holds no key and has no code path that opens it.

## The contract

- **Reads are public.** Routing is meant to be found:
  `GET /directory/:root`.
- **Writes are signed and fail-closed.** `PUT /directory/:root` verifies the
  signature against the record's *own* `root_pubkey`, so nobody can overwrite
  your routing but you.
- **Durable and append-only** (`INV-6`): each signed publish appends to the
  reserved `directory` scope; on restart the read cache is rebuilt by folding
  the log, latest-publish-per-root wins (`INV-5`).

## Running it

```sh
GAUGEWRIGHT_DIRECTORY_ADDR=0.0.0.0:7901 \
GAUGEWRIGHT_DIRECTORY_DB=/var/lib/gaugewright/dir.db \
gaugewright-directory
```

Unset `GAUGEWRIGHT_DIRECTORY_DB` for a pure in-memory instance (dev). An
optional `GAUGEWRIGHT_DIRECTORY_READY` file path is touched once the listener
binds, for container healthchecks.

Live verification against a deployed instance:
`GAUGEWRIGHT_DIRECTORY_URL=http://HOST:7901 scripts/directory-check.sh`.
