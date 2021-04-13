use algonaut::core::{Address, MicroAlgos};
use algonaut::crypto::mnemonic;
use algonaut::transaction::account::Account;
use algonaut::transaction::{Payment, Txn};
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
    println!("Public Key: {:?}", account.address().encode_string());

    let m = mnemonic::from_key(&account.seed())?;
    println!("Backup phrase: {}", m);

    let amount = MicroAlgos(0);
    let params = algod.transaction_params()?;
    let genesis_id = params.genesis_id;
    let genesis_hash = params.genesis_hash;

    let payment = Payment {
        amount,
        receiver: Address::from_string(
            "4MYUHDWHWXAKA5KA7U5PEN646VYUANBFXVJNONBK3TIMHEMWMD4UBOJBI4",
        )?,
        close_remainder_to: None,
    };

    let t = Txn::new()
        .sender(account.address())
        .first_valid(params.last_round)
        .last_valid(params.last_round + 1000)
        .genesis_id(genesis_id)
        .genesis_hash(genesis_hash)
        .fee(MicroAlgos(10_000))
        .payment(payment)
        .build();

    println!("Made unsigned transaction: {:?}", t);

    // Sign the transaction
    let signed_transaction = account.sign_transaction(&t)?;
    let bytes = rmp_serde::to_vec_named(&signed_transaction)?;

    let filename = "./signed.tx";
    let mut f = File::create(filename)?;
    f.write_all(&bytes)?;

    println!("Saved signed transaction to file: {}", filename);

    Ok(())
}
