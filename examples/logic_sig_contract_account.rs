use algonaut::algod::v2::Algod;
use algonaut_core::MicroAlgos;
use algonaut_transaction::account::ContractAccount;
use algonaut_transaction::Pay;
use algonaut_transaction::TxnBuilder;
use dotenv::dotenv;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let compiled_teal = algod
        .compile_teal(
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
        )
        .await?;
    let contract_account = ContractAccount::new(compiled_teal);

    let receiver = "DN7MBMCL5JQ3PFUQS7TMX5AH4EEKOBJVDUF4TCV6WERATKFLQF4MQUPZTA".parse()?;

    let params = algod.suggested_transaction_params().await?;

    let t = TxnBuilder::with(
        params,
        Pay::new(contract_account.address, receiver, MicroAlgos(123_456)).build(),
    )
    .build();

    let signed_t = contract_account.sign(&t, vec![vec![1, 0], vec![255]])?;

    let send_response = algod.broadcast_signed_transaction(&signed_t).await;
    println!("response {:?}", send_response);

    Ok(())
}
