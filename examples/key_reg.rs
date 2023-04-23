use algonaut::algod::v2::Algod;
use algonaut::core::{Round, VotePk, VrfPk};
use algonaut::transaction::RegisterKey;
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

    let vote_pk_str = "KgL5qW1jtHAQb1lQNIKuqHBqDWXRmb7GTmBN92a/sOQ=";
    let selection_pk_str = "A3s+2bgKlbG9qIaA4wJsrrJl8mVKGzTp/h6gGEyZmAg=";

    info!("retrieving suggested params");
    let params = algod.txn_params().await?;

    info!("building RegisterKey transaction");
    let t = TxnBuilder::with(
        &params,
        RegisterKey::online(
            alice.address(),
            VotePk::from_base64_str(vote_pk_str)?,
            VrfPk::from_base64_str(selection_pk_str)?,
            Round(params.last_round),
            Round(params.last_round + 3_000_000),
            10_000,
        )
        .build(),
    )
    .build()?;

    info!("signing transaction");
    let sign_response = alice.sign_transaction(t)?;

    info!("broadcasting transaction");
    // Broadcast the transaction to the network
    // Note this transaction will get rejected because the accounts do not have any tokens
    let send_response = algod.send_txn(&sign_response).await;
    info!("{:?}", send_response);

    Ok(())
}
