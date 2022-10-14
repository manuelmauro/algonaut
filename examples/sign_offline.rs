use algonaut::algod::v2::Algod;
use algonaut::core::{MicroAlgos, ToMsgPack};
use algonaut::transaction::account::Account;
use algonaut::transaction::{Pay, TxnBuilder};
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
    let params = algod.suggested_transaction_params().await?;

    info!("building Pay transaction");
    let t = TxnBuilder::with(
        &params,
        Pay::new(
            alice.address(),
            (&env::var("BOB_ADDRESS")?).parse()?,
            MicroAlgos(123_456),
        )
        .build(),
    )
    .build()?;

    info!("signing transaction");
    // sign the transaction
    let signed_transaction = alice.sign_transaction(t)?;
    let bytes = signed_transaction.to_msg_pack()?;

    info!("saving transaction to file");
    let filename = "./signed.tx";
    let mut f = File::create(filename)?;
    f.write_all(&bytes)?;

    info!("saved signed transaction to file: {}", filename);

    Ok(())
}
