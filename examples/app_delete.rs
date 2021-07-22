use algonaut::algod::AlgodBuilder;
use algonaut::transaction::TxnBuilder;
use algonaut_transaction::account::Account;
use algonaut_transaction::builder::DeleteApplication;
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

    let creator = Account::from_mnemonic("auction inquiry lava second expand liberty glass involve ginger illness length room item discover ahead table doctor term tackle cement bonus profit right above catch")?;

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
        DeleteApplication::new(creator.address(), 5)
            .app_arguments(vec![vec![1, 0], vec![255]])
            .build(),
    )
    .build();

    let signed_t = creator.sign_transaction(&t)?;

    let send_response = algod.broadcast_signed_transaction(&signed_t).await?;
    println!("response: {:?}", send_response);

    Ok(())
}
