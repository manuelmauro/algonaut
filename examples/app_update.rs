use algonaut::algod::v2::Algod;
use algonaut::transaction::TxnBuilder;
use algonaut_transaction::account::Account;
use algonaut_transaction::builder::UpdateApplication;
use dotenv::dotenv;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let sender = Account::from_mnemonic("auction inquiry lava second expand liberty glass involve ginger illness length room item discover ahead table doctor term tackle cement bonus profit right above catch")?;

    let approval_program = r#"
#pragma version 4
int 1
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
    // example approval program:
    // #pragma version 4
    // txna ApplicationArgs 0
    // byte 0x0100
    // ==
    // txna ApplicationArgs 1
    // byte 0xFF
    // ==
    // &&
    // example clear program:
    // #pragma version 4
    // int 1
    let t = TxnBuilder::with(
        params,
        UpdateApplication::new(
            sender.address(),
            5,
            compiled_approval_program.program,
            compiled_clear_program.program,
        )
        .app_arguments(vec![vec![1, 0], vec![255]]) // for the program being upgraded
        .build(),
    )
    .build();

    let signed_t = sender.sign_transaction(&t)?;

    let send_response = algod.broadcast_signed_transaction(&signed_t).await?;
    println!("response: {:?}", send_response);

    Ok(())
}
