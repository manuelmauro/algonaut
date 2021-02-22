# algorand-rs

[![Crate](https://meritbadge.herokuapp.com/algorand-rs)](https://crates.io/crates/algorand-rs)
[![Docs](https://docs.rs/paypal-rs/badge.svg)](https://docs.rs/algorand-rs)
[![GitHub license](https://img.shields.io/github/license/Naereen/StrapDown.js.svg)](https://github.com/manuelmauro/algorand-rs/blob/main/LICENSE)

This crate is a WORK IN PROGRESS!

**algorand-rs** aims at becoming a rusty algorand sdk.

```rust
use algorand_rs::algod::Algod;

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

- [ ] Examples guiding API development
- [ ] Builder pattern and sensible defaults
- [ ] Async requests
- [ ] Clear error messages
- [ ] Thorough test suite
- [ ] Comprehensive documentation

## Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

### [Unreleased]

### [0.1.1] - 2021-02-19

#### Added

- Add some sandbox integration tests
- Use ```dotenv``` for address and token env variable pointing at the sandbox
- Add client builder for: algod, kmd, and indexer
- Use ```thiserror``` crate for error management
- Add algorand's indexer client (incomplete)
- Add algod v2 client (incomplete)
- Add ```reqwest``` http client to clients' structs
- Forked [rust-algorand-sdk](https://github.com/mraof/rust-algorand-sdk)

#### Changed

- Change modules structure

#### Removed

- Remove cucumber test suite

[unreleased]: https://github.com/manuelmauro/algorand-rs/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/manuelmauro/algorand-rs/releases/tag/v0.1.1

## Acknowledgements

This crate is based on the work of [@mraof](https://github.com/mraof/rust-algorand-sdk).

### Contributors

A great thanks goes to:

## License

Licensed under MIT license.
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, shall be licensed as above, without any additional terms or conditions.
