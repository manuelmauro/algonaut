use algonaut::core::MicroAlgos;
use algonaut::crypto::mnemonic;
use algonaut::transaction::account::Account;
use algonaut::transaction::{BaseTransaction, Payment, Transaction, TransactionType};
use algonaut::Algod;
use dotenv::dotenv;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v1()?;

    // print algod status
    let node_status = algod.status()?;
    println!("node_status: {:?}", node_status);

    let account = Account::generate();
    println!("Public Key: {:?}", account.address().to_string());

    let m = mnemonic::from_key(&account.seed())?;
    println!("Backup phrase: {}", m);

    let fee = MicroAlgos(1000);
    let amount = MicroAlgos(0);
    let first_round = node_status.last_round;
    let last_round = node_status.last_round + 1000;

    let transaction_params = algod.transaction_params()?;
    let genesis_id = transaction_params.genesis_id;
    let genesis_hash = transaction_params.genesis_hash;

    let base = BaseTransaction {
        sender: account.address(),
        first_valid: first_round,
        last_valid: last_round,
        note: Vec::new(),
        genesis_id,
        genesis_hash,
    };

    let payment = Payment {
        amount,
        receiver: "4MYUHDWHWXAKA5KA7U5PEN646VYUANBFXVJNONBK3TIMHEMWMD4UBOJBI4".parse()?,
        close_remainder_to: None,
    };

    let transaction = Transaction::new(base, fee, TransactionType::Payment(payment))?;

    println!("Made unsigned transaction: {:?}", transaction);

    // Sign the transaction
    let signed_transaction = account.sign_transaction(&transaction)?;
    let bytes = rmp_serde::to_vec_named(&signed_transaction)?;

    let filename = "./signed.tx";
    let mut f = File::create(filename)?;
    f.write_all(&bytes)?;

    println!("Saved signed transaction to file: {}", filename);

    Ok(())
}
