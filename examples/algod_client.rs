use algorust::Algod;
use std::error::Error;

// ideally these should be env variables
const ALGOD_URL: &str = "http://localhost:4001";
const ALGOD_TOKEN: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn main() -> Result<(), Box<dyn Error>> {
    let algod = Algod::new().bind(ALGOD_URL)?.auth(ALGOD_TOKEN)?.client()?;

    // print algod status
    let node_status = algod.status()?;
    println!("algod last round: {}", node_status.last_round);
    println!(
        "algod time since last round: {}",
        node_status.time_since_last_round
    );
    println!("algod catchup: {}", node_status.catchup_time);
    println!("algod latest version: {}", node_status.last_version);

    // fetch block information
    let last_block = algod.block(node_status.last_round)?;
    println!("{:#?}", last_block);

    Ok(())
}
