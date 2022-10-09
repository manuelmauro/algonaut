use algonaut::algod::v2::Algod;
use algonaut::core::Round;
use dotenv::dotenv;
use std::env;
use std::error::Error;
#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();
    env_logger::init();

    info!("creating algod client");
    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    // print algod status
    info!("retrieving node status");
    let node_status = algod.status().await?;
    info!("algod last round: {}", node_status.last_round);
    info!(
        "algod time since last round: {}",
        node_status.time_since_last_round
    );
    info!("algod catchup: {}", node_status.catchup_time);
    info!("algod latest version: {}", node_status.last_version);

    // fetch block information
    let last_block = algod.block(Round(node_status.last_round)).await?;
    info!("{:?}", last_block);

    Ok(())
}
