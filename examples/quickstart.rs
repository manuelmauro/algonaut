use algonaut::algod::AlgodBuilder;
use algonaut::indexer::IndexerBuilder;
use algonaut::kmd::KmdBuilder;
use dotenv::dotenv;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = AlgodBuilder::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .build_v2()?;
    let kmd = KmdBuilder::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .build_v1()?;
    let indexer = IndexerBuilder::new()
        .bind(env::var("INDEXER_URL")?.as_ref())
        .build_v2()?;

    println!("Algod versions: {:#?}", algod.versions().await?);
    println!("Kmd versions: {:#?}", kmd.versions().await?);
    println!("Indexer health: {:#?}", indexer.health().await);

    Ok(())
}
