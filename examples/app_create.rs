use algonaut::algod::v2::{Algod, SourceMap};
use algonaut::transaction::CreateApplication;
use algonaut::transaction::account::Account;
use algonaut::transaction::transaction::StateSchema;
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

    // To read teal from file: fs::read("my_program.teal")
    let approval_program = r#"
#pragma version 4
txna ApplicationArgs 0
byte 0x0100
==
txna ApplicationArgs 1
byte 0xFF
==
&&
"#
    .as_bytes();

    let clear_program = r#"
#pragma version 4
int 1
"#
    .as_bytes();

    info!("compiling approval and clear programs");
    let compiled_approval_program = algod
        .teal_compile(approval_program, SourceMap::Skip)
        .await?;
    let compiled_clear_program = algod.teal_compile(clear_program, SourceMap::Skip).await?;

    info!("retrieving suggested params");
    let params = algod.txn_params().await?;

    info!("building CreateApplication transaction");
    let t = CreateApplication::new(
        alice.address(),
        compiled_approval_program,
        compiled_clear_program,
        StateSchema::empty(),
        StateSchema::empty(),
    )
    .build(&params)?;

    info!("signing transaction");
    let signed_t = alice.sign_transaction(t)?;

    info!("broadcasting transaction and waiting for finality");
    let pending_t = algod.submit(&signed_t).await?.confirm().await?;

    info!("application id: {:?}", pending_t.application_index);

    Ok(())
}
