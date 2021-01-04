use std::error::Error;
use std::fs::File;
use std::io::Read;

use algorust::AlgodClient;

fn main() -> Result<(), Box<dyn Error>> {
    let algod_address = "http://localhost:4001";
    let algod_token = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    let mut f = File::open("./signed.tx")?;
    let mut raw_transaction = Vec::new();
    let _ = f.read_to_end(&mut raw_transaction)?;

    let algod_client = AlgodClient::new(algod_address, algod_token);

    let send_response = algod_client.raw_transaction(&raw_transaction)?;
    println!("Transaction ID: {}", send_response.tx_id);

    Ok(())
}
