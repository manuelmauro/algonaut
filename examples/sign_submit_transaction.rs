use algorand_rs::transaction::{BaseTransaction, Payment, Transaction, TransactionType};
use algorand_rs::{Algod, kmd, Address, MicroAlgos};
use std::error::Error;

// ideally these should be env variables
const ALGOD_URL: &str = "http://localhost:4001";
const ALGOD_TOKEN: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn main() -> Result<(), Box<dyn Error>> {
    let kmd_address = "http://localhost:4002";
    let kmd_token = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    let kmd_client = kmd::Client::new(kmd_address, kmd_token);
    let algod = Algod::new().bind(ALGOD_URL).auth(ALGOD_TOKEN).client()?;

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

    let payment = Payment {
        amount: MicroAlgos(10_000),
        receiver: to_address,
        close_remainder_to: None,
    };

    let transaction = Transaction::new(base, MicroAlgos(1), TransactionType::Payment(payment))?;

    let sign_response =
        kmd_client.sign_transaction(&wallet_handle_token, "testpassword", &transaction)?;

    println!(
        "kmd made signed transaction with {} bytes",
        sign_response.signed_transaction.len()
    );

    // Broadcast the transaction to the network
    // Note this transaction will get rejected because the accounts do not have any tokens
    let send_response = algod.raw_transaction(&sign_response.signed_transaction)?;

    println!("Transaction ID: {}", send_response.tx_id);

    Ok(())
}
