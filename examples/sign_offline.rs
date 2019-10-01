use std::error::Error;
use std::fs::File;
use std::io::Write;

use algosdk::account::Account;
use algosdk::transaction::Transaction;
use algosdk::{mnemonic, Address, HashDigest, MicroAlgos, Round};

fn main() -> Result<(), Box<dyn Error>> {
    let account = Account::generate();

    let m = mnemonic::from_key(&account.seed)?;
    println!("Backup phrase: {}", m);
    let fee = MicroAlgos(1000);
    let amount = MicroAlgos(20000);
    let first_round = Round(642_715);
    let last_round = first_round + 1000;

    let transaction = Transaction::new_payment(
        account.address(),
        fee,
        first_round,
        last_round,
        Vec::new(),
        "",
        HashDigest([0; 32]),
        Address::from_string("4MYUHDWHWXAKA5KA7U5PEN646VYUANBFXVJNONBK3TIMHEMWMD4UBOJBI4")?,
        amount,
        None,
    )?;

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
