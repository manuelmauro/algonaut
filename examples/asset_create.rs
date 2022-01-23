use algonaut::algod::v2::Algod;
use algonaut::error::AlgonautError;
use algonaut::model::algod::v2::PendingTransaction;
use algonaut::transaction::account::Account;
use algonaut::transaction::{CreateAsset, TxnBuilder};
use dotenv::dotenv;
use std::env;
use std::error::Error;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    // an account with some funds in our sandbox
    let creator = Account::from_mnemonic("fire enlist diesel stamp nuclear chunk student stumble call snow flock brush example slab guide choice option recall south kangaroo hundred matrix school above zero")?;
    println!("Creator: {:?}", creator.address());

    // algod has a convenient method that retrieves basic information for a transaction
    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let params = algod.suggested_transaction_params().await?;

    let t = TxnBuilder::with(
        params,
        CreateAsset::new(creator.address(), 10, 2, false)
            .unit_name("EIRI".to_owned())
            .asset_name("Naki".to_owned())
            .manager(creator.address())
            .reserve(creator.address())
            .freeze(creator.address())
            .clawback(creator.address())
            .url("example.com".to_owned())
            .build(),
    )
    .build();

    // we need to sign the transaction to prove that we own the sender address
    let signed_t = creator.sign_transaction(&t)?;

    // broadcast the transaction to the network
    let send_response = algod.broadcast_signed_transaction(&signed_t).await?;
    println!("Transaction ID: {}", send_response.tx_id);

    let pending_t = wait_for_pending_transaction(&algod, &send_response.tx_id).await?;
    println!("Asset index: {:?}", pending_t.map(|t| t.asset_index));

    Ok(())
}

/// Utility function to wait on a transaction to be confirmed
async fn wait_for_pending_transaction(
    algod: &Algod,
    txid: &str,
) -> Result<Option<PendingTransaction>, AlgonautError> {
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
