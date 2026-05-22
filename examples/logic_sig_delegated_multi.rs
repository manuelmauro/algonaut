use algonaut::algod::v2::{Algod, SourceMap};
use algonaut::core::{LogicSignature, MicroAlgos, MultisigAddress};
use algonaut::transaction::Pay;
use algonaut::transaction::account::Account;
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
        .teal_compile(
            r#"
#pragma version 3
int 1
"#
            .as_bytes(),
            SourceMap::Skip,
        )
        .await?;

    info!("creating account for alice");
    let alice = Account::from_mnemonic(&env::var("ALICE_MNEMONIC")?)?;

    info!("creating account for bob");
    let bob = Account::from_mnemonic(&env::var("BOB_MNEMONIC")?)?;

    info!("creating account for casey");
    let casey = env::var("CASEY_ADDRESS")?.parse()?;

    info!("creating multisig address");
    let multisig_address = MultisigAddress::new(1, 2, &[alice.address(), bob.address()])?;

    info!("retrieving suggested params");
    let params = algod.suggested_params().await?;

    info!("building Pay transaction");
    let t = Pay::new(multisig_address.address(), casey, MicroAlgos(123_456)).build(&params)?;

    info!("alice is initializing multi-signature");
    let msig = alice.init_logic_msig(&program, &multisig_address)?;

    info!("bob is appending to multi-signature");
    let msig = bob.append_to_logic_msig(&program, msig)?;

    info!("signing transaction");
    // the transaction will fail because the multisig address has no funds
    let signed_t = SignedLogic {
        logic: program,
        args: vec![],
        sig: LogicSignature::DelegatedMultiSig(msig),
    }
    .sign(t)?;

    info!("broadcasting transaction");
    let send_response = algod.send(&signed_t).await;
    info!("response: {:?}", send_response);

    Ok(())
}
