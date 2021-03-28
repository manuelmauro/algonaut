use algonaut::core::{Address, MicroAlgos, Round};
use algonaut::crypto::{mnemonic, HashDigest};
use algonaut::transaction::account::Account;
use algonaut::transaction::{BaseTransaction, Payment, Transaction, TransactionType};
use std::error::Error;
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn Error>> {
    let account = Account::generate();

    let m = mnemonic::from_key(&account.seed())?;
    println!("Backup phrase: {}", m);
    let fee = MicroAlgos(1000);
    let amount = MicroAlgos(20000);
    let first_round = Round(642_715);
    let last_round = first_round + 1000;

    let base = BaseTransaction {
        sender: account.address(),
        first_valid: first_round,
        last_valid: last_round,
        note: Vec::new(),
        genesis_id: "".to_string(),
        genesis_hash: HashDigest([0; 32]),
    };

    let payment = Payment {
        amount,
        receiver: Address::from_string(
            "4MYUHDWHWXAKA5KA7U5PEN646VYUANBFXVJNONBK3TIMHEMWMD4UBOJBI4",
        )?,
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
