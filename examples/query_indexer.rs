use algonaut_client::indexer::v2::message::QueryAccount;
use algonaut_client::Indexer;
use dotenv::dotenv;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let indexer = Indexer::new()
        .bind(env::var("INDEXER_URL")?.as_ref())
        .client_v2()?;

    // query accounts using default query parameters (all None).
    let accounts = indexer.accounts(&QueryAccount::default()).await?.accounts;
    println!("found {} accounts", accounts.len());

    // query accounts with custom query parameters.
    let mut accounts_query = QueryAccount::default();
    // why 2? see: https://github.com/algorand/indexer/issues/516
    accounts_query.limit = Some(2);

    let accounts = indexer.accounts(&accounts_query).await?.accounts;
    println!("found {} accounts", accounts.len());

    Ok(())
}
