//! Call an ARC-4 method on a deployed application via the
//! [`GroupBuilder`] typestate chain and the fluent [`MethodCall`]
//! builder. This is the recommended path for application calls.

use algonaut::algod::v2::Algod;
use algonaut::atomic_transaction_composer::{AbiArgValue, GroupBuilder, MethodCall};
use algonaut::core::AppId;
use algonaut::transaction::account::Account;
use algonaut_abi::{abi_interactions::AbiMethod, abi_type::AbiValue};
use dotenv::dotenv;
use num_bigint::BigUint;
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
    let params = algod.txn_params().await?;

    // TODO point this at a real ARC-4 method on a real deployed contract.
    // The signature here is illustrative — `add(uint64,uint64)uint64`
    // takes two unsigned-64 arguments and returns one.
    let method = AbiMethod::from_signature("add(uint64,uint64)uint64")?;

    info!("building method call");
    let call = MethodCall::new(AppId(5), method, alice.address(), signer)
        .args(vec![
            AbiArgValue::AbiValue(AbiValue::Int(BigUint::from(2u64))),
            AbiArgValue::AbiValue(AbiValue::Int(BigUint::from(3u64))),
        ])
        .build(&params);

    info!("composing and executing");
    let result = GroupBuilder::new()
        .add_method_call(call)
        .build()?
        .sign()?
        .execute(&algod)
        .await?;
    info!("confirmed in round {:?}", result.confirmed_round);
    for r in result.method_results {
        info!("method return: {:?}", r.return_value);
    }

    Ok(())
}
