use algonaut::algod::v2::Algod;
use algonaut::transaction::account::Account;
use algonaut::transaction::builder::UpdateApplication;
use algonaut::transaction::TxnBuilder;
use dotenv::dotenv;
use std::env;
use std::error::Error;
#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    env_logger::init();

    info!("creating algod client");
    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    info!("creating account for alice");
    let alice = Account::from_mnemonic(&env::var("ALICE_MNEMONIC")?)?;

    info!("creating approval program");
    let approval_program = r#"
#pragma version 4
int 1
"#
    .as_bytes();

    info!("creating clear program");
    let clear_program = r#"
#pragma version 4
int 1
"#
    .as_bytes();

    info!("compiling approval program");
    let compiled_approval_program = algod.compile_teal(&approval_program).await?;

    info!("compiling approval program");
    let compiled_clear_program = algod.compile_teal(&clear_program).await?;

    info!("retrieving suggested params");
    let params = algod.suggested_transaction_params().await?;

    info!("building UpdateApplication transaction");
    let t = TxnBuilder::with(
        &params,
        UpdateApplication::new(
            alice.address(),
            5,
            compiled_approval_program,
            compiled_clear_program,
        )
        .app_arguments(vec![vec![1, 0], vec![255]]) // for the program being upgraded
        .build(),
    )
    .build()?;

    info!("signing transaction");
    let signed_t = alice.sign_transaction(t)?;

    info!("broadcasting transaction");
    let send_response = algod.broadcast_signed_transaction(&signed_t).await?;
    info!("response: {:?}", send_response);

    Ok(())
}
