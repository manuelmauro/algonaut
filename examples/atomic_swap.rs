use algonaut::algod::v2::Algod;
use algonaut::core::MicroAlgos;
use algonaut::transaction::account::Account;
use algonaut::transaction::tx_group::TxGroup;
use algonaut::transaction::Pay;
use algonaut::transaction::TxnBuilder;
use dotenv::dotenv;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let account1 = Account::from_mnemonic("fire enlist diesel stamp nuclear chunk student stumble call snow flock brush example slab guide choice option recall south kangaroo hundred matrix school above zero")?;
    let account2 = Account::from_mnemonic("since during average anxiety protect cherry club long lawsuit loan expand embark forum theory winter park twenty ball kangaroo cram burst board host ability left")?;

    let params = algod.suggested_transaction_params().await?;

    // To keep the example short and as self-contained as possible, both transactions send Algos.
    // Normally you'll want to submit e.g. a payment and asset transfer or asset transfers for different assets.

    let t1 = &mut TxnBuilder::with(
        params.clone(),
        Pay::new(account1.address(), account2.address(), MicroAlgos(1_000)).build(),
    )
    .build();

    let t2 = &mut TxnBuilder::with(
        params,
        Pay::new(account2.address(), account1.address(), MicroAlgos(3_000)).build(),
    )
    .build();

    TxGroup::assign_group_id(vec![t1, t2])?;

    let signed_t1 = account1.sign_transaction(&t1)?;
    let signed_t2 = account2.sign_transaction(&t2)?;

    let send_response = algod
        .broadcast_signed_transactions(&[signed_t1, signed_t2])
        .await;
    println!("response: {:?}", send_response);

    Ok(())
}
