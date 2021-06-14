use algonaut_client::Algod;
use dotenv::dotenv;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let mut f = File::open("./signed.tx")?;
    let mut raw_transaction = Vec::new();
    let _ = f.read_to_end(&mut raw_transaction)?;

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v2()?;

    let send_response = algod.broadcast_raw_transaction(&raw_transaction).await?;
    println!("Transaction ID: {}", send_response.tx_id);

    Ok(())
}
