use algonaut::algod::v2::Algod;
use algonaut::error::ServiceError;
use algonaut::model::algod::v2::PendingTransaction;
use algonaut::transaction::account::Account;
use algonaut::transaction::transaction::StateSchema;
use algonaut::transaction::CreateApplication;
use algonaut::transaction::TxnBuilder;
use dotenv::dotenv;
use std::env;
use std::error::Error;
use std::time::{Duration, Instant};
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

    info!("compiling approva and clear programs");
    let compiled_approval_program = algod.compile_teal(&approval_program).await?;
    let compiled_clear_program = algod.compile_teal(&clear_program).await?;

    info!("retrieving suggested params");
    let params = algod.suggested_transaction_params().await?;

    info!("building CreateApplication transaction");
    let t = TxnBuilder::with(
        &params,
        CreateApplication::new(
            alice.address(),
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
        .app_arguments(vec![vec![1, 0], vec![255]])
        .build(),
    )
    .build()?;

    info!("signing transaction");
    let signed_t = alice.sign_transaction(t)?;

    info!("broadcasting transaction");
    let send_response = algod.broadcast_signed_transaction(&signed_t).await?;

    info!("waiting for transaction finality");
    let pending_t = wait_for_pending_transaction(&algod, &send_response.tx_id).await?;

    info!(
        "application id: {:?}",
        pending_t.map(|t| t.application_index)
    );

    Ok(())
}

/// Utility function to wait on a transaction to be confirmed
async fn wait_for_pending_transaction(
    algod: &Algod,
    txid: &str,
) -> Result<Option<PendingTransaction>, ServiceError> {
    let timeout = Duration::from_secs(10);
    let start = Instant::now();
    loop {
        let pending_transaction = algod.pending_transaction_with_id(txid).await?;
        // If the transaction has been confirmed or we time out, exit.
        if pending_transaction.confirmed_round.is_some() {
            return Ok(Some(pending_transaction));
        } else if start.elapsed() >= timeout {
            return Ok(None);
        }
        std::thread::sleep(Duration::from_millis(250))
    }
}
