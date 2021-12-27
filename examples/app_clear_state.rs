use algonaut::algod::v2::Algod;
use algonaut::transaction::TxnBuilder;
use algonaut_transaction::account::Account;
use algonaut_transaction::builder::ClearApplication;
use dotenv::dotenv;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let sender = Account::from_mnemonic("auction inquiry lava second expand liberty glass involve ginger illness length room item discover ahead table doctor term tackle cement bonus profit right above catch")?;

    let params = algod.suggested_transaction_params().await?;
    // to test this, create an application that sets local state and opt-in, for/with the account sending this transaction.
    let t = TxnBuilder::with(params, ClearApplication::new(sender.address(), 5).build()).build();

    let signed_t = sender.sign_transaction(&t)?;

    let send_response = algod.broadcast_signed_transaction(&signed_t).await?;
    println!("response: {:?}", send_response);

    Ok(())
}
