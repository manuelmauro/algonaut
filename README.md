# algorand-rs
This crate is a work in progress and it aims at becoming a rusty algorand sdk.
It is based on the great work of [@mraof](https://github.com/mraof/rust-algorand-sdk).

```rust
use algorand_rs::kmd;
use algorand_rs::Algod;
use std::error::Error;

// ideally these should be env variables
const ALGOD_URL: &str = "http://localhost:4001";
const ALGOD_TOKEN: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn main() -> Result<(), Box<dyn Error>> {
    let algod = Algod::new().bind(ALGOD_URL)?.auth(ALGOD_TOKEN)?.client()?;
    
    println!("Algod versions: {:?}", algod.versions()?.versions);

    Ok(())
}
```