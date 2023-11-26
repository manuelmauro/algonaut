use algonaut::algod::v2::Algod;
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

    let last_round = algod.status().await.unwrap().last_round;
    let _ = algod.status_after_block(last_round).await;
    let last_block = algod.block(last_round).await.unwrap();

    info!("last block: {:#?}", last_block);

    Ok(())
}
