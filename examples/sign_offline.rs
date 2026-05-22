use algonaut::algod::v2::Algod;
use algonaut::core::{MicroAlgos, ToMsgPack};
use algonaut::transaction::Pay;
use algonaut::transaction::account::Account;
use dotenv::dotenv;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write;
#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    env_logger::init();

    info!("creating algod client");
    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    info!("creating account for alice");
    let alice = Account::from_mnemonic(&env::var("ALICE_MNEMONIC")?)?;

    info!("retrieving suggested params");
    let params = algod.suggested_params().await?;

    info!("building Pay transaction");
    let t = Pay::new(
        alice.address(),
        env::var("BOB_ADDRESS")?.parse()?,
        MicroAlgos(123_456),
    )
    .build(&params)?;

    info!("signing transaction");
    let signed_transaction = alice.sign(t)?;
    let bytes = signed_transaction.to_msg_pack()?;

    info!("saving transaction to file");
    let filename = "./signed.tx";
    let mut f = File::create(filename)?;
    f.write_all(&bytes)?;

    info!("saved signed transaction to file: {}", filename);

    Ok(())
}
