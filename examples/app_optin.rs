use algonaut::algod::AlgodBuilder;
use algonaut::transaction::TxnBuilder;
use algonaut_transaction::account::Account;
use algonaut_transaction::builder::OptInApplication;
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

    let creator = Account::from_mnemonic("fire enlist diesel stamp nuclear chunk student stumble call snow flock brush example slab guide choice option recall south kangaroo hundred matrix school above zero")?;

    let params = algod.suggested_transaction_params().await?;
    // contract being opted-in: no args, returns success
    let t = TxnBuilder::with(params, OptInApplication::new(creator.address(), 5).build()).build();

    let signed_t = creator.sign_transaction(&t)?;

    let send_response = algod.broadcast_signed_transaction(&signed_t).await?;
    println!("response: {:?}", send_response);

    Ok(())
}
