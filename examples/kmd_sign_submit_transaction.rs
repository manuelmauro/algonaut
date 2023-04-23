use algonaut::algod::v2::Algod;
use algonaut::core::MicroAlgos;
use algonaut::kmd::v1::Kmd;
use algonaut::transaction::{Pay, TxnBuilder};
use dotenv::dotenv;
use std::env;
use std::error::Error;
#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    env_logger::init();

    info!("creating kmd client");
    // kmd manages wallets and accounts
    let kmd = Kmd::new(&env::var("KMD_URL")?, &env::var("KMD_TOKEN")?)?;

    info!("listing wallets");
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

    info!("initializing handle to the wallet");
    let init_response = kmd.init_wallet_handle(&wallet_id, "").await?;
    let wallet_handle_token = init_response.wallet_handle_token;
    info!("wallet handle: {}", wallet_handle_token);

    info!("retrieving account for sender");
    // an account with some funds in our sandbox
    let sender = "OV4BQOSU7RQODSJ3VK4EXL3JOKZFO3IT3EKWVHHPQBEJOXEVNOJT545BLU"
        .parse()
        .expect("You need to specify an Algorand address from your kmd instance");
    println!("sender: {:?}", sender);

    info!("creating account for bob");
    let bob = env::var("BOB_ADDRESS")?.parse()?;
    println!("receiver: {:#?}", bob);

    info!("creating algod client");
    // algod has a convenient method that retrieves basic information for a transaction
    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    info!("retrieving suggested params");
    let params = algod.txn_params().await?;

    info!("building Pay transaction");
    let t =
        TxnBuilder::with(&params, Pay::new(sender, bob, MicroAlgos(123_456)).build()).build()?;

    info!("signing transaction");
    // we need to sign the transaction to prove that we own the sender address
    let sign_response = kmd.sign_transaction(&wallet_handle_token, "", &t).await?;

    info!("broadcasting transaction");
    // broadcast the transaction to the network
    let send_response = algod
        .send_raw_txn(&sign_response.signed_transaction)
        .await?;

    info!("transaction ID: {}", send_response.tx_id);

    Ok(())
}
