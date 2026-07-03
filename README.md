# gaugewright-directory

The GaugeWright **blind account directory** (ACCT-2, ADR 0054) — the readable
routing/identity layer of the GaugeWright account model, hosted so your
identity, devices, and placement pointers stay reachable even when your
machines are off. It is the one GaugeWright-operated service that is **free
and open source** (Apache-2.0), because its trust story depends on anyone
being able to audit that it is **blind by construction**.

Per account root it stores exactly two things:

- a **readable routing record**: the account root public key, device public
  keys, and placement pointers. No secrets, no work content.
- an **opaque sealed account blob**: hex ciphertext produced by your own
  devices. The directory holds no key and has no code path that opens it.

The contract: reads are public (`GET /directory/:root`); writes are signed and
fail-closed (`PUT /directory/:root` verifies against the record's *own*
`root_pubkey`); storage is durable and append-only. See
[docs/index.md](docs/index.md) for the full story.

## Building

The open platform repo ([gaugewright-workbench](https://github.com/jamesjscully/gaugewright-workbench))
is consumed as the `platform/` git submodule, pinned to a known-good commit;
this crate takes `path` dependencies into it (`platform/crates/*`) — the
ADR 0069 SPLIT-7 transitional mechanism until the platform crates are
published.

> **Note:** `gaugewright-workbench` is currently a **private** repository
> while this repo is public. The submodule pointer is public and harmless, but
> initializing it — and therefore building — requires access to the platform
> repo until it goes public.

```sh
git submodule update --init
cargo test                       # unit + integration (live tests are #[ignore])
cargo run                        # GAUGEWRIGHT_DIRECTORY_ADDR / _DB / _READY env
```

Live verification against a deployed instance:

```sh
GAUGEWRIGHT_DIRECTORY_URL=http://HOST:7901 scripts/directory-check.sh
```

Docs build: `mkdocs build --strict` (output under `target/directory-docs-site`).

## Sibling repos

- [gaugewright-workbench](https://github.com/jamesjscully/gaugewright-workbench)
  — the open platform (workbench, control plane, crates this service builds on).
- `gaugewright-cloud` — the private managed-services band (hosted control
  plane, settlement, embed host, attestation/KMS).

## History

Fresh-start history: this repo begins at the extraction commit (ADR 0069,
SPLIT-6). The archived `un-tie` monorepo is the history of record for
everything that came before.

## License

Apache-2.0 (see [LICENSE](LICENSE) and [NOTICE](NOTICE)).
