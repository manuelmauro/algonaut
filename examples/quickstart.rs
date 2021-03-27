use algonaut::{Algod, Indexer, Kmd};
use dotenv::dotenv;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v1()?;
    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;
    let indexer = Indexer::new()
        .bind(env::var("INDEXER_URL")?.as_ref())
        .client_v2()?;

    println!("Algod versions: {:#?}", algod.versions()?);
    println!("Kmd versions: {:#?}", kmd.versions()?);
    println!("Indexer health: {:#?}", indexer.health());

    Ok(())
}
