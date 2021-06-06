use algonaut_client::algod::v1::message::QueryAccountTransactions;
use algonaut_client::Algod;
use algonaut_core::CompiledTeal;
use algonaut_core::SignedLogic;
use algonaut_core::{LogicSignature, MicroAlgos, MultisigAddress, ToMsgPack};
use algonaut_transaction::transaction::TransactionSignature;
use algonaut_transaction::tx_group::TxGroup;
use algonaut_transaction::{account::Account, ConfigureAsset, Pay, SignedTransaction, Txn};
use dotenv::dotenv;
use std::convert::TryInto;
use std::env;
use std::error::Error;

#[test]
fn test_transaction() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v2()?;

    let from = account1();
    let to = account2();

    let params = algod.transaction_params()?;

    let t = Txn::new()
        .sender(from.address())
        .first_valid(params.last_round)
        .last_valid(params.last_round + 10)
        .genesis_id(params.genesis_id)
        .genesis_hash(params.genesis_hash)
        .fee(MicroAlgos(10_000))
        .payment(
            Pay::new()
                .amount(MicroAlgos(123_456))
                .to(to.address())
                .build(),
        )
        .build();

    let sign_response = from.sign_and_generate_signed_transaction(&t);
    println!("{:#?}", sign_response);
    assert!(sign_response.is_ok());
    let sign_response = sign_response.unwrap();

    let t_bytes = sign_response.to_msg_pack()?;
    // Broadcast the transaction to the network
    // Note this transaction will get rejected because the accounts do not have any tokens
    let send_response = algod.broadcast_raw_transaction(&t_bytes);

    println!("{:#?}", send_response);
    assert!(send_response.is_err());

    Ok(())
}

#[test]
fn test_multisig_transaction() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v2()?;

    let account1 = account1();
    let account2 = account2();

    let multisig_address = MultisigAddress::new(1, 2, &[account1.address(), account2.address()])?;

    let params = algod.transaction_params()?;

    let t = Txn::new()
        .sender(multisig_address.address())
        .first_valid(params.last_round)
        .last_valid(params.last_round + 10)
        .genesis_id(params.genesis_id)
        .genesis_hash(params.genesis_hash)
        .fee(MicroAlgos(10_000))
        .payment(
            Pay::new()
                .amount(MicroAlgos(123_456))
                .to(account2.address())
                .build(),
        )
        .build();

    let msig = account1.init_transaction_msig(&t, multisig_address.clone())?;
    let msig = account2.append_to_transaction_msig(&t, msig)?;

    let sig = TransactionSignature::Multi(msig);

    let signed_t = SignedTransaction {
        transaction: t,
        transaction_id: "".to_owned(),
        sig,
    };

    let t_bytes = signed_t.to_msg_pack()?;
    // Broadcast the transaction to the network
    // Note this transaction will get rejected because the accounts do not have any tokens
    let send_response = algod.broadcast_raw_transaction(&t_bytes);

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

    let program: CompiledTeal = algod
        .compile_teal(
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
        )?
        .try_into()?;

    let from_address = program.hash.parse()?;

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
                .to(account1().address())
                .build(),
        )
        .build();

    let signed_transaction = SignedTransaction {
        transaction: t,
        transaction_id: "".to_owned(),
        sig: TransactionSignature::Logic(SignedLogic {
            logic: program,
            args: vec![vec![1, 0], vec![255]],
            sig: LogicSignature::ContractAccount,
        }),
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

    let program = algod
        .compile_teal(
            r#"
#pragma version 3
int 1
"#
            .into(),
        )?
        .try_into()?;

    let from = account1();

    let params = algod.transaction_params()?;

    let t = Txn::new()
        .sender(from.address())
        .first_valid(params.last_round)
        .last_valid(params.last_round + 10)
        .genesis_id(params.genesis_id)
        .genesis_hash(params.genesis_hash)
        .fee(MicroAlgos(10_000))
        .payment(
            Pay::new()
                .amount(MicroAlgos(123_456))
                .to(account2().address())
                .build(),
        )
        .build();

    let signature = from.sign_program(&program);

    let signed_transaction = SignedTransaction {
        transaction: t,
        transaction_id: "".to_owned(),
        sig: TransactionSignature::Logic(SignedLogic {
            logic: program,
            args: vec![],
            sig: LogicSignature::DelegatedSig(signature),
        }),
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

    let program: CompiledTeal = algod
        .compile_teal(
            r#"
#pragma version 3
int 1
"#
            .into(),
        )?
        .try_into()?;

    let account1 = account1();
    let account2 = account2();

    let multisig_address = MultisigAddress::new(1, 2, &[account1.address(), account2.address()])?;

    let params = algod.transaction_params()?;

    let t = Txn::new()
        .sender(multisig_address.address())
        .first_valid(params.last_round)
        .last_valid(params.last_round + 10)
        .genesis_id(params.genesis_id)
        .genesis_hash(params.genesis_hash)
        .fee(MicroAlgos(10_000))
        .payment(
            Pay::new()
                .amount(MicroAlgos(123_456))
                .to(account3().address())
                .build(),
        )
        .build();

    let msig = account1.init_logic_msig(&program, multisig_address.clone())?;
    let msig = account2.append_to_logic_msig(&program, msig)?;

    let sig = TransactionSignature::Logic(SignedLogic {
        logic: program,
        args: vec![],
        sig: LogicSignature::DelegatedMultiSig(msig),
    });

    let signed_transaction = SignedTransaction {
        transaction: t,
        transaction_id: "".to_owned(),
        sig,
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

    let from = account1();

    let params = algod.transaction_params()?;

    let t = Txn::new()
        .sender(from.address())
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
    let signed_t = from.sign_and_generate_signed_transaction(&t)?;
    let send_response = algod.broadcast_raw_transaction(&signed_t.to_msg_pack()?);

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

    let account1 = account1();
    let account2 = account2();

    let params = algod.transaction_params()?;

    let t1 = &mut Txn::new()
        .sender(account1.address())
        .first_valid(params.last_round)
        .last_valid(params.last_round + 10)
        .genesis_id(params.genesis_id.clone())
        .genesis_hash(params.genesis_hash)
        .fee(MicroAlgos(1_000))
        .payment(
            Pay::new()
                .amount(MicroAlgos(1_000))
                .to(account2.address())
                .build(),
        )
        .build();

    let t2 = &mut Txn::new()
        .sender(account2.address())
        .first_valid(params.last_round)
        .last_valid(params.last_round + 10)
        .genesis_id(params.genesis_id)
        .genesis_hash(params.genesis_hash)
        .fee(MicroAlgos(1_000))
        .payment(
            Pay::new()
                .amount(MicroAlgos(3_000))
                .to(account1.address())
                .build(),
        )
        .build();

    TxGroup::assign_group_id(vec![t1, t2])?;

    let signed_t1 = account1.sign_and_generate_signed_transaction(&t1)?;
    let signed_t2 = account2.sign_and_generate_signed_transaction(&t2)?;

    let send_response = algod
        .broadcast_raw_transaction(&[signed_t1.to_msg_pack()?, signed_t2.to_msg_pack()?].concat());

    println!("{:#?}", send_response);
    assert!(send_response.is_err());

    Ok(())
}

fn account1() -> Account {
    let mnemonic = "fire enlist diesel stamp nuclear chunk student stumble call snow flock brush example slab guide choice option recall south kangaroo hundred matrix school above zero";
    Account::from_mnemonic(mnemonic).unwrap()
}

fn account2() -> Account {
    let mnemonic = "since during average anxiety protect cherry club long lawsuit loan expand embark forum theory winter park twenty ball kangaroo cram burst board host ability left";
    Account::from_mnemonic(mnemonic).unwrap()
}

fn account3() -> Account {
    let mnemonic = "auction inquiry lava second expand liberty glass involve ginger illness length room item discover ahead table doctor term tackle cement bonus profit right above catch";
    Account::from_mnemonic(mnemonic).unwrap()
}
