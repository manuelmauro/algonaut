use algonaut::algod::v2::Algod;
use algonaut::core::{MicroAlgos, MultisigAddress};
use algonaut::transaction::transaction::TransactionSignature;
use algonaut::transaction::{account::Account, TxnBuilder};
use algonaut::transaction::{Pay, SignedTransaction};
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

    info!("creating account for bob");
    let bob = Account::from_mnemonic(&env::var("BOB_MNEMONIC")?)?;

    info!("creating multisig address");
    let multisig_address = MultisigAddress::new(1, 2, &[alice.address(), bob.address()])?;
    info!("multisig address: {}", multisig_address.address());

    info!("retrieving suggested params");
    let params = algod.suggested_transaction_params().await?;

    info!("building Pay transaction");
    let t = TxnBuilder::with(
        &params,
        Pay::new(
            multisig_address.address(),
            bob.address(),
            MicroAlgos(123_456),
        )
        .build(),
    )
    .build()?;

    info!("initializing multisig");
    let msig = alice.init_transaction_msig(&t, &multisig_address)?;

    info!("appending to multisig");
    let msig = bob.append_to_transaction_msig(&t, msig)?;

    info!("creating signature");
    let sig = TransactionSignature::Multi(msig);

    info!("signing transaction");
    let signed_t = SignedTransaction {
        transaction: t,
        transaction_id: "".to_owned(),
        sig,
        auth_address: None,
    };

    info!("broadcasting transaction");
    // the transaction will fail because the multisig address has no funds
    let send_response = algod.broadcast_signed_transaction(&signed_t).await;
    info!("response: {:?}", send_response);

    Ok(())
}
