use algonaut_client::{Algod, Kmd};
use algonaut_core::address::Address;
use algonaut_core::{MicroAlgos, Round};
use algonaut_crypto::{mnemonic, MasterDerivationKey};
use algonaut_transaction::account::Account;
use algonaut_transaction::{BaseTransaction, Payment, Transaction, TransactionType};
use dotenv::dotenv;
use rand::{distributions::Alphanumeric, Rng};
use std::env;
use std::error::Error;

#[test]
fn test_transaction() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v1()?;
    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let wallet = kmd.create_wallet(
        "testwallet",
        "testpassword",
        "sqlite",
        MasterDerivationKey([0; 32]),
    );

    println!("{:#?}", wallet);

    let list_response = kmd.list_wallets()?;

    let wallet_id = match list_response
        .wallets
        .into_iter()
        .find(|wallet| wallet.name == "testwallet")
    {
        Some(wallet) => wallet.id,
        None => return Err("Wallet not found".into()),
    };

    let init_response = kmd.init_wallet_handle(&wallet_id, "testpassword")?;
    let wallet_handle_token = init_response.wallet_handle_token;

    let gen_response = kmd.generate_key(&wallet_handle_token)?;
    let from_address = Address::from_string(&gen_response.address)?;

    let gen_response = kmd.generate_key(&wallet_handle_token)?;
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

    let sign_response = kmd.sign_transaction(&wallet_handle_token, "testpassword", &transaction);

    println!("{:#?}", sign_response);
    assert!(sign_response.is_ok());

    let sign_response = sign_response.unwrap();

    println!(
        "kmd made signed transaction with {} bytes",
        sign_response.signed_transaction.len()
    );

    // Broadcast the transaction to the network
    // Note this transaction will get rejected because the accounts do not have any tokens
    let send_response = algod.raw_transaction(&sign_response.signed_transaction);

    println!("{:#?}", send_response);
    assert!(send_response.is_err());

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

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;
    let wallet_name: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

    let wallet = kmd.create_wallet(
        wallet_name.as_ref(),
        "testpassword",
        "sqlite",
        MasterDerivationKey([0; 32]),
    );

    println!("{:#?}", wallet);
    assert!(wallet.is_ok());

    let wallet = wallet.unwrap();

    let id = wallet.wallet.id.as_ref();
    let handle = kmd.init_wallet_handle(id, "testpassword");

    println!("{:#?}", handle);
    assert!(handle.is_ok());

    let gen_response = kmd.generate_key(handle.unwrap().wallet_handle_token.as_ref())?;

    let node_status = algod.status()?;

    let res = algod.transactions(
        &gen_response.address,
        Some(node_status.last_round),
        Some(node_status.last_round),
        None,
        None,
        None,
    );

    println!("{:#?}", res);
    assert!(res.is_ok());

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
    assert!(algod.pending_transactions(0).is_ok());

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

    let transaction_params = algod.transaction_params();

    println!("{:#?}", transaction_params);
    assert!(transaction_params.is_ok());

    let transaction_params = transaction_params.unwrap();

    let genesis_id = transaction_params.genesis_id;
    let genesis_hash = transaction_params.genesis_hash;

    let base = BaseTransaction {
        sender: account.address(),
        first_valid: first_round,
        last_valid: last_round,
        note: Vec::new(),
        genesis_id: genesis_id,
        genesis_hash: genesis_hash,
    };

    let payment = Payment {
        amount,
        receiver: Address::from_string(
            "4MYUHDWHWXAKA5KA7U5PEN646VYUANBFXVJNONBK3TIMHEMWMD4UBOJBI4",
        )?,
        close_remainder_to: None,
    };

    let transaction = Transaction::new(base, fee, TransactionType::Payment(payment));

    println!("{:#?}", transaction);
    assert!(transaction.is_ok());

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
    assert!(algod.transaction_params().is_ok());

    Ok(())
}
