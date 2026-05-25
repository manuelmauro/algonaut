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
- Typed contract clients generated from an ARC-4 ABI JSON at compile time (`contract!`), or compile-time-checked one-off calls with `abi_call!`
- An open, async `Signer` trait: `Account` out of the box, or plug in an HSM, remote KMS, or WalletConnect
- TEAL compile / disassemble + V3 source-map decoder
- Cucumber acceptance suite that exercises the algorand-sdk-testing harness end-to-end

## Quickstart: an atomic group

Generate a typed client from an ARC-4 ABI with `contract!`, bundle two of its
method calls into one all-or-nothing group, dry-run it with `simulate`, then
`sign` and `execute` the very same group — the headline `algonaut` flow as it
stands today. Raw transactions (payments, asset ops) drop into the same group
via `add_transaction`. See
[`examples/contract_client.rs`](./examples/contract_client.rs) for the fully
annotated version.

```rust
use algonaut::Algod;
use algonaut::atomic::AtomicGroupBuilder;
use algonaut::core::AppId;
use algonaut::transaction::Signer;
use algonaut::transaction::account::Account;
use std::sync::Arc;
use std::{env, error::Error};

// `contract!` reads an ARC-4 ABI JSON at compile time and generates a typed
// `Calculator` client: one method per ABI entry, argument types checked by the
// compiler, plus `testnet()` / `mainnet()` constructors when the JSON carries a
// `networks` field.
algonaut::contract!("contracts/calculator.json");

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let alice = Account::from_mnemonic(&env::var("ALICE_MNEMONIC")?)?;

    // A signer is shared as `Arc<dyn Signer>`: `Account` here, but any HSM,
    // remote KMS, or WalletConnect impl drops in the same way.
    let signer: Arc<dyn Signer> = Arc::new(alice.clone());

    let params = algod.suggested_params().await?;

    // The generated client holds the app id, sender, and signer. Each method
    // returns a builder; `calculator.add(2, 3)` won't compile unless the ABI
    // declares `add(uint64,uint64)uint64` and the argument types line up.
    let calculator = Calculator::new(AppId(123), alice.address(), signer);
    let add = calculator.add(2u64, 3u64).build(&params);
    let subtract = calculator.subtract(10u64, 4u64).build(&params);

    // The typestate chain — AtomicGroupBuilder → UnsignedAtomicGroup →
    // SignedAtomicGroup — means "submit before sign" simply won't compile.
    let group = AtomicGroupBuilder::new()
        .add_method_call(add)
        .add_method_call(subtract)
        .build()?;

    // `simulate` borrows the group, so we dry-run it before touching a key.
    let sim = group.simulate(&algod).await?;
    println!("simulate ok: {} transaction(s)", sim.transaction_ids.len());

    // `sign` is async (a wallet may await user approval); `execute` submits,
    // waits for confirmation, and decodes each method call's ABI return.
    let outcome = group.sign().await?.execute(&algod).await?;
    println!("confirmed in round {:?}", outcome.confirmed_round);
    for result in &outcome.method_results {
        println!("returned: {:?}", result.return_value);
    }
    Ok(())
}
```

## What's new since 0.4.2

`algonaut` rested at `0.4.2` for years; the `0.5`–`0.8` line is a ground-up
modernization, and the example above is the API as it stands today:

- **0.5** — Rust 2024 edition (MSRV 1.85); `ring` swapped for `ed25519-dalek`, so `wasm32` builds need no C toolchain; workspace-wide dependency refresh; `lefthook` + `make ci`.
- **0.6** — `simulate` and dry-run request builders, a TEAL V3 source-map decoder, and domain types that serialize to both JSON and msgpack.
- **0.7** — identifier newtypes (`AppId`, `AssetId`, `TransactionId`) at the client boundary, block / account-resource / ledger-delta endpoints, and msgpack response decoding.
- **0.8** — an open, async `Signer` trait (HSM / remote KMS / WalletConnect friendly), the typestate `AtomicGroupBuilder` shown above, compile-time-checked ARC-4 calls via `abi_call!`, Cargo feature gates for clients (`algod`, `indexer`, `kmd`), and structured error types with full source-chaining.
- **unreleased** — typed contract clients generated from an ARC-4 ABI JSON at compile time with `contract!`.

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
| `algonaut_abi_model`   | Pure serde data model for ARC-4 ABI JSON, shared by the runtime and the macros           |
| `algonaut_abi_sig`     | ARC-4 signature/type grammar shared by the macros and the runtime                        |
| `algonaut_abi_macros`  | `contract!` client generator plus `abi_call!` / `abi_method!` compile-time-checked ABI proc-macros |
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
