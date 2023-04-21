use algonaut::algod::v2::Algod;
use algonaut::indexer::v2::Indexer;
use algonaut::kmd::v1::Kmd;
use dotenv::dotenv;
use std::env;
use std::error::Error;
#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    env_logger::init();

    info!("creating algod client");
    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    info!("creating kmd client");
    let kmd = Kmd::new(&env::var("KMD_URL")?, &env::var("KMD_TOKEN")?)?;

    info!("creating indexer client");
    let indexer = Indexer::new(&env::var("INDEXER_URL")?)?;

    info!("algod versions: {:?}", algod.get_version().await?);
    info!("kmd versions: {:?}", kmd.versions().await?);
    info!("indexer health: {:?}", indexer.health().await);

    Ok(())
}
