use algonaut::algod::AlgodBuilder;
use algonaut_transaction::AcceptAsset;
use algonaut_transaction::{account::Account, TxnBuilder};
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

    let account = Account::from_mnemonic("since during average anxiety protect cherry club long lawsuit loan expand embark forum theory winter park twenty ball kangaroo cram burst board host ability left")?;

    let params = algod.suggested_transaction_params().await?;

    let t = TxnBuilder::with(params, AcceptAsset::new(account.address(), 4).build()).build();

    let sign_response = account.sign_transaction(&t)?;

    // Broadcast the transaction to the network
    // Note this transaction will get rejected because the accounts do not have any tokens
    let send_response = algod.broadcast_signed_transaction(&sign_response).await;
    println!("{:#?}", send_response);

    Ok(())
}
