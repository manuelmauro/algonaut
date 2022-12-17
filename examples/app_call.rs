use algonaut::algod::v2::Algod;
use algonaut::transaction::account::Account;
use algonaut::transaction::builder::CallApplication;
use algonaut::transaction::TxnBuilder;
use dotenv::dotenv;
use std::env;
use std::error::Error;
#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    env_logger::init();

    let url = String::from("https://node.testnet.algoexplorerapi.io");
    let token = String::from(" ");
    
    info!("creating algod client");
    let algod = Algod::new(&url, &token)?;

    info!("creating account for alice");
    let alice = Account::from_mnemonic(&alice_mnemonic)?;

    info!("retrieving suggested params");
    let params = algod.suggested_transaction_params().await?;

    info!("building transaction");
    let t = TxnBuilder::with(
        &params,
        CallApplication::new(alice.address(), 3)
            .app_arguments(vec![vec![1, 0], vec![255]])
            .build(),
    )
    .build()?;

    info!("signing transaction");
    let signed_t = alice.sign_transaction(t)?;

    info!("broadcasting transaction");
    let send_response = algod.broadcast_signed_transaction(&signed_t).await?;
    info!("response: {:?}", send_response);

    Ok(())
}
