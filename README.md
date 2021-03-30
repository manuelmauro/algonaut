# algonaut

[![Crate](https://meritbadge.herokuapp.com/algonaut)](https://crates.io/crates/algonaut)
[![Docs](https://docs.rs/paypal-rs/badge.svg)](https://docs.rs/algonaut)
[![GitHub license](https://img.shields.io/github/license/Naereen/StrapDown.js.svg)](https://github.com/manuelmauro/algonaut/blob/main/LICENSE)
![Continuous integration](https://github.com/manuelmauro/algonaut/actions/workflows/quickstart.yml/badge.svg)

This crate is a WORK IN PROGRESS!

**algonaut** aims at becoming a rusty algorand sdk.

```rust
use algonaut::Algod;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let algod = Algod::new()
        .bind("http://localhost:4001")
        .auth("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        .client_v1()?;

    println!("Algod versions: {:?}", algod.versions()?);

    Ok(())
}
```

## Objectives

- [ ] Example-driven API development
- [ ] Async requests
- [ ] Builder pattern and sensible defaults
- [x] Modularity
- [ ] Clear error messages
- [ ] Thorough test suite
- [ ] Comprehensive documentation

## Crates

`algonaut` has a modular structure and is composed of multiple crates.

- `algonaut_client` contains clients for `algod`, `kmd`, and `indexer` RPC APIs.
- `algonaut_core` defines core structures for Algorand like: `Address`, `Round`, `MicroAlgos`, etc.
- `algonaut_crypto` implements crypto utilities such as: `ed25519` and `mnemonics`.
- `algonaut_encoding` implements encoding utility functions such as `serde` visitors.
- `algonaut_transaction` support developers in building all kinds of Algorand transactions.

Planned:

- `algonaut_teal` validators, templates, and dryrun helpers.

## Changelog

Read the [changelog](./CHANGELOG.md) for more details.

## Contribute

Do you want to help with the development? Please find out how by reading our [contributions guidelines](https://github.com/manuelmauro/algonaut/blob/main/CONTRIBUTING.md).

## Acknowledgements

This crate is based on the work of [@mraof](https://github.com/mraof/rust-algorand-sdk).

## License

Licensed under MIT license.
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, shall be licensed as above, without any additional terms or conditions.
