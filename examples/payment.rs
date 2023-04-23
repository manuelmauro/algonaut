use algonaut::algod::v2::Algod;
use algonaut::core::MicroAlgos;
use algonaut::transaction::Pay;
use algonaut::transaction::{account::Account, TxnBuilder};
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

    info!("creating account for alice");
    let alice = Account::from_mnemonic(&env::var("ALICE_MNEMONIC")?)?;

    info!("creating account for bob");
    let bob = Account::from_mnemonic(&env::var("BOB_MNEMONIC")?)?;

    info!("retrieving suggested params");
    let params = algod.txn_params().await?;

    info!("building Pay transaction");
    let t = TxnBuilder::with(
        &params,
        Pay::new(alice.address(), bob.address(), MicroAlgos(123_456)).build(),
    )
    .build()?;

    info!("signing transaction");
    let sign_response = alice.sign_transaction(t)?;

    info!("broadcasting transaction");
    let send_response = algod.send_txn(&sign_response).await;
    info!("response: {:?}", send_response);

    Ok(())
}
