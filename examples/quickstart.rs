use algorand_rs::kmd;
use algorand_rs::Algod;
use std::error::Error;

// ideally these should be env variables
const ALGOD_URL: &str = "http://localhost:4001";
const ALGOD_TOKEN: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const KMD_URL: &str = "http://localhost:4002";
const KMD_TOKEN: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn main() -> Result<(), Box<dyn Error>> {
    let algod = Algod::new().bind(ALGOD_URL)?.auth(ALGOD_TOKEN)?.client()?;
    let kmd = kmd::Client::new(KMD_URL, KMD_TOKEN);

    println!("Algod versions: {:?}", algod.versions()?.versions);
    println!("Kmd versions: {:?}", kmd.versions()?.versions);

    Ok(())
}
