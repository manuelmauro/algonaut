use algonaut::algod::v2::Algod;
use algonaut::core::AssetId;
use algonaut::transaction::ClawbackAsset;
use algonaut::transaction::account::Account;
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
    // The account specified as clawback when creating the asset.
    let alice = Account::from_mnemonic(&env::var("ALICE_MNEMONIC")?)?;

    info!("creating account for bob");
    // The asset "sender": The account from which the asset is withdrawn.
    let bob = Account::from_mnemonic(&env::var("BOB_MNEMONIC")?)?;

    info!("retrieving suggested params");
    let params = algod.suggested_params().await?;

    info!("building ClawbackAsset transaction");
    let t = ClawbackAsset::new(
        alice.address(),
        AssetId(21),
        1,
        bob.address(),
        alice.address(),
    )
    .build(&params)?;

    info!("signing transaction");
    let signed_t = alice.sign(t)?;

    info!("broadcasting transaction");
    // Broadcast the transaction to the network
    // Note this transaction will get rejected because the accounts do not have any tokens
    let send_response = algod.send(&signed_t).await;
    info!("response: {:?}", send_response);

    Ok(())
}
