use algonaut::algod::AlgodBuilder;
use algonaut_core::{LogicSignature, MicroAlgos, SignedLogic};
use algonaut_transaction::transaction::TransactionSignature;
use algonaut_transaction::TxnBuilder;
use algonaut_transaction::{Pay, SignedTransaction};
use dotenv::dotenv;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = AlgodBuilder::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .build_v2()?;

    let program = algod
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

    let receiver = "DN7MBMCL5JQ3PFUQS7TMX5AH4EEKOBJVDUF4TCV6WERATKFLQF4MQUPZTA".parse()?;

    let params = algod.suggested_transaction_params().await?;

    let t = TxnBuilder::with(
        params,
        Pay::new(program.address, receiver, MicroAlgos(123_456)).build(),
    )
    .build();

    let signed_t = SignedTransaction {
        transaction: t,
        transaction_id: "".to_owned(),
        sig: TransactionSignature::Logic(SignedLogic {
            logic: program.program,
            args: vec![vec![1, 0], vec![255]],
            sig: LogicSignature::ContractAccount,
        }),
    };

    let send_response = algod.broadcast_signed_transaction(&signed_t).await;
    println!("response {:?}", send_response);

    Ok(())
}
