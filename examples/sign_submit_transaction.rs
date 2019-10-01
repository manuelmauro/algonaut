use std::error::Error;

use algosdk::transaction::Transaction;
use algosdk::{Address, AlgodClient, KmdClient, MicroAlgos};

fn main() -> Result<(), Box<dyn Error>> {
    let kmd_address = "http://localhost:7833";
    let kmd_token = "contents-of-kmd.token";
    let algod_address = "http://localhost:8080";
    let algod_token = "contents-of-algod.token";

    let kmd_client = KmdClient::new(kmd_address, kmd_token);
    let algod_client = AlgodClient::new(algod_address, algod_token);

    let list_response = kmd_client.list_wallets()?;

    let wallet_id = match list_response
        .wallets
        .into_iter()
        .find(|wallet| wallet.name == "testwallet")
    {
        Some(wallet) => wallet.id,
        None => return Err("Wallet not found".into()),
    };

    let init_response = kmd_client.init_wallet_handle(&wallet_id, "testpassword")?;
    let wallet_handle_token = init_response.wallet_handle_token;

    let gen_response = kmd_client.generate_key(&wallet_handle_token)?;
    let from_address = Address::from_string(&gen_response.address)?;

    let gen_response = kmd_client.generate_key(&wallet_handle_token)?;
    let to_address = Address::from_string(&gen_response.address)?;

    let transaction_params = algod_client.transaction_params()?;

    let genesis_id = transaction_params.genesis_id;
    let genesis_hash = transaction_params.genesis_hash;

    let transaction = Transaction::new_payment(
        from_address,
        MicroAlgos(1000),
        transaction_params.last_round,
        transaction_params.last_round + 1000,
        Vec::new(),
        &genesis_id,
        genesis_hash,
        to_address,
        MicroAlgos(200_000),
        None,
    )?;

    let sign_response =
        kmd_client.sign_transaction(&wallet_handle_token, "testpassword", &transaction)?;

    println!(
        "kmd made signed transaction with {} bytes",
        sign_response.signed_transaction.len()
    );

    // Broadcast the transaction to the network
    // Note this transaction will get rejected because the accounts do not have any tokens
    let send_response = algod_client.raw_transaction(&sign_response.signed_transaction)?;

    println!("Transaction ID: {}", send_response.tx_id);

    Ok(())
}
