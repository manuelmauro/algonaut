# algorand-rs

This crate is a work in progress and it aims at becoming a rusty algorand sdk.

```rust
use algorand_rs::kmd;
use algorand_rs::Algod;
use std::error::Error;

// ideally these should be env variables
const ALGOD_URL: &str = "http://localhost:4001";
const ALGOD_TOKEN: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn main() -> Result<(), Box<dyn Error>> {
    let algod = Algod::new().bind(ALGOD_URL).auth(ALGOD_TOKEN).client_v1()?;

    println!("Algod versions: {:?}", algod.versions()?.versions);

    Ok(())
}
```

## Objectives

- [ ] Clear error messages
- [ ] Async requests
- [ ] Thorough test suite
- [ ] Proper documentation
- [ ] Examples guiding API development

## Acknowledgements

This crate is based on the great work of [@mraof](https://github.com/mraof/rust-algorand-sdk) and partly on [@KBryan](https://github.com/KBryan/algorand_rust_sdk)'s fork.

## License

Licensed under MIT license.
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, shall be licensed as above, without any additional terms or conditions.
