# Rust `algonaut`

[![Crate](https://img.shields.io/crates/v/algonaut.svg)](https://crates.io/crates/algonaut)
[![Docs](https://docs.rs/algonaut/badge.svg)](https://docs.rs/algonaut)
[![GitHub license](https://img.shields.io/github/license/Naereen/StrapDown.js.svg)](https://github.com/manuelmauro/algonaut/blob/main/LICENSE)
![Continuous integration](https://github.com/manuelmauro/algonaut/actions/workflows/quickstart.yml/badge.svg)

Rust **algonaut** aims at becoming a rusty SDK for [Algorand](https://www.algorand.com/). Please, be aware that this crate is a work in progress.

```rust
use algonaut::algod::AlgodBuilder;
use algonaut_core::MicroAlgos;
use algonaut_transaction::Pay;
use algonaut_transaction::{account::Account, TxnBuilder};
use std::error::Error;;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let algod = AlgodBuilder::new()
        .bind("http://localhost:4001")
        .auth("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        .build_v2()?;

    // an account with some funds
    let from_account = Account::from_mnemonic("fire enlist diesel stamp nuclear chunk student stumble call snow flock brush example slab guide choice option recall south kangaroo hundred matrix school above zero")?;

    let to_address = "2FMLYJHYQWRHMFKRHKTKX5UNB5DGO65U57O3YVLWUJWKRE4YYJYC2CWWBY".parse()?;

    // algod has a convenient method that retrieves basic information for a transaction
    let params = algod.suggested_transaction_params().await?;

    // we are ready to build the transaction
    let t = TxnBuilder::with(
        params,
        Pay::new(from_account.address(), to_address, MicroAlgos(123_456)).build(),
    )
    .build();

    // we need to sign the transaction to prove that we own the sender address
    let signed_t = from_account.sign_transaction(&t)?;

    // broadcast the transaction to the network
    let send_response = algod.broadcast_signed_transaction(&signed_t).await?;

    println!("Transaction ID: {}", send_response.tx_id);

    Ok(())
}
```

## Crates

- `algonaut_client` contains clients for `algod`, `kmd`, and `indexer` RPC APIs.
- `algonaut_core` defines core structures for Algorand like: `Address`, `Round`, `MicroAlgos`, etc.
- `algonaut_crypto` contains crypto utilities such as: `ed25519` and `mnemonics`.
- `algonaut_encoding` implements encoding utility functions such as `serde` visitors.
- `algonaut_transaction` support developers in building all kinds of Algorand transactions.

## Integration examples

- [My Algo signing with Yew / WASM](https://github.com/i-schuetz/algonaut-myalgo-yew-template)
- [Payment prompt with Yew / WASM](https://github.com/i-schuetz/algo-prompt)
- [Basic Yew / WASM](https://github.com/i-schuetz/algorand-yew-example)
- [iOS app](https://github.com/i-schuetz/algonaut_ios)

## Changelog

Read the [changelog](./CHANGELOG.md) for more details.

## Contribute

Do you want to help with the development? Please find out how by reading our [contributions guidelines](https://github.com/manuelmauro/algonaut/blob/main/CONTRIBUTING.md).

## Acknowledgements

This crate is based on the work of [@mraof](https://github.com/mraof/rust-algorand-sdk).

## License

[![Ferris Algonaut](assets/logo/rustacean-flat-algonaut.svg)](https://crates.io/crates/algonaut)

Licensed under MIT license.
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, shall be licensed as above, without any additional terms or conditions.

[Ferris Algonaut](assets/logo/rustacean-flat-algonaut.svg) is licensed under a [Creative Commons Attribution 4.0 International License](https://creativecommons.org/licenses/by/4.0/).
