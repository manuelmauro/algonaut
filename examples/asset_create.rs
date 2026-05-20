use algonaut::algod::v2::Algod;
use algonaut::transaction::CreateAsset;
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
    // algod has a convenient method that retrieves basic information for a transaction
    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    info!("creating account for alice");
    // an account with some funds in our sandbox
    let alice = Account::from_mnemonic(&env::var("ALICE_MNEMONIC")?)?;
    info!("creator: {:?}", alice.address());

    info!("retrieving suggested params");
    let params = algod.txn_params().await?;

    info!("building CreateAsset transaction");
    let t = CreateAsset::new(alice.address(), 100, 2, false)
        .unit_name("EIRI".to_owned())
        .asset_name("Naki".to_owned())
        .manager(alice.address())
        .reserve(alice.address())
        .freeze(alice.address())
        .clawback(alice.address())
        .url("example.com".to_owned())
        .build(&params)?;

    info!("signing transaction");
    // we need to sign the transaction to prove that we own the sender address
    let signed_t = alice.sign_transaction(t)?;

    info!("broadcasting transaction");
    // broadcast the transaction to the network and wait for confirmation
    let pending = algod.submit(&signed_t).await?;
    info!("transaction ID: {}", pending.tx_id());

    info!("waiting for transaction finality");
    let pending_t = pending.confirm().await?;
    info!("asset index: {:?}", pending_t.asset_index);

    Ok(())
}
