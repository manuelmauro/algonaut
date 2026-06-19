use algonaut::Algod;
use algonaut::core::MicroAlgos;
use algonaut::transaction::Pay;
use algonaut::transaction::account::Account;
use algonaut::transaction::group::TransactionGroup;
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

    info!("creating accounts for alice and bob");
    let alice = Account::from_mnemonic(&env::var("ALICE_MNEMONIC")?)?;
    let bob = Account::from_mnemonic(&env::var("BOB_MNEMONIC")?)?;

    info!("retrieving suggested params");
    let params = algod.suggested_params().await?;

    // To keep the example short and as self-contained as possible, both transactions send Algos.
    // Normally you'll want to submit e.g. a payment and asset transfer or asset transfers for different assets.

    info!("building payment transaction alice -> bob");
    let t1 = Pay::new(alice.address(), bob.address(), MicroAlgos(1_000)).build(&params)?;

    info!("building payment transaction bob -> alice");
    let t2 = Pay::new(bob.address(), alice.address(), MicroAlgos(3_000)).build(&params)?;

    info!("grouping transactions");
    let group = TransactionGroup::try_from(vec![t1, t2])?;
    let mut iter = group.into_iter();
    let t1 = iter.next().expect("group has two transactions");
    let t2 = iter.next().expect("group has two transactions");

    info!("signing transactions");
    let signed_t1 = alice.sign(t1)?;
    let signed_t2 = bob.sign(t2)?;

    info!("broadcasting transaction");
    let send_response = algod.send_transactions(&[signed_t1, signed_t2]).await;
    info!("response: {:?}", send_response);

    Ok(())
}
