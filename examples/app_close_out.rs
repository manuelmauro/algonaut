use algonaut::algod::v2::Algod;
use algonaut::transaction::account::Account;
use algonaut::transaction::builder::CloseApplication;
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

    info!("creating account for alice");
    let alice = Account::from_mnemonic(&env::var("ALICE_MNEMONIC")?)?;

    info!("retrieving suggested params");
    let params = algod.transaction_params().await?;

    info!("building CloseApplication transaction");
    // to test this, create an application that sets local state and opt-in, for/with the account sending this transaction.
    // the approval program has to return success for the local state to be cleared.
    let t = TxnBuilder::with(&params, CloseApplication::new(alice.address(), 3).build()).build()?;

    info!("signing transaction");
    let signed_t = alice.sign_transaction(t)?;

    info!("broadcasting transaction");
    let send_response = algod.signed_transaction(&signed_t).await?;
    info!("response: {:?}", send_response);

    Ok(())
}
