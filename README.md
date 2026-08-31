<p align="center">
    <img src="assets/rocket-solid.png" width="128" height="128">
</p>

# Rust `algonaut`

[![Crate](https://img.shields.io/crates/v/algonaut.svg)](https://crates.io/crates/algonaut)
[![Docs](https://docs.rs/algonaut/badge.svg)](https://docs.rs/algonaut)
[![CI](https://github.com/manuelmauro/algonaut/actions/workflows/general.yml/badge.svg?branch=main)](https://github.com/manuelmauro/algonaut/actions/workflows/general.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/algonaut.svg)](#license)

A Rust SDK for the [Algorand](https://www.algorand.com/) blockchain. Pre-1.0 — the API is stabilising but still moves between minor versions.

## Highlights

- Async clients for `algod` v2, `kmd` v1, and `indexer` v2
- One-call transaction builders for payments, asset config / transfer / freeze / clawback, application calls, key registration, and state proofs
- A typestate `AtomicGroupBuilder` — bundle transactions and ARC-4 ABI calls, then `simulate`, `sign`, and `execute`
- Typed contract clients generated at compile time from an ARC-4 ABI or a full ARC-56 app spec (`contract!`) — typed-struct args, a `deploy` constructor, state readers, and ARC-28 events
- An open, async `Signer` trait: `Account` out of the box, or plug in an HSM, remote KMS, or WalletConnect
- TEAL compile / disassemble + V3 source-map decoder
- The cross-SDK [`algorand-sdk-testing`](https://github.com/algorand/algorand-sdk-testing) acceptance suite runs in CI against a live node — 249 scenarios, 0 failures (see [Compatibility](#compatibility))

## Quickstart: an atomic group

Generate a typed client from an ARC-56 app spec with `contract!`, `deploy` it
straight from the spec, call a method with a typed-struct argument, dry-run the
group with `simulate`, then `sign` and `execute` the very same group — the
headline `algonaut` flow. Raw transactions (payments, asset ops) drop into the
same group via `add_transaction`. See
[`examples/contract_arc56.rs`](./examples/contract_arc56.rs) for the fully annotated
version (events, defaults, lifecycle actions, and more).

```rust
use algonaut::Algod;
use algonaut::atomic::AtomicGroupBuilder;
use algonaut::transaction::Signer;
use algonaut::transaction::account::Account;
use std::sync::Arc;
use std::{env, error::Error};

// `contract!` reads an ARC-56 "Extended App Description" at compile time and
// generates a typed `Vault` client: a `deploy` constructor that compiles the
// spec's TEAL, the named `Pair` struct as a typed argument, one builder per ABI
// method, and `global_*` readers that decode state per the spec's declared types.
algonaut::contract!("contracts/vault.arc56.json");

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let alice = Account::from_mnemonic(&env::var("ALICE_MNEMONIC")?)?;
    let sender = alice.address();

    // A signer is shared as `Arc<dyn Signer>`: `Account` here, but any HSM,
    // remote KMS, or WalletConnect impl drops in the same way.
    let signer: Arc<dyn Signer> = Arc::new(alice);

    let params = algod.suggested_params().await?;

    // Deploy straight from the spec: `deploy` compiles the contract's TEAL,
    // submits the app-create with the declared state schema, and hands back a
    // client bound to the new app id — no hand-managed `AppId`.
    let vault = Vault::deploy(&algod, sender, Arc::clone(&signer), &params).await?;
    println!("deployed Vault as app {}", vault.app_id().0);

    // `Pair` is the Rust struct generated for the ARC-56 `Pair`; `store` won't
    // compile unless the spec declares it and the field types line up.
    let store = vault.store(Pair { first: 2, second: 3 }).build(&params);
    let group = AtomicGroupBuilder::new().add_method_call(store).build()?;

    // `simulate` borrows the group, so we dry-run it before touching a key; then
    // `sign` (async — a wallet may await approval) and `execute` the same group.
    group.simulate(&algod).await?;
    let outcome = group.sign().await?.execute(&algod).await?;
    println!("confirmed in round {:?}", outcome.confirmed_round);

    // Global state, decoded per the key's declared ARC-56 type.
    println!("total = {:?}", vault.global_total(&algod).await?);
    Ok(())
}
```

## Compatibility

"Does this SDK work" has two separate answers, so there are two reference points.

**The cross-SDK acceptance suite.** [`algorand/algorand-sdk-testing`](https://github.com/algorand/algorand-sdk-testing)
is the Gherkin contract every Algorand SDK is measured against — the same
feature files that gate the Python, Java, Go and JavaScript SDKs. `algonaut`
runs them against a live algod + indexer + kmd harness, and that CI job is
**gating**: one red scenario fails the build.

**The TypeScript SDK.** [`algorand/js-algorand-sdk`](https://github.com/algorand/js-algorand-sdk)
v3.7.0 is the de-facto reference implementation, so the capability table below
is a straight comparison against it — gaps included.

### Acceptance-suite coverage

On a fresh harness: **249 scenarios passed, 0 failed** — 265 collected, 16
excluded for the reasons listed below. Reproduce with
`make harness && make cucumber`.

| Suite                        | Features live | Notes                                       |
| ---------------------------- | ------------- | ------------------------------------------- |
| Integration (live harness)   | **11 / 11**   | every upstream integration feature runs     |
| Unit (in-process)            | **5 / 16**    | 11 await step definitions, not SDK capability |

| Integration feature  | Status | Integration feature | Status |
| -------------------- | ------ | ------------------- | ------ |
| `abi`                | ✅     | `kmd`               | ✅     |
| `algod`              | ✅     | `rekey`             | ✅     |
| `applications`       | ✅     | `send`              | ✅     |
| `assets`             | ✅ ¹   | `simulate`          | ✅ ²   |
| `auction`            | ✅     | `compile`           | ✅     |
| `c2c`                | ✅     |                     |        |

| Unit feature              | Status | Unit feature               | Status |
| ------------------------- | ------ | -------------------------- | ------ |
| `offline`                 | ✅ ³   | `feetest`                  | ⏳     |
| `v2algodclient_paths`     | ✅ ⁴   | `program_sanity_check`     | ⏳     |
| `v2algodclient_responses` | ✅     | `rekey`                    | ⏳     |
| `v2indexerclient_paths`   | ✅ ⁴   | `responses`                | ⏳     |
| `v2indexerclient_responses` | ✅   | `sourcemap`                | ⏳     |
| `abijson`                 | ⏳     | `tealsign`                 | ❌ ⁵   |
| `algodclient_paths`       | ⏳     | `transactions`             | ⏳     |
| `atomic_transaction_composer` | ⏳ | `client-no-headers`        | ⏳     |

✅ runs in CI &nbsp;·&nbsp; ⏳ step definitions not yet written (the SDK capability exists) &nbsp;·&nbsp; ❌ blocked on a real capability gap

Every skip is a named entry in [`tests/cucumber/main.rs`](./tests/cucumber/main.rs)
with a written rationale — the runner enumerates all 27 upstream features so a
gap is visible rather than silently absent. Scenarios are **never** excluded to
make an assertion pass; each exclusion names a concrete capability or semantic
gap:

1. `assets` → *Asset reconfigure*: the scenario clears all four asset roles and
   expects the asset to survive, but a zero-valued `apar` with a `caid` is a
   *destroy* in Algorand, so it 404s.
2. `simulate` → *Simulating unsigned transactions in the ATC group*, plus the
   `exec_trace_with_stack_scratch` and `exec_trace_with_state_change_and_hash`
   tags: the group builder's `simulate` always sets `allow-empty-signatures`,
   so it cannot surface the `signedtxn has no sig` rejection the scenario asserts.
3. `offline`: only the address / mnemonic / microalgos round-trips are wired;
   the signing scenarios share step phrases with the integration features.
4. `v2algodclient_paths` → *Get Block, header-only*, and `v2indexerclient_paths`
   → *SearchAccounts path with OnlineOnly* / *SearchForTransactions path*: the
   generated clients don't expose those query parameters.
5. TEAL signing (`tealSign` / `verifyTealSign`) is not implemented — see the gap
   list below.

### Capability matrix vs. the TypeScript SDK

✅ supported &nbsp;·&nbsp; ⚠️ partial &nbsp;·&nbsp; ❌ not implemented

**Clients**

| Capability                        | `algonaut` 0.9 | `algosdk` 3.7 | Notes                                                              |
| --------------------------------- | :------------: | :-----------: | ------------------------------------------------------------------ |
| algod v2 REST                     | ✅             | ✅            | generated from the pinned `algod.oas3` spec (50 paths)             |
| indexer v2 REST                   | ✅             | ✅            | pinned `indexer.oas3` spec (21 paths)                              |
| kmd v1                            | ✅             | ✅            | hand-written ([ADR](docs/adr/kmd-client-hand-written.md))          |
| msgpack + JSON response decoding  | ✅             | ✅            |                                                                    |
| Wait for confirmation             | ✅             | ✅            | `PendingSubmission::confirm` long-polls `wait-for-block-after`     |
| Pluggable HTTP transport          | ❌             | ✅            | `algosdk` has `BaseHTTPClient`; `Algod::new` owns its `reqwest` client |

**Transactions**

| Capability                                   | `algonaut` 0.9 | `algosdk` 3.7 | Notes                                        |
| -------------------------------------------- | :------------: | :-----------: | -------------------------------------------- |
| Payment                                      | ✅             | ✅            |                                              |
| Key registration (online / offline / non-participating) | ✅  | ✅            | including `state_proof_key`                  |
| Asset create / reconfigure / destroy          | ✅             | ✅            |                                              |
| Asset transfer / opt-in / clawback            | ✅             | ✅            |                                              |
| Asset freeze                                  | ✅             | ✅            |                                              |
| Application call (create/update/delete/opt-in/close-out/clear-state/NoOp) | ✅ | ✅ |                          |
| Box references                                | ✅             | ✅            |                                              |
| State proof                                   | ✅             | ✅            |                                              |
| Heartbeat                                     | decode         | decode        | emitted by nodes; neither SDK builds them    |
| Rekey, lease, note, group                     | ✅             | ✅            |                                              |
| Resource access lists (`appAccess`)           | ❌             | ✅            | algod 5.0 surface, absent from our pinned spec |

**Signing**

| Capability                          | `algonaut` 0.9 | `algosdk` 3.7 | Notes                                                    |
| ----------------------------------- | :------------: | :-----------: | -------------------------------------------------------- |
| Ed25519 accounts, 25-word mnemonic  | ✅             | ✅            |                                                          |
| Sign / verify arbitrary bytes       | ✅             | ✅            | `Account::generate_sig` / `Address::verify_bytes`        |
| Multisig — create, append, merge    | ✅             | ✅            |                                                          |
| LogicSig — delegated + contract account | ✅         | ✅            |                                                          |
| Pluggable async signer              | ✅             | ✅            | `Signer` trait vs. `TransactionSigner`                   |
| TEAL sign (`tealSign`)              | ❌             | ✅            | gates the `tealsign` unit feature                        |
| Falcon-1024 / post-quantum accounts | ❌             | ✅            | algod 5.0; the largest open gap                          |

**Groups and ABI**

| Capability                                 | `algonaut` 0.9 | `algosdk` 3.7 | Notes                                                              |
| ------------------------------------------ | :------------: | :-----------: | ------------------------------------------------------------------ |
| Group ID computation / assignment          | ✅             | ✅            |                                                                    |
| Composer                                   | ✅             | ✅            | typestate `AtomicGroupBuilder` vs. `AtomicTransactionComposer`      |
| ARC-4 type system + method encoding        | ✅             | ✅            |                                                                    |
| ARC-4 method / interface / contract JSON   | ✅             | ✅            |                                                                    |
| ARC-28 events                              | ✅             | ✅            | decoded from transaction logs                                      |
| ARC-56 app spec → typed client             | ✅             | ❌            | `contract!` generates it at compile time; `algosdk` leaves ARC-56 to `algokit-utils-ts` |
| `ufixedNxM` and anonymous tuples in `contract!` | ⚠️        | ✅            | [#345](https://github.com/manuelmauro/algonaut/issues/345)         |

**Tooling**

| Capability                          | `algonaut` 0.9 | `algosdk` 3.7 | Notes                                              |
| ----------------------------------- | :------------: | :-----------: | -------------------------------------------------- |
| `simulate`                          | ✅             | ✅            |                                                    |
| TEAL compile / disassemble          | ✅             | ✅            |                                                    |
| TEAL source map (V3)                | ✅             | ✅            |                                                    |
| Block / ledger-state-delta endpoints | ✅            | ✅            |                                                    |
| `dryrun` + trace printer            | ✅             | ❌            | `algosdk` dropped its dryrun helpers in v3, and the acceptance suite dropped its dryrun features in 2026 — covered by unit tests only |
| ARC-26 payment URIs                 | ✅             | ❌            | `LinkableTransactionBuilder`                       |
| `wasm32` target                     | ✅             | n/a           | `cargo check --target wasm32-unknown-unknown` in CI |

### Known gaps

Stated plainly, because a matrix that only lists wins isn't a matrix:

- **algod 5.0 / post-quantum.** The pinned OpenAPI specs predate algod 5.0, so
  Falcon-1024 and post-quantum accounts, the PQ 25-word mnemonic, and resource
  access lists are all missing. This is the biggest gap and the one to close first.
- **TEAL signing.** `tealSign` / `tealSignFromProgram` / `verifyTealSign` have no
  `algonaut` equivalent.
- **`contract!` ABI coverage.** `ufixedNxM` and anonymous tuples are unsupported
  ([#345](https://github.com/manuelmauro/algonaut/issues/345)).
- **11 unit features** in the acceptance suite still need step definitions. The
  underlying SDK capability exists in every case except `tealsign`.
- **No pluggable HTTP transport**, so no drop-in retry or proxy layer.

### Reproducing all of this

```bash
make test                      # 227 unit + offline tests, no network
make harness && make cucumber  # the cross-SDK acceptance suite, live node
make test-e2e                  # ARC-56 deploy/call end-to-end against a node
make ci                        # fmt, clippy, tests, wasm32 check, build
```

CI runs all of the above on every push; `cargo-test-integration` (the live-node
acceptance suite) and `cargo-test-e2e` are both gating.

## What's new since 0.4.2

`algonaut` rested at `0.4.2` for years; the `0.5`–`0.8` line is a ground-up
modernization, and the example above is the API as it stands today:

- **0.5** — Rust 2024 edition (MSRV 1.85); `ring` swapped for `ed25519-dalek`, so `wasm32` builds need no C toolchain; workspace-wide dependency refresh; `lefthook` + `make ci`.
- **0.6** — `simulate` and dry-run request builders, a TEAL V3 source-map decoder, and domain types that serialize to both JSON and msgpack.
- **0.7** — identifier newtypes (`AppId`, `AssetId`, `TransactionId`) at the client boundary, block / account-resource / ledger-delta endpoints, and msgpack response decoding.
- **0.8** — an open, async `Signer` trait (HSM / remote KMS / WalletConnect friendly), the typestate `AtomicGroupBuilder` shown above, Cargo feature gates for clients (`algod`, `indexer`, `kmd`), and structured error types with full source-chaining.
- **0.9** — typed contract clients generated at compile time from a full **ARC-56** app spec: `contract!("app.arc56.json")` emits a `deploy` constructor (compiling the spec's TEAL or using precompiled `byteCode`), typed struct / tuple / array / reference / transaction method arguments, `global` / `local` / `box` / `map` state readers decoded per their declared ARC-56 types, ARC-28 event decoding, literal and sourced argument defaults, and a read-only `simulate` path — building on the ARC-4 `contract!` macro also introduced in this line. The earlier `abi_call!` / `abi_method!` macros are removed in its favour.

Each decision is recorded as an ADR under [`docs/adr/`](./docs/adr/); [CHANGELOG.md](./CHANGELOG.md) has the full entry-by-entry history.

## Workspace layout

| Crate                  | Purpose                                                                                  |
| ---------------------- | ---------------------------------------------------------------------------------------- |
| `algonaut`             | Top-level crate: the `atomic` group builder, `simulate`/`dryrun` helpers, and re-exports |
| `algonaut_algod`       | Generated client for the algod v2 REST API                                               |
| `algonaut_kmd`         | Client for the key-management daemon                                                     |
| `algonaut_indexer`     | Client for the indexer v2 REST API                                                       |
| `algonaut_core`        | Core types: `Address`, `MicroAlgos`, `Round`, `AppId`/`AssetId`/`TransactionId`, keys, multisig |
| `algonaut_crypto`      | Ed25519 sign/verify (via `ed25519-dalek`) and BIP-39 mnemonics                           |
| `algonaut_transaction` | Transaction builders and the open `Signer` trait                                         |
| `algonaut_abi`         | ARC-4 ABI types, method encoding, TEAL source-map decoder                                |
| `algonaut_abi_model`   | Pure serde data model for ARC-4 / ARC-56 app-spec JSON, shared by the runtime and the macros |
| `algonaut_abi_sig`     | ARC-4 signature/type grammar shared by the `contract!` macro and the runtime             |
| `algonaut_abi_macros`  | `contract!` proc-macro: typed ARC-4/ARC-56 contract client generator                     |
| `algonaut_encoding`    | Shared `serde` visitors and base32/base64 helpers                                        |
| `algonaut_model`       | Hand-written response models shared between the clients                                  |

## Running the examples

`/examples` has a wide set of runnable programs.

```bash
cp examples.env .env       # ALGOD_URL, KMD_URL, INDEXER_URL, mnemonics
cargo run --example quickstart
```

If you see `Error: NotPresent`, your environment variables aren't set — `cp examples.env .env` and edit as needed.

## Changelog

See [CHANGELOG.md](./CHANGELOG.md).

## Contributing

Read the [contribution guidelines](./CONTRIBUTING.md) before opening a PR. The pre-commit hook runs `make ci`; commit messages follow [Conventional Commits](https://www.conventionalcommits.org/).

## Acknowledgements

This crate is based on the work of [@mraof](https://github.com/mraof/rust-algorand-sdk).

## License

[![Ferris Algonaut](assets/ferris-algonaut.svg)](https://crates.io/crates/algonaut)

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](./LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

### Asset attribution

[Ferris Algonaut](assets/ferris-algonaut.svg) is licensed under a [Creative Commons Attribution 4.0 International License](https://creativecommons.org/licenses/by/4.0/).
[Rust `algonaut`'s logo](assets/rocket-solid.svg) is based on [Font Awesome](https://fontawesome.com/v5.15/icons/rocket)'s icon and licensed under a [Creative Commons Attribution 4.0 International License](https://creativecommons.org/licenses/by/4.0/).
