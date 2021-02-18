use algorand_rs::account::Account;
use algorand_rs::transaction::{BaseTransaction, Payment, Transaction, TransactionType};
use algorand_rs::{mnemonic, Address, Algod, HashDigest, MicroAlgos, Round};
use dotenv::dotenv;
use std::env;
use std::error::Error;

#[test]
fn test_proper_client_builder() {
    let algod = Algod::new()
        .bind("http://localhost:4001")
        .auth("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        .client_v1();

    assert!(algod.ok().is_some());
}

#[test]
fn test_health_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v1()?;

    assert_eq!(algod.health()?, ());

    Ok(())
}

#[test]
fn test_versions_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v1()?;

    assert!(algod.versions().ok().is_some());

    Ok(())
}

#[test]
fn test_status_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v1()?;

    assert!(algod.status().ok().is_some());

    Ok(())
}

#[test]
fn test_status_after_block_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v1()?;

    assert!(algod.status_after_block(Round(0)).ok().is_some());

    Ok(())
}

#[test]
fn test_block_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v1()?;

    println!("{:?}", algod.block(Round(0)));
    assert!(algod.block(Round(0)).ok().is_some());

    Ok(())
}

#[test]
fn test_ledger_supply_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v1()?;

    assert!(algod.ledger_supply().ok().is_some());

    Ok(())
}

#[test]
fn test_account_information_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v1()?;

    assert!(algod
        .account_information("4MYUHDWHWXAKA5KA7U5PEN646VYUANBFXVJNONBK3TIMHEMWMD4UBOJBI4")
        .ok()
        .is_some());

    Ok(())
}

#[test]
fn test_pending_transactions_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v1()?;

    println!("{:?}", algod.pending_transactions(0));
    assert!(algod.pending_transactions(0).ok().is_some());

    Ok(())
}

#[test]
fn test_pending_transaction_information_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v1()?;

    println!(
        "{:?}",
        algod.pending_transaction_information(
            "RORHWYQ3227P6KBZNEBCXP4AC2C7ZSZFVGDBKJO63PD6AXFJLAYA"
        )
    );
    assert!(algod
        .pending_transaction_information("RORHWYQ3227P6KBZNEBCXP4AC2C7ZSZFVGDBKJO63PD6AXFJLAYA")
        .ok()
        .is_some());

    Ok(())
}

#[test]
fn test_transactions_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v1()?;

    assert!(algod
        .transactions(
            "4MYUHDWHWXAKA5KA7U5PEN646VYUANBFXVJNONBK3TIMHEMWMD4UBOJBI4",
            None,
            None,
            None,
            None,
            None,
        )
        .ok()
        .is_some());

    Ok(())
}

#[test]
fn test_send_transaction_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v1()?;

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

    println!("{:?}", algod.send_transaction(&signed_transaction));
    assert!(algod.send_transaction(&signed_transaction).ok().is_some());

    Ok(())
}

#[test]
fn test_transaction_information_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v1()?;

    println!("{:?}", algod.transaction_params());
    assert!(algod.transaction_params().ok().is_some());

    Ok(())
}
