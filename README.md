# Rust `algonaut`

[![Crate](https://meritbadge.herokuapp.com/algonaut)](https://crates.io/crates/algonaut)
[![Docs](https://docs.rs/paypal-rs/badge.svg)](https://docs.rs/algonaut)
[![GitHub license](https://img.shields.io/github/license/Naereen/StrapDown.js.svg)](https://github.com/manuelmauro/algonaut/blob/main/LICENSE)
![Continuous integration](https://github.com/manuelmauro/algonaut/actions/workflows/quickstart.yml/badge.svg)

Rust **algonaut** aims at becoming a rusty SDK for [Algorand](https://www.algorand.com/). Please, be aware that this crate is a work in progress.

```rust
use algonaut::core::MicroAlgos;
use algonaut::transaction::{Pay, Txn};
use algonaut::{Algod, Kmd};
use dotenv::dotenv;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    // kmd manages wallets and accounts
    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    // first we obtain a handle to our wallet
    let list_response = kmd.list_wallets()?;
    let wallet_id = match list_response
        .wallets
        .into_iter()
        .find(|wallet| wallet.name == "unencrypted-default-wallet")
    {
        Some(wallet) => wallet.id,
        None => return Err("Wallet not found".into()),
    };
    let init_response = kmd.init_wallet_handle(&wallet_id, "")?;
    let wallet_handle_token = init_response.wallet_handle_token;
    println!("Wallet Handle: {}", wallet_handle_token);

    // an account with some funds in our sandbox
    let from_address = env::var("ACCOUNT")?.parse()?;
    println!("Sender: {:#?}", from_address);

    let to_address = "2FMLYJHYQWRHMFKRHKTKX5UNB5DGO65U57O3YVLWUJWKRE4YYJYC2CWWBY".parse()?;
    println!("Receiver: {:#?}", to_address);

    // algod has a convenient method that retrieves basic information for a transaction
    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v1()?;

    let params = algod.transaction_params()?;

    // we are ready to build the transaction
    let t = Txn::new()
        .sender(from_address)
        .first_valid(params.last_round)
        .last_valid(params.last_round + 10)
        .genesis_id(params.genesis_id)
        .genesis_hash(params.genesis_hash)
        .fee(MicroAlgos(10_000))
        .payment(
            Pay::new()
                .amount(MicroAlgos(123_456))
                .to(to_address)
                .build(),
        )
        .build();

    // we need to sign the transaction to prove that we own the sender address
    let sign_response = kmd.sign_transaction(&wallet_handle_token, "", &t)?;

    // broadcast the transaction to the network
    let send_response = algod.raw_transaction(&sign_response.signed_transaction)?;

    println!("Transaction ID: {}", send_response.tx_id);

    Ok(())
}
```

## Objectives

- Example-driven API development
- Async requests
- Builder pattern and sensible defaults
- Modularity
- Clear error messages
- Thorough test suite
- Comprehensive documentation

## Crates

`algonaut` has a modular structure and is composed of multiple crates.

- `algonaut_client` contains clients for `algod`, `kmd`, and `indexer` RPC APIs.
- `algonaut_core` defines core structures for Algorand like: `Address`, `Round`, `MicroAlgos`, etc.
- `algonaut_crypto` contains crypto utilities such as: `ed25519` and `mnemonics`.
- `algonaut_encoding` implements encoding utility functions such as `serde` visitors.
- `algonaut_transaction` support developers in building all kinds of Algorand transactions.

Planned:

- `algonaut_teal` will add validators, templates, and dryrun helpers.

## Changelog

Read the [changelog](./CHANGELOG.md) for more details.

## Contribute

Do you want to help with the development? Please find out how by reading our [contributions guidelines](https://github.com/manuelmauro/algonaut/blob/main/CONTRIBUTING.md).

## Acknowledgements

This crate is based on the work of [@mraof](https://github.com/mraof/rust-algorand-sdk).

## License

Licensed under MIT license.
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, shall be licensed as above, without any additional terms or conditions.
