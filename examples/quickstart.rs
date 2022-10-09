use algonaut::algod::v2::Algod;
use algonaut::indexer::v2::Indexer;
use algonaut::kmd::v1::Kmd;
use dotenv::dotenv;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;
    let kmd = Kmd::new(&env::var("KMD_URL")?, &env::var("KMD_TOKEN")?)?;
    let indexer = Indexer::new(&env::var("INDEXER_URL")?)?;

    println!("Algod versions: {:#?}", algod.versions().await?);
    println!("Kmd versions: {:#?}", kmd.versions().await?);
    println!("Indexer health: {:#?}", indexer.health().await);

    Ok(())
}
