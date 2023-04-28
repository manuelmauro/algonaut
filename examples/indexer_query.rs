use algonaut::indexer::v2::Indexer;
use dotenv::dotenv;
use std::env;
use std::error::Error;
#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    env_logger::init();

    info!("creating indexer client");
    let indexer = Indexer::new(&env::var("INDEXER_URL")?, &env::var("INDEXER_TOKEN")?)?;

    info!("querying accounts with default query parameters");
    // query accounts using default query parameters (all None).
    let accounts = indexer
        .search_for_accounts(None, None, None, None, None, None, None, None, None, None)
        .await?
        .accounts;
    info!("found {} accounts", accounts.len());

    // why 2? see: https://github.com/algorand/indexer/issues/516
    info!("querying accounts with limit=2");
    let accounts = indexer
        .search_for_accounts(
            None,
            Some(2),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await?
        .accounts;
    info!("found {} accounts", accounts.len());

    Ok(())
}
