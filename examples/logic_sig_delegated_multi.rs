use algonaut::algod::v2::Algod;
use algonaut::core::{LogicSignature, MicroAlgos, MultisigAddress};
use algonaut::transaction::transaction::TransactionSignature;
use algonaut::transaction::{account::Account, TxnBuilder};
use algonaut::transaction::{Pay, SignedTransaction};
use algonaut_core::CompiledTeal;
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
"#,
            None,
        )
        .await?;

    info!("creating account for alice");
    let alice = Account::from_mnemonic(&env::var("ALICE_MNEMONIC")?)?;

    info!("creating account for bob");
    let bob = Account::from_mnemonic(&env::var("BOB_MNEMONIC")?)?;

    info!("creating account for casey");
    let casey = (&env::var("CASEY_ADDRESS")?).parse()?;

    info!("creating multisig address");
    let multisig_address = MultisigAddress::new(1, 2, &[alice.address(), bob.address()])?;

    info!("retrieving suggested params");
    let params = algod.transaction_params().await?;

    info!("building Pay transaction");
    let t = TxnBuilder::with(
        &params,
        Pay::new(multisig_address.address(), casey, MicroAlgos(123_456)).build(),
    )
    .build()?;

    info!("alice is initializing multi-signature");
    let msig = alice.init_logic_msig(
        &CompiledTeal(program.clone().result.into_bytes()),
        &multisig_address,
    )?;

    info!("bob is appending to multi-signature");
    let msig =
        bob.append_to_logic_msig(&CompiledTeal(program.clone().result.into_bytes()), msig)?;

    info!("building logic signature");
    let sig = TransactionSignature::Logic(SignedLogic {
        logic: CompiledTeal(program.result.into_bytes()),
        args: vec![],
        sig: LogicSignature::DelegatedMultiSig(msig),
    });

    info!("signing transaction");
    let signed_t = SignedTransaction {
        transaction: t,
        transaction_id: "".to_owned(),
        sig,
        auth_address: None,
    };

    info!("broadcasting transaction");
    // the transaction will fail because the multisig address has no funds
    let send_response = algod.signed_transaction(&signed_t).await;
    info!("response: {:?}", send_response);

    Ok(())
}
