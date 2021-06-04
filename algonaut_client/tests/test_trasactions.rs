use algonaut_client::algod::v1::message::QueryAccountTransactions;
use algonaut_client::{Algod, Kmd};
use algonaut_core::{Address, LogicSignature, MicroAlgos, MultisigAddress, ToMsgPack};
use algonaut_crypto::MasterDerivationKey;
use algonaut_transaction::tx_group::TxGroup;
use algonaut_transaction::{account::Account, ConfigureAsset, Pay, SignedTransaction, Txn};
use data_encoding::BASE64;
use dotenv::dotenv;
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

    let transaction_bytes = signed_transaction.to_msg_pack()?;

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

    let transaction_bytes = signed_transaction.to_msg_pack()?;

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

    let transaction_bytes = signed_transaction.to_msg_pack()?;

    // Broadcast the transaction to the network
    // Note this transaction will get rejected because the accounts do not have any tokens
    let send_response = algod.broadcast_raw_transaction(&transaction_bytes);
    println!("response {:?}", send_response);
    assert!(send_response.is_err());

    Ok(())
}

#[test]
fn test_create_asset_transaction() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v2()?;

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let from_address = env::var("ACCOUNT")?.parse()?;

    let list_response = kmd.list_wallets()?;
    let wallet_id = match list_response
        .wallets
        .into_iter()
        .find(|wallet| wallet.name == "unencrypted-default-wallet")
    {
        Some(wallet) => wallet.id,
        None => return Err("Wallet not found".into()),
    };
    let init_response = kmd.init_wallet_handle(&wallet_id, "")?;
    let wallet_handle_token = init_response.wallet_handle_token;

    let params = algod.transaction_params()?;

    let t = Txn::new()
        .sender(from_address)
        .first_valid(params.last_round)
        .last_valid(params.last_round + 10)
        .genesis_id(params.genesis_id)
        .genesis_hash(params.genesis_hash)
        .fee(MicroAlgos(10_000))
        .asset_configuration(
            ConfigureAsset::new()
                .asset_name("Foo".to_owned())
                .decimals(2)
                .total(1000000)
                .unit_name("FOO".to_owned())
                .build(),
        )
        .build();
    let sign_response = kmd.sign_transaction(&wallet_handle_token, "", &t)?;
    let send_response = algod.broadcast_raw_transaction(&sign_response.signed_transaction);

    println!("{:#?}", send_response);
    assert!(send_response.is_ok());

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

    let address: String = env::var("ACCOUNT")?.parse()?;

    let last_round = algod.status()?.last_round;

    let query = QueryAccountTransactions {
        first_round: Some(last_round),
        from_date: None,
        last_round: Some(last_round),
        max: None,
        to_date: None,
    };

    let res = algod.transactions(address.as_str(), &query);

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

/// Swap between 2 accounts. For simplicity, both send Algos.
#[test]
fn test_atomic_swap() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v2()?;

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let address1: Address = env::var("ACCOUNT")?.parse()?;
    let address2: Address = env::var("ACCOUNT_2")?.parse()?;

    let list_response = kmd.list_wallets()?;
    let wallet_id = match list_response
        .wallets
        .into_iter()
        .find(|wallet| wallet.name == "unencrypted-default-wallet")
    {
        Some(wallet) => wallet.id,
        None => return Err("Wallet not found".into()),
    };
    let init_response = kmd.init_wallet_handle(&wallet_id, "")?;
    let wallet_handle_token = init_response.wallet_handle_token;

    let params = algod.transaction_params()?;

    let t1 = &mut Txn::new()
        .sender(address1)
        .first_valid(params.last_round)
        .last_valid(params.last_round + 10)
        .genesis_id(params.genesis_id.clone())
        .genesis_hash(params.genesis_hash)
        .fee(MicroAlgos(1_000))
        .payment(Pay::new().amount(MicroAlgos(1_000)).to(address2).build())
        .build();

    let t2 = &mut Txn::new()
        .sender(address2)
        .first_valid(params.last_round)
        .last_valid(params.last_round + 10)
        .genesis_id(params.genesis_id)
        .genesis_hash(params.genesis_hash)
        .fee(MicroAlgos(1_000))
        .payment(Pay::new().amount(MicroAlgos(3_000)).to(address1).build())
        .build();

    TxGroup::assign_group_id(vec![t1, t2])?;

    let sign_response_t1 = kmd.sign_transaction(&wallet_handle_token, "", &t1)?;
    let sign_response_t2 = kmd.sign_transaction(&wallet_handle_token, "", &t2)?;

    let send_response = algod.broadcast_raw_transaction(
        &[
            sign_response_t1.signed_transaction,
            sign_response_t2.signed_transaction,
        ]
        .concat(),
    );

    println!("{:#?}", send_response);
    assert!(send_response.is_ok());

    Ok(())
}
