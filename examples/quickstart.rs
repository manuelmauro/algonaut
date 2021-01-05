use algorust::Algod;
use algorust::kmd;
use std::error::Error;

// ideally these should be env variables
const ALGOD_URL: &str = "http://localhost:4001";
const ALGOD_TOKEN: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn main() -> Result<(), Box<dyn Error>> {
    let kmd_address = "http://localhost:4002";
    let kmd_token = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    let algod = Algod::new().bind(ALGOD_URL)?.auth(ALGOD_TOKEN)?.client()?;
    let kmd_client = kmd::Client::new(kmd_address, kmd_token);

    println!("Algod versions: {:?}", algod.versions().unwrap().versions);
    println!(
        "Kmd versions: {:?}",
        kmd_client.versions().unwrap().versions
    );

    Ok(())
}
