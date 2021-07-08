use algonaut::algod::AlgodBuilder;
use algonaut_core::{MicroAlgos, VotePk, VrfPk};
use algonaut_transaction::RegisterKey;
use algonaut_transaction::{account::Account, TxnBuilder};
use dotenv::dotenv;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = AlgodBuilder::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .build_v2()?;

    let account = Account::from_mnemonic("fire enlist diesel stamp nuclear chunk student stumble call snow flock brush example slab guide choice option recall south kangaroo hundred matrix school above zero")?;

    let vote_pk_str = "KgL5qW1jtHAQb1lQNIKuqHBqDWXRmb7GTmBN92a/sOQ=";
    let selection_pk_str = "A3s+2bgKlbG9qIaA4wJsrrJl8mVKGzTp/h6gGEyZmAg=";

    let params = algod.transaction_params().await?;

    let t = TxnBuilder::new()
        .sender(account.address())
        .first_valid(params.last_round)
        .last_valid(params.last_round + 10)
        .genesis_id(params.genesis_id)
        .genesis_hash(params.genesis_hash)
        .fee(MicroAlgos(100_000))
        .key_registration(
            RegisterKey::new()
                .vote_pk(VotePk::from_base64_str(vote_pk_str)?)
                .selection_pk(VrfPk::from_base64_str(selection_pk_str)?)
                .vote_first(params.last_round)
                .vote_last(params.last_round + 3_000_000)
                .vote_key_dilution(10_000)
                .build(),
        )
        .build();

    let sign_response = account.sign_transaction(&t);
    println!("{:#?}", sign_response);
    assert!(sign_response.is_ok());
    let sign_response = sign_response.unwrap();

    // Broadcast the transaction to the network
    // Note this transaction will get rejected because the accounts do not have any tokens
    let send_response = algod.broadcast_signed_transaction(&sign_response).await;

    println!("{:#?}", send_response);

    Ok(())
}
