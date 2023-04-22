use algonaut::algod::v2::Algod;
use algonaut::core::MicroAlgos;
use algonaut::transaction::contract_account::ContractAccount;
use algonaut::transaction::Pay;
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

    info!("compiling teal program");
    let compiled_teal = algod
        .teal_compile(
            r#"
#pragma version 4
arg 0
byte 0x0100
==
arg 1
byte 0xFF
==
&&
"#
            .as_bytes(),
            None,
        )
        .await?;

    info!("creating contract account");
    let contract_account = ContractAccount::new(compiled_teal);

    info!("creating account for alice");
    let receiver = env::var("ALICE_ADDRESS")?.parse()?;

    info!("retrieving suggested params");
    let params = algod.transaction_params().await?;

    info!("building Pay transaction");
    let t = TxnBuilder::with(
        &params,
        Pay::new(*contract_account.address(), receiver, MicroAlgos(123_456)).build(),
    )
    .build()?;

    info!("signing transaction with contract account");
    let signed_t = contract_account.sign(t, vec![vec![1, 0], vec![255]])?;

    info!("broadcasting transaction");
    // the transaction will fail because contract_account has no funds
    let send_response = algod.signed_transaction(&signed_t).await;
    info!("response: {:?}", send_response);

    Ok(())
}
