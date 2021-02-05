use algorand_rs::Algod;
use algorand_rs::Kmd;
use std::error::Error;

// ideally these should be env variables
const ALGOD_URL: &str = "http://localhost:4001";
const ALGOD_TOKEN: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const KMD_URL: &str = "http://localhost:4002";
const KMD_TOKEN: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn main() -> Result<(), Box<dyn Error>> {
    let algod = Algod::new().bind(ALGOD_URL).auth(ALGOD_TOKEN).client_v1()?;
    let kmd = Kmd::new().bind(KMD_URL).auth(KMD_TOKEN).client_v1()?;

    println!("Algod versions: {:#?}", algod.versions()?);
    println!("Kmd versions: {:#?}", kmd.versions()?);

    Ok(())
}
