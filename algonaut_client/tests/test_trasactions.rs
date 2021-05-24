use algonaut_client::{Algod, Kmd};
use algonaut_core::{Address, LogicSignature, MicroAlgos, MultisigAddress};
use algonaut_crypto::MasterDerivationKey;
use algonaut_transaction::{account::Account, Pay, SignedTransaction, Txn};
use data_encoding::BASE64;
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
    let from_address = gen_response.address.parse()?;

    let gen_response = kmd.generate_key(&wallet_handle_token)?;
    let to_address = gen_response.address.parse()?;

    let params = algod.transaction_params()?;

    let t = Txn::new()
        .sender(from_address)
        .first_valid(params.last_round)
        .last_valid(params.last_round + 10)
        .genesis_id(params.genesis_id)
        .genesis_hash(params.genesis_hash)
        .fee(MicroAlgos(10_000))
        .payment(
            Pay::new()
                .amount(MicroAlgos(123_456))
                .to(to_address)
                .build(),
        )
        .build();

    let sign_response = kmd.sign_transaction(&wallet_handle_token, "testpassword", &t);

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
fn test_transaction_with_contract_account_logic_sig() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v2()?;

    let res = algod.compile_teal(
        r#"
#pragma version 3
arg 0
byte 0x0100
==
arg 1
byte 0xFF
==
&&
"#
        .into(),
    )?;

    let program_bytes = BASE64.decode(res.result.as_bytes())?;
    let lsig = LogicSignature {
        logic: program_bytes,
        sig: None,
        msig: None,
        args: vec![vec![1, 0], vec![255]],
    };

    let from_address: Address = res.hash.parse()?;
    let to_address: Address =
        "ZOSNRNYXOHQIPFDHJWDBWKRZFRJUMCXQGKTHV7LWZZNIKEEU6AWEODSQ4U".parse()?;

    let params = algod.transaction_params()?;

    let t = Txn::new()
        .sender(from_address)
        .first_valid(params.last_round)
        .last_valid(params.last_round + 10)
        .genesis_id(params.genesis_id)
        .genesis_hash(params.genesis_hash)
        .fee(MicroAlgos(10_000))
        .payment(
            Pay::new()
                .amount(MicroAlgos(123_456))
                .to(to_address)
                .build(),
        )
        .build();

    let signed_transaction = SignedTransaction {
        sig: None,
        multisig: None,
        logicsig: Some(lsig),
        transaction: t,
        transaction_id: "".to_owned(),
    };

    let transaction_bytes = rmp_serde::to_vec_named(&signed_transaction)?;

    // Broadcast the transaction to the network
    // Note this transaction will get rejected because the accounts do not have any tokens
    let send_response = algod.broadcast_raw_transaction(&transaction_bytes);
    println!("response {:?}", send_response);
    assert!(send_response.is_err());

    Ok(())
}

#[test]
fn test_transaction_with_delegated_logic_sig() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v2()?;

    let res = algod.compile_teal(
        r#"
#pragma version 3
int 1
"#
        .into(),
    )?;

    let mnemonic = "fire enlist diesel stamp nuclear chunk student stumble call snow flock brush example slab guide choice option recall south kangaroo hundred matrix school above zero";
    let account = Account::from_mnemonic(mnemonic)?;

    let program_bytes = BASE64.decode(res.result.as_bytes())?;
    let signature = account.sign_program(&program_bytes);
    let lsig = LogicSignature {
        logic: program_bytes,
        sig: Some(signature),
        msig: None,
        args: vec![],
    };

    let from_address = account.address();
    let to_address: Address =
        "ZOSNRNYXOHQIPFDHJWDBWKRZFRJUMCXQGKTHV7LWZZNIKEEU6AWEODSQ4U".parse()?;

    let params = algod.transaction_params()?;

    let t = Txn::new()
        .sender(from_address)
        .first_valid(params.last_round)
        .last_valid(params.last_round + 10)
        .genesis_id(params.genesis_id)
        .genesis_hash(params.genesis_hash)
        .fee(MicroAlgos(10_000))
        .payment(
            Pay::new()
                .amount(MicroAlgos(123_456))
                .to(to_address)
                .build(),
        )
        .build();

    let signed_transaction = SignedTransaction {
        sig: None,
        multisig: None,
        logicsig: Some(lsig),
        transaction: t,
        transaction_id: "".to_owned(),
    };

    let transaction_bytes = rmp_serde::to_vec_named(&signed_transaction)?;

    // Broadcast the transaction to the network
    // Note this transaction will get rejected because the accounts do not have any tokens
    let send_response = algod.broadcast_raw_transaction(&transaction_bytes);
    println!("response {:?}", send_response);
    assert!(send_response.is_err());

    Ok(())
}

#[test]
fn test_transaction_with_delegated_logic_multisig() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v2()?;

    let res = algod.compile_teal(
        r#"
#pragma version 3
int 1
"#
        .into(),
    )?;

    let mnemonic1 = "auction inquiry lava second expand liberty glass involve ginger illness length room item discover ahead table doctor term tackle cement bonus profit right above catch";
    let account1 = Account::from_mnemonic(mnemonic1)?;

    let mnemonic2 = "since during average anxiety protect cherry club long lawsuit loan expand embark forum theory winter park twenty ball kangaroo cram burst board host ability left";
    let account2 = Account::from_mnemonic(mnemonic2)?;

    let multisig_address = MultisigAddress::new(1, 2, &[account1.address(), account2.address()])?;

    let program_bytes = BASE64.decode(res.result.as_bytes())?;
    let lsig = LogicSignature {
        logic: program_bytes,
        sig: None,
        msig: None,
        args: vec![],
    };
    let lsig = account1.sign_logic_msig(lsig, multisig_address.clone())?;
    let lsig = account2.append_to_logic_msig(lsig)?;

    let from_address = multisig_address.address();
    let to_address: Address =
        "ZOSNRNYXOHQIPFDHJWDBWKRZFRJUMCXQGKTHV7LWZZNIKEEU6AWEODSQ4U".parse()?;

    let params = algod.transaction_params()?;

    let t = Txn::new()
        .sender(from_address)
        .first_valid(params.last_round)
        .last_valid(params.last_round + 10)
        .genesis_id(params.genesis_id)
        .genesis_hash(params.genesis_hash)
        .fee(MicroAlgos(10_000))
        .payment(
            Pay::new()
                .amount(MicroAlgos(123_456))
                .to(to_address)
                .build(),
        )
        .build();

    let signed_transaction = SignedTransaction {
        sig: None,
        multisig: None,
        logicsig: Some(lsig),
        transaction: t,
        transaction_id: "".to_owned(),
    };

    let transaction_bytes = rmp_serde::to_vec_named(&signed_transaction)?;

    // Broadcast the transaction to the network
    // Note this transaction will get rejected because the accounts do not have any tokens
    let send_response = algod.broadcast_raw_transaction(&transaction_bytes);
    println!("response {:?}", send_response);
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
