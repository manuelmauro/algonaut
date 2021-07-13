use algonaut::algod::AlgodBuilder;
use algonaut::core::MicroAlgos;
use algonaut::kmd::KmdBuilder;
use algonaut::transaction::{Pay, TxnBuilder};
use dotenv::dotenv;
use std::env;
use std::error::Error;

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
    let from_address = env::var("ACCOUNT")?.parse()?;
    println!("Sender: {:#?}", from_address);

    let to_address = "2FMLYJHYQWRHMFKRHKTKX5UNB5DGO65U57O3YVLWUJWKRE4YYJYC2CWWBY".parse()?;
    println!("Receiver: {:#?}", to_address);

    // algod has a convenient method that retrieves basic information for a transaction
    let algod = AlgodBuilder::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .build_v2()?;

    let params = algod.suggested_transaction_params().await?;

    let t = TxnBuilder::with(
        params,
        Pay::new(from_address, to_address, MicroAlgos(123_456)).build(),
    )
    .build();

    // we need to sign the transaction to prove that we own the sender address
    let sign_response = kmd.sign_transaction(&wallet_handle_token, "", &t).await?;

    // broadcast the transaction to the network
    let send_response = algod
        .broadcast_raw_transaction(&sign_response.signed_transaction)
        .await?;

    println!("Transaction ID: {}", send_response.tx_id);

    Ok(())
}
