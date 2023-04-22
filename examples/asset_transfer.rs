use algonaut::algod::v2::Algod;
use algonaut::transaction::TransferAsset;
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

    info!("creating accounts for alice and bob");
    let alice = Account::from_mnemonic(&env::var("ALICE_MNEMONIC")?)?;
    let bob = Account::from_mnemonic(&env::var("BOB_MNEMONIC")?)?;

    info!("retrieving suggested params");
    let params = algod.transaction_params().await?;

    info!("building TransferAsset transaction");
    let t = TxnBuilder::with(
        &params,
        TransferAsset::new(alice.address(), 16, 1, bob.address()).build(),
    )
    .build()?;

    info!("signing transaction");
    let sign_response = alice.sign_transaction(t)?;

    info!("broadcasting transaction");
    // Broadcast the transaction to the network
    // Note this transaction will get rejected because the accounts do not have any tokens
    let send_response = algod.signed_transaction(&sign_response).await;
    info!("response: {:?}", send_response);

    Ok(())
}
