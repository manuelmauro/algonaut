//! Call an ARC-4 method on a deployed application via the
//! [`AtomicGroupBuilder`] typestate chain and the fluent [`MethodCall`]
//! builder. This is the recommended path for application calls.

use algonaut::abi::abi_call;
use algonaut::algod::v2::Algod;
use algonaut::atomic::{AtomicGroupBuilder, MethodCall};
use algonaut::core::AppId;
use algonaut::transaction::account::Account;
use dotenv::dotenv;
use std::env;
use std::error::Error;
use std::sync::Arc;
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
    let signer = Arc::new(alice.clone());

    info!("retrieving suggested params");
    let params = algod.suggested_params().await?;

    // TODO point this at a real ARC-4 method on a real deployed contract.
    // The signature here is illustrative — `add(uint64,uint64)uint64`
    // takes two unsigned-64 arguments and returns one. `abi_call!` validates
    // the signature and type-checks the two `u64` arguments at compile time,
    // so there is no `?` here: a typo or a wrong argument is a build error.
    info!("building method call");
    let call = MethodCall::builder(AppId(5), alice.address(), signer)
        .invoke(abi_call!("add(uint64,uint64)uint64", 2u64, 3u64))
        .build(&params);

    info!("composing and executing");
    let result = AtomicGroupBuilder::new()
        .add_method_call(call)
        .build()?
        .sign()
        .await?
        .execute(&algod)
        .await?;
    info!("confirmed in round {:?}", result.confirmed_round);
    for r in result.method_results {
        info!("method return: {:?}", r.return_value);
    }

    Ok(())
}
