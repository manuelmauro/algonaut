use algonaut::algod::v2::Algod;
use algonaut::core::{LogicSignature, MicroAlgos};
use algonaut::transaction::transaction::TransactionSignature;
use algonaut::transaction::{account::Account, TxnBuilder};
use algonaut::transaction::{Pay, SignedTransaction};
use algonaut_transaction::transaction::SignedLogic;
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

    info!("compiling teal program");
    let program = algod
        .compile_teal(
            r#"
#pragma version 3
int 1
"#
            .as_bytes(),
        )
        .await?;

    info!("creating account for alice");
    let alice = Account::from_mnemonic(&env::var("ALICE_MNEMONIC")?)?;

    info!("creating account for bob");
    let bob = Account::from_mnemonic(&env::var("BOB_MNEMONIC")?)?;

    info!("retrieving suggested params");
    let params = algod.suggested_transaction_params().await?;

    info!("building Pay transaction");
    let t = TxnBuilder::with(
        &params,
        Pay::new(alice.address(), bob.address(), MicroAlgos(123_456)).build(),
    )
    .build()?;

    info!("generating program signature");
    let signature = alice.generate_program_sig(&program);

    info!("delegating signature for the transaction");
    let signed_t = SignedTransaction {
        transaction: t,
        transaction_id: "".to_owned(),
        sig: TransactionSignature::Logic(SignedLogic {
            logic: program,
            args: vec![],
            sig: LogicSignature::DelegatedSig(signature),
        }),
        auth_address: None,
    };

    info!("broadcasting transaction");
    let send_response = algod.broadcast_signed_transaction(&signed_t).await;
    info!("response: {:?}", send_response);

    Ok(())
}
