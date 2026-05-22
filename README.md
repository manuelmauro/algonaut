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
- Transaction builders for payments, asset config / transfer / freeze / clawback, application calls, key registration, and state proofs
- ARC-4 ABI types, method invocation, and an `AtomicTransactionComposer` with `simulate` + `execute`
- TEAL compile / disassemble + V3 source-map decoder
- Cucumber acceptance suite that exercises the algorand-sdk-testing harness end-to-end

## Quickstart

```rust
use algonaut::algod::v2::Algod;
use algonaut_core::MicroAlgos;
use algonaut_transaction::{Pay, TxnBuilder, account::Account};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let algod = Algod::new(
        "http://localhost:4001",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    )?;

    let alice = Account::from_mnemonic(
        "fire enlist diesel stamp nuclear chunk student stumble call snow flock brush \
         example slab guide choice option recall south kangaroo hundred matrix school \
         above zero",
    )?;
    let bob = "2FMLYJHYQWRHMFKRHKTKX5UNB5DGO65U57O3YVLWUJWKRE4YYJYC2CWWBY".parse()?;

    let params = algod.suggested_params().await?;
    let tx = TxnBuilder::with(
        &params,
        Pay::new(alice.address(), bob, MicroAlgos(123_456)).build(),
    )
    .build()?;

    let signed = alice.sign(tx)?;
    let resp = algod.send(&signed).await?;
    println!("submitted: {}", resp.tx_id);
    Ok(())
}
```

## Workspace layout

| Crate                  | Purpose                                                        |
| ---------------------- | -------------------------------------------------------------- |
| `algonaut`             | Top-level convenience crate; re-exports the rest               |
| `algonaut_algod`       | Generated client for the algod v2 REST API                     |
| `algonaut_kmd`         | Client for the key-management daemon                           |
| `algonaut_indexer`     | Client for the indexer v2 REST API                             |
| `algonaut_core`        | Core types: `Address`, `MicroAlgos`, `Round`, keys, multisig   |
| `algonaut_crypto`      | Ed25519 sign/verify (via `ed25519-dalek`) and BIP-39 mnemonics |
| `algonaut_transaction` | Transaction builders and the `AtomicTransactionComposer`       |
| `algonaut_abi`         | ARC-4 ABI types, method encoding, TEAL source-map decoder      |
| `algonaut_encoding`    | Shared `serde` visitors and base32/base64 helpers              |
| `algonaut_model`       | Hand-written response models shared between the clients        |

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
