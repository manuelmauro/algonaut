use algonaut::Algod;
use algonaut::core::{MicroAlgos, MultisigAddress};
use algonaut::transaction::Pay;
use algonaut::transaction::account::Account;
use algonaut::transaction::signer::MultisigSigningSession;
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
    let params = algod.suggested_params().await?;

    info!("building Pay transaction");
    let t = Pay::new(
        multisig_address.address(),
        bob.address(),
        MicroAlgos(123_456),
    )
    .build(&params)?;

    info!("signing transaction through a multisig session");
    let signed_t = MultisigSigningSession::new(multisig_address)
        .sign(t, &alice)?
        .sign_more(&bob)?
        .finish()?;

    info!("broadcasting transaction");
    // the transaction will fail because the multisig address has no funds
    let send_response = algod.send(&signed_t).await;
    info!("response: {:?}", send_response);

    Ok(())
}
