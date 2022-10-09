use algonaut::algod::v2::Algod;
use algonaut::core::MicroAlgos;
use algonaut::transaction::account::Account;
use algonaut::transaction::tx_group::TxGroup;
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

    info!("creating accounts for alice and bob");
    let alice = Account::from_mnemonic(&env::var("ALICE_MNEMONIC")?)?;
    let bob = Account::from_mnemonic(&env::var("BOB_MNEMONIC")?)?;

    info!("retrieving suggested params");
    let params = algod.suggested_transaction_params().await?;

    // To keep the example short and as self-contained as possible, both transactions send Algos.
    // Normally you'll want to submit e.g. a payment and asset transfer or asset transfers for different assets.

    info!("building payment transaction alice -> bob");
    let mut t1 = TxnBuilder::with(
        &params,
        Pay::new(alice.address(), bob.address(), MicroAlgos(1_000)).build(),
    )
    .build()?;

    info!("building payment transaction bob -> alice");
    let mut t2 = TxnBuilder::with(
        &params,
        Pay::new(bob.address(), alice.address(), MicroAlgos(3_000)).build(),
    )
    .build()?;

    info!("grouping transactions");
    TxGroup::assign_group_id(&mut [&mut t1, &mut t2])?;

    info!("signing transactions");
    let signed_t1 = alice.sign_transaction(t1)?;
    let signed_t2 = bob.sign_transaction(t2)?;

    info!("broadcasting transaction");
    let send_response = algod
        .broadcast_signed_transactions(&[signed_t1, signed_t2])
        .await;
    info!("response: {:?}", send_response);

    Ok(())
}
