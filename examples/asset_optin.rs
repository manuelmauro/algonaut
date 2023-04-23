use algonaut::algod::v2::Algod;
use algonaut::transaction::AcceptAsset;
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

    info!("creating account for bob");
    let bob = Account::from_mnemonic(&env::var("BOB_MNEMONIC")?)?;
    info!("bob's address {:?}", bob.address());

    info!("retrieving suggested params");
    let params = algod.txn_params().await?;

    info!("building AcceptAsset transaction");
    let t = TxnBuilder::with(&params, AcceptAsset::new(bob.address(), 16).build()).build()?;

    info!("signing transaction");
    let sign_response = bob.sign_transaction(t)?;

    info!("broadcasting transaction");
    // Broadcast the transaction to the network
    // Note this transaction will get rejected because the accounts do not have any tokens
    let send_response = algod.send_txn(&sign_response).await;
    info!("response: {:?}", send_response);

    Ok(())
}
