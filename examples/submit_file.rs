use std::error::Error;
use std::fs::File;
use std::io::Read;

use algorand_rs::Algod;

// ideally these should be env variables
const ALGOD_URL: &str = "http://localhost:4001";
const ALGOD_TOKEN: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn main() -> Result<(), Box<dyn Error>> {
    let mut f = File::open("./signed.tx")?;
    let mut raw_transaction = Vec::new();
    let _ = f.read_to_end(&mut raw_transaction)?;

    let algod = Algod::new().bind(ALGOD_URL).auth(ALGOD_TOKEN).client()?;

    let send_response = algod.raw_transaction(&raw_transaction)?;
    println!("Transaction ID: {}", send_response.tx_id);

    Ok(())
}
