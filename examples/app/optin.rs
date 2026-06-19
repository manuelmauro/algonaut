use algonaut::Algod;
use algonaut::core::AppId;
use algonaut::transaction::account::Account;
use algonaut::transaction::builder::OptInApplication;
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

    info!("retrieving suggested params");
    let params = algod.suggested_params().await?;

    info!("building OptInApplication transaction");
    // TODO set a correct app-id here
    let t = OptInApplication::new(alice.address(), AppId(11)).build(&params)?;

    info!("signing transaction");
    let signed_t = alice.sign(t)?;

    info!("broadcasting transaction");
    let send_response = algod.send(&signed_t).await?;
    info!("response: {:?}", send_response);

    Ok(())
}
