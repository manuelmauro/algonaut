use algonaut::algod::v2::Algod;
use algonaut_core::{VotePk, VrfPk};
use algonaut_transaction::RegisterKey;
use algonaut_transaction::{account::Account, TxnBuilder};
use dotenv::dotenv;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let account = Account::from_mnemonic("fire enlist diesel stamp nuclear chunk student stumble call snow flock brush example slab guide choice option recall south kangaroo hundred matrix school above zero")?;

    let vote_pk_str = "KgL5qW1jtHAQb1lQNIKuqHBqDWXRmb7GTmBN92a/sOQ=";
    let selection_pk_str = "A3s+2bgKlbG9qIaA4wJsrrJl8mVKGzTp/h6gGEyZmAg=";

    let params = algod.suggested_transaction_params().await?;

    let t = TxnBuilder::with(
        params.clone(),
        RegisterKey::online(
            account.address(),
            VotePk::from_base64_str(vote_pk_str)?,
            VrfPk::from_base64_str(selection_pk_str)?,
            params.first_valid,
            params.first_valid + 3_000_000,
            10_000,
        )
        .build(),
    )
    .build();

    let sign_response = account.sign_transaction(&t)?;

    // Broadcast the transaction to the network
    // Note this transaction will get rejected because the accounts do not have any tokens
    let send_response = algod.broadcast_signed_transaction(&sign_response).await;
    println!("{:#?}", send_response);

    Ok(())
}
