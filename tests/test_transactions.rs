use algonaut::algod::v2::Algod;
use algonaut_core::Address;
use algonaut_transaction::account::Account;
use algonaut_transaction::transaction::StateSchema;
use algonaut_transaction::CreateApplication;
use algonaut_transaction::TxnBuilder;
use dotenv::dotenv;
use std::env;
use std::error::Error;
use tokio::test;

#[test]
async fn test_pending_transactions_endpoint() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    println!("{:?}", algod.pending_transactions(0).await);
    assert!(algod.pending_transactions(0).await.is_ok());

    Ok(())
}

#[test]
async fn test_transaction_information_endpoint() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    println!("{:?}", algod.transaction_params().await);
    assert!(algod.transaction_params().await.is_ok());

    Ok(())
}

// Preconditions: import and fund sender account
#[test]
async fn test_app_call_parameters() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let sender = Account::from_mnemonic("auction inquiry lava second expand liberty glass involve ginger illness length room item discover ahead table doctor term tackle cement bonus profit right above catch")?;
    dbg!(algod.account_information(&sender.address()).await?);

    let approval_program = r#"
#pragma version 4
txna ApplicationArgs 0
byte 0x0100FF
==
txna ApplicationArgs 1
btoi
int 18446744073709551615
==
&&
txna ApplicationArgs 2
addr MKRBTLNZRS3UZZDS5OWPLP7YPHUDNKXFUFN5PNCJ3P2XRG74HNOGY6XOYQ
==
&&
txna ApplicationArgs 3
byte b64 aGVsbG8=
==
&&
txna Accounts 1
addr BKO5VOVEPG7TXAF5YYJHPMRADP2FQ3MGXEZG5BQEBJ2BF76IGFSO5M7PLE
==
&&
txna Accounts 2
addr OEZKCDB3GLVZ5BJ7BNFS2IU7XL5TQ5E4QMRKODM3EM6JODWWSYV4FECJRY
==
&&
txna Applications 1
int 1
==
&&
txna Applications 2
int 2
==
&&
txna Assets 0
int 1234
==
&&
"#
    .as_bytes();

    let clear_program = r#"
#pragma version 4
int 1
"#
    .as_bytes();

    let compiled_approval_program = algod.compile_teal(&approval_program).await?;
    let compiled_clear_program = algod.compile_teal(&clear_program).await?;

    let params = algod.suggested_transaction_params().await?;
    let t = TxnBuilder::with(
        &params,
        CreateApplication::new(
            sender.address(),
            compiled_approval_program.clone(),
            compiled_clear_program,
            StateSchema {
                number_ints: 0,
                number_byteslices: 0,
            },
            StateSchema {
                number_ints: 0,
                number_byteslices: 0,
            },
        )
        .app_arguments(vec![
            vec![1, 0, 255],                 // bytes (directly)
            u64::MAX.to_be_bytes().to_vec(), // u64
            "MKRBTLNZRS3UZZDS5OWPLP7YPHUDNKXFUFN5PNCJ3P2XRG74HNOGY6XOYQ"
                .parse::<Address>()?
                .0
                .to_vec(), // address
            "hello".as_bytes().to_vec(),     // string
        ])
        .accounts(vec![
            "BKO5VOVEPG7TXAF5YYJHPMRADP2FQ3MGXEZG5BQEBJ2BF76IGFSO5M7PLE".parse()?,
            "OEZKCDB3GLVZ5BJ7BNFS2IU7XL5TQ5E4QMRKODM3EM6JODWWSYV4FECJRY".parse()?,
        ])
        .foreign_apps(vec![1, 2])
        .foreign_assets(vec![1234])
        .build(),
    )
    .build()?;

    let signed_t = sender.sign_transaction(t)?;

    let send_response = algod.broadcast_signed_transaction(&signed_t).await?;
    println!("send_response: {:?}", send_response);

    Ok(())
}
