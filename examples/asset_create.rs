use algonaut::algod::v2::Algod;
use algonaut::error::ServiceError;
use algonaut::model::algod::v2::PendingTransaction;
use algonaut::transaction::account::Account;
use algonaut::transaction::{CreateAsset, TxnBuilder};
use dotenv::dotenv;
use std::env;
use std::error::Error;
use std::time::{Duration, Instant};
#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // load variables in .env
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
    let params = algod.suggested_transaction_params().await?;

    info!("building CreateAsset transaction");
    let t = TxnBuilder::with(
        &params,
        CreateAsset::new(alice.address(), 100, 2, false)
            .unit_name("EIRI".to_owned())
            .asset_name("Naki".to_owned())
            .manager(alice.address())
            .reserve(alice.address())
            .freeze(alice.address())
            .clawback(alice.address())
            .url("example.com".to_owned())
            .build(),
    )
    .build()?;

    info!("signing transaction");
    // we need to sign the transaction to prove that we own the sender address
    let signed_t = alice.sign_transaction(t)?;

    info!("broadcasting transaction");
    // broadcast the transaction to the network
    let send_response = algod.broadcast_signed_transaction(&signed_t).await?;
    info!("transaction ID: {}", send_response.tx_id);

    info!("waiting for transaction finality");
    let pending_t = wait_for_pending_transaction(&algod, &send_response.tx_id).await?;
    info!("asset index: {:?}", pending_t.map(|t| t.asset_index));

    Ok(())
}

/// Utility function to wait on a transaction to be confirmed
async fn wait_for_pending_transaction(
    algod: &Algod,
    txid: &str,
) -> Result<Option<PendingTransaction>, ServiceError> {
    let timeout = Duration::from_secs(10);
    let start = Instant::now();
    loop {
        let pending_transaction = algod.pending_transaction_with_id(txid).await?;
        // If the transaction has been confirmed or we time out, exit.
        if pending_transaction.confirmed_round.is_some() {
            return Ok(Some(pending_transaction));
        } else if start.elapsed() >= timeout {
            return Ok(None);
        }
        std::thread::sleep(Duration::from_millis(250))
    }
}
