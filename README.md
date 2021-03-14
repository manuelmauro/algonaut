# algorand-rs

[![Crate](https://meritbadge.herokuapp.com/algorand-rs)](https://crates.io/crates/algorand-rs)
[![Docs](https://docs.rs/paypal-rs/badge.svg)](https://docs.rs/algorand-rs)
[![GitHub license](https://img.shields.io/github/license/Naereen/StrapDown.js.svg)](https://github.com/manuelmauro/algorand-rs/blob/main/LICENSE)
![Continuous integration](https://github.com/manuelmauro/algorand-rs/actions/workflows/quickstart.yml/badge.svg)

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

- [ ] Example-driven API development
- [ ] Thorough test suite
- [ ] Clear error messages
- [ ] Async requests
- [ ] Comprehensive documentation
- [ ] Builder pattern and sensible defaults

## Changelog

Read the [changelog](./CHANGELOG.md) for more details.

## Contribute

Do you want to help with the development? Please find out how by reading our [contributions guidelines](https://github.com/manuelmauro/algorand-rs/blob/main/CONTRIBUTING.md).

## Acknowledgements

This crate is based on the work of [@mraof](https://github.com/mraof/rust-algorand-sdk).

## License

Licensed under MIT license.
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, shall be licensed as above, without any additional terms or conditions.
