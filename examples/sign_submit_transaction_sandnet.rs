use algonaut::core::{Address, MicroAlgos};
use algonaut::transaction::{BaseTransaction, Payment, Transaction, TransactionType};
use algonaut::{Algod, Kmd};
use dotenv::dotenv;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let list_response = kmd.list_wallets()?;

    let wallet_id = match list_response
        .wallets
        .into_iter()
        .find(|wallet| wallet.name == "unencrypted-default-wallet")
    {
        Some(wallet) => wallet.id,
        None => return Err("Wallet not found".into()),
    };
    println!("Wallet: {}", wallet_id);

    let init_response = kmd.init_wallet_handle(&wallet_id, "")?;
    let wallet_handle_token = init_response.wallet_handle_token;

    let from_address =
        Address::from_string("2FMLYJHYQWRHMFKRHKTKX5UNB5DGO65U57O3YVLWUJWKRE4YYJYC2CWWBY")?;
    println!("Sender: {:#?}", from_address);

    let to_address =
        Address::from_string("2FMLYJHYQWRHMFKRHKTKX5UNB5DGO65U57O3YVLWUJWKRE4YYJYC2CWWBY")?;
    println!("Receiver: {:#?}", to_address);

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v1()?;

    let transaction_params = algod.transaction_params()?;
    let genesis_id = transaction_params.genesis_id;
    let genesis_hash = transaction_params.genesis_hash;

    let base = BaseTransaction {
        sender: from_address,
        first_valid: transaction_params.last_round,
        last_valid: transaction_params.last_round + 1000,
        note: Vec::new(),
        genesis_id,
        genesis_hash,
    };
    println!("Base: {:#?}", base);

    let payment = Payment {
        amount: MicroAlgos(100_000),
        receiver: to_address,
        close_remainder_to: None,
    };
    println!("Payment: {:#?}", payment);

    let transaction =
        Transaction::new_flat_fee(base, MicroAlgos(10_000), TransactionType::Payment(payment));
    println!("Transaction: {:#?}", transaction);

    let sign_response = kmd.sign_transaction(&wallet_handle_token, "", &transaction)?;
    println!("Signed: {:#?}", sign_response);

    // Broadcast the transaction to the network
    let send_response = algod.raw_transaction(&sign_response.signed_transaction)?;

    println!("Transaction ID: {}", send_response.tx_id);

    Ok(())
}
