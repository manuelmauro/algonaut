use algonaut::algod::AlgodBuilder;
use algonaut_client::algod::v1::message::QueryAccountTransactions;
use algonaut_core::CompiledTeal;
use algonaut_core::SignedLogic;
use algonaut_core::{LogicSignature, MicroAlgos, MultisigAddress};
use algonaut_transaction::transaction::TransactionSignature;
use algonaut_transaction::tx_group::TxGroup;
use algonaut_transaction::{account::Account, CreateAsset, Pay, SignedTransaction, TxnBuilder};
use dotenv::dotenv;
use std::convert::TryInto;
use std::env;
use std::error::Error;
use tokio::test;

#[test]
async fn test_transaction() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = AlgodBuilder::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .build_v2()?;

    let from = account1();
    let to = account2();

    let params = algod.transaction_params().await?;

    let t = TxnBuilder::new(
        MicroAlgos(100_000),
        params.last_round,
        params.last_round + 10,
        params.genesis_hash,
        params.genesis_id,
        Pay::new(from.address(), to.address(), MicroAlgos(123_456)).build(),
    )
    .build();

    let sign_response = from.sign_transaction(&t);
    println!("{:#?}", sign_response);
    assert!(sign_response.is_ok());
    let sign_response = sign_response.unwrap();

    // Broadcast the transaction to the network
    // Note this transaction will get rejected because the accounts do not have any tokens
    let send_response = algod.broadcast_signed_transaction(&sign_response).await;

    println!("{:#?}", send_response);
    assert!(send_response.is_err());

    Ok(())
}

#[test]
async fn test_multisig_transaction() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = AlgodBuilder::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .build_v2()?;

    let account1 = account1();
    let account2 = account2();

    let multisig_address = MultisigAddress::new(1, 2, &[account1.address(), account2.address()])?;

    let params = algod.transaction_params().await?;

    let t = TxnBuilder::new(
        MicroAlgos(10_000),
        params.last_round,
        params.last_round + 10,
        params.genesis_hash,
        params.genesis_id,
        Pay::new(
            multisig_address.address(),
            account2.address(),
            MicroAlgos(123_456),
        )
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

    // Broadcast the transaction to the network
    // Note this transaction will get rejected because the accounts do not have any tokens
    let send_response = algod.broadcast_signed_transaction(&signed_t).await;

    println!("{:#?}", send_response);
    assert!(send_response.is_err());

    Ok(())
}

#[test]
async fn test_transaction_with_contract_account_logic_sig() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = AlgodBuilder::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .build_v2()?;

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
        )
        .await?
        .try_into()?;

    let from_address = program.hash.parse()?;

    let params = algod.transaction_params().await?;

    let t = TxnBuilder::new(
        MicroAlgos(10_000),
        params.last_round,
        params.last_round + 10,
        params.genesis_hash,
        params.genesis_id,
        Pay::new(from_address, account1().address(), MicroAlgos(123_456)).build(),
    )
    .build();

    let signed_t = SignedTransaction {
        transaction: t,
        transaction_id: "".to_owned(),
        sig: TransactionSignature::Logic(SignedLogic {
            logic: program,
            args: vec![vec![1, 0], vec![255]],
            sig: LogicSignature::ContractAccount,
        }),
    };

    // Broadcast the transaction to the network
    // Note this transaction will get rejected because the accounts do not have any tokens
    let send_response = algod.broadcast_signed_transaction(&signed_t).await;
    println!("response {:?}", send_response);
    assert!(send_response.is_err());

    Ok(())
}

#[test]
async fn test_transaction_with_delegated_logic_sig() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = AlgodBuilder::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .build_v2()?;

    let program = algod
        .compile_teal(
            r#"
#pragma version 3
int 1
"#
            .into(),
        )
        .await?
        .try_into()?;

    let from = account1();

    let params = algod.transaction_params().await?;

    let t = TxnBuilder::new(
        MicroAlgos(10_000),
        params.last_round,
        params.last_round + 10,
        params.genesis_hash,
        params.genesis_id,
        Pay::new(from.address(), account2().address(), MicroAlgos(123_456)).build(),
    )
    .build();

    let signature = from.generate_program_sig(&program);

    let signed_t = SignedTransaction {
        transaction: t,
        transaction_id: "".to_owned(),
        sig: TransactionSignature::Logic(SignedLogic {
            logic: program,
            args: vec![],
            sig: LogicSignature::DelegatedSig(signature),
        }),
    };

    // Broadcast the transaction to the network
    // Note this transaction will get rejected because the accounts do not have any tokens
    let send_response = algod.broadcast_signed_transaction(&signed_t).await;
    println!("response {:?}", send_response);
    assert!(send_response.is_err());

    Ok(())
}

#[test]
async fn test_transaction_with_delegated_logic_multisig() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = AlgodBuilder::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .build_v2()?;

    let program: CompiledTeal = algod
        .compile_teal(
            r#"
#pragma version 3
int 1
"#
            .into(),
        )
        .await?
        .try_into()?;

    let account1 = account1();
    let account2 = account2();

    let multisig_address = MultisigAddress::new(1, 2, &[account1.address(), account2.address()])?;

    let params = algod.transaction_params().await?;

    let t = TxnBuilder::new(
        MicroAlgos(10_000),
        params.last_round,
        params.last_round + 10,
        params.genesis_hash,
        params.genesis_id,
        Pay::new(
            multisig_address.address(),
            account3().address(),
            MicroAlgos(123_456),
        )
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

    let signed_t = SignedTransaction {
        transaction: t,
        transaction_id: "".to_owned(),
        sig,
    };

    // Broadcast the transaction to the network
    // Note this transaction will get rejected because the accounts do not have any tokens
    let send_response = algod.broadcast_signed_transaction(&signed_t).await;
    println!("response {:?}", send_response);
    assert!(send_response.is_err());

    Ok(())
}

#[test]
async fn test_create_asset_transaction() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = AlgodBuilder::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .build_v2()?;

    let from = account1();

    let params = algod.transaction_params().await?;

    let t = TxnBuilder::new(
        MicroAlgos(10_000),
        params.last_round,
        params.last_round + 10,
        params.genesis_hash,
        params.genesis_id,
        CreateAsset::new(from.address(), 1000000, 2, false)
            .unit_name("FOO".to_owned())
            .asset_name("Foo".to_owned())
            .build(),
    )
    .build();

    let signed_t = from.sign_transaction(&t)?;
    let send_response = algod.broadcast_signed_transaction(&signed_t).await;

    println!("{:#?}", send_response);
    assert!(send_response.is_err());

    Ok(())
}

#[test]
async fn test_transactions_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = AlgodBuilder::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .build_v1()?;

    let address = env::var("ACCOUNT")?.parse()?;

    let last_round = algod.status().await?.last_round;

    let query = QueryAccountTransactions {
        first_round: Some(last_round),
        from_date: None,
        last_round: Some(last_round),
        max: None,
        to_date: None,
    };

    let res = algod.transactions(&address, &query).await;

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
async fn test_pending_transactions_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = AlgodBuilder::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .build_v1()?;

    println!("{:?}", algod.pending_transactions(0).await);
    assert!(algod.pending_transactions(0).await.is_ok());

    Ok(())
}

#[test]
async fn test_transaction_information_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = AlgodBuilder::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .build_v1()?;

    println!("{:?}", algod.transaction_params().await);
    assert!(algod.transaction_params().await.is_ok());

    Ok(())
}

/// Swap between 2 accounts. For simplicity, both send Algos.
#[test]
async fn test_atomic_swap() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = AlgodBuilder::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .build_v2()?;

    let account1 = account1();
    let account2 = account2();

    let params = algod.transaction_params().await?;

    let t1 = &mut TxnBuilder::new(
        MicroAlgos(1_000),
        params.last_round,
        params.last_round + 10,
        params.genesis_hash,
        params.genesis_id.clone(),
        Pay::new(account1.address(), account2.address(), MicroAlgos(1_000)).build(),
    )
    .build();

    let t2 = &mut TxnBuilder::new(
        MicroAlgos(1_000),
        params.last_round,
        params.last_round + 10,
        params.genesis_hash,
        params.genesis_id,
        Pay::new(account2.address(), account1.address(), MicroAlgos(3_000)).build(),
    )
    .build();

    TxGroup::assign_group_id(vec![t1, t2])?;

    let signed_t1 = account1.sign_transaction(&t1)?;
    let signed_t2 = account2.sign_transaction(&t2)?;

    let send_response = algod
        .broadcast_signed_transactions(&[signed_t1, signed_t2])
        .await;

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
