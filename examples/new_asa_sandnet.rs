use algonaut::algod::v2::Algod;
use algonaut::algod::AlgodBuilder;
use algonaut::core::MicroAlgos;
use algonaut::kmd::KmdBuilder;
use algonaut::error::AlgonautError;
use algonaut::transaction::{ConfigureAsset, TxnBuilder};
use algonaut_client::algod::v2::message::PendingTransaction;
use dotenv::dotenv;
use std::env;
use std::error::Error;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    // kmd manages wallets and accounts
    let kmd = KmdBuilder::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .build_v1()?;

    // first we obtain a handle to our wallet
    let list_response = kmd.list_wallets().await?;
    let wallet_id = match list_response
        .wallets
        .into_iter()
        .find(|wallet| wallet.name == "unencrypted-default-wallet")
    {
        Some(wallet) => wallet.id,
        None => return Err("Wallet not found".into()),
    };
    let init_response = kmd.init_wallet_handle(&wallet_id, "").await?;
    let wallet_handle_token = init_response.wallet_handle_token;
    println!("Wallet Handle: {}", wallet_handle_token);

    // an account with some funds in our sandbox
    let creator = env::var("ACCOUNT")?.parse()?;
    println!("Creator: {:?}", creator);

    // algod has a convenient method that retrieves basic information for a transaction
    let algod = AlgodBuilder::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .build_v2()?;

    let params = algod.transaction_params().await?;
    println!("Last round: {}", params.last_round);

    // we are ready to build the transaction
    let t = TxnBuilder::new()
        .sender(creator)
        .first_valid(params.last_round)
        .last_valid(params.last_round + 1000)
        .genesis_id(params.genesis_id)
        .genesis_hash(params.genesis_hash)
        .fee(MicroAlgos(100_000))
        .asset_configuration(
            ConfigureAsset::new()
                .total(10)
                .default_frozen(false)
                .unit_name("EIRI".to_owned())
                .asset_name("Naki".to_owned())
                .manager(creator)
                .reserve(creator)
                .freeze(creator)
                .clawback(creator)
                .url("example.com".to_owned())
                .decimals(2)
                .build(),
        )
        .build();

    // we need to sign the transaction to prove that we own the sender address
    let sign_response = kmd.sign_transaction(&wallet_handle_token, "", &t).await?;

    // broadcast the transaction to the network
    let send_response = algod
        .broadcast_raw_transaction(&sign_response.signed_transaction)
        .await?;

    println!("Transaction ID: {}", send_response.tx_id);

    let pending_t = wait_for_pending_transaction(&algod, &send_response.tx_id).await?;
    assert!(pending_t.is_some());
    println!("Asset index: {:?}", pending_t.unwrap().asset_index);

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
