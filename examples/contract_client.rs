//! Generate a type-safe contract client from an ARC-4 ABI JSON file using
//! the [`contract!`] macro.
//!
//! This example demonstrates how `contract!` simplifies contract interaction
//! compared to manually using `MethodCall::builder()` with `abi_call!`. The
//! macro reads the ABI JSON at compile time and generates a struct with
//! typed methods for each ABI method.
//!
//! # Comparison
//!
//! **Before (manual):**
//! ```ignore
//! let call = MethodCall::builder(AppId(123), alice.address(), signer)
//!     .invoke(abi_call!("add(uint64,uint64)uint64", 2u64, 3u64))
//!     .build(&params);
//! ```
//!
//! **After (with contract!):**
//! ```ignore
//! let calculator = Calculator::new(AppId(123), alice.address(), signer);
//! let call = calculator.add(2u64, 3u64).build(&params);
//! ```
//!
//! The generated struct also provides network-specific constructors if the
//! ABI JSON contains a `networks` field.

use algonaut::Algod;
use algonaut::atomic::AtomicGroupBuilder;
use algonaut::core::AppId;
use algonaut::transaction::Signer;
use algonaut::transaction::account::Account;
use dotenv::dotenv;
use std::env;
use std::error::Error;
use std::sync::Arc;

#[macro_use]
extern crate log;

// Generate a typed Calculator client from the ABI JSON.
// The macro reads the file at compile time and generates:
//   - A `Calculator` struct holding app_id, sender, and signer
//   - Methods for each ABI method (add, subtract, etc.)
//   - Builder structs for each method (CalculatorAddBuilder, etc.)
//   - Network constructors (testnet, mainnet) if networks are defined
algonaut::contract!("tests/fixtures/calculator.json");

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    env_logger::init();

    info!("creating algod client");
    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    info!("creating account for alice");
    let alice = Account::from_mnemonic(&env::var("ALICE_MNEMONIC")?)?;
    // The signer must be typed as Arc<dyn Signer> to work with the generated contract client
    let signer: Arc<dyn Signer> = Arc::new(alice.clone());

    info!("retrieving suggested params");
    let params = algod.suggested_params().await?;

    // Create a Calculator client with explicit app ID.
    // The struct name comes from the "name" field in the ABI JSON.
    info!("creating calculator client");
    let calculator = Calculator::new(AppId(123), alice.address(), Arc::clone(&signer));

    // Alternatively, use a network-specific constructor if the ABI JSON
    // contains a "networks" field with known genesis hashes:
    //   let calculator = Calculator::testnet(alice.address(), signer);
    //   let calculator = Calculator::mainnet(alice.address(), signer);

    // Build a method call using the typed interface.
    // The method signature is validated at compile time from the ABI JSON.
    // Argument types are also checked — passing a String where u64 is
    // expected would be a compile error.
    info!("building add(2, 3) method call");
    let add_call = calculator.add(2u64, 3u64).build(&params);

    // Multiple calls can be composed into an atomic group.
    info!("building subtract(10, 4) method call");
    let subtract_call = Calculator::new(AppId(123), alice.address(), Arc::clone(&signer))
        .subtract(10u64, 4u64)
        .build(&params);

    // Execute the atomic group.
    info!("composing and executing atomic group");
    let result = AtomicGroupBuilder::new()
        .add_method_call(add_call)
        .add_method_call(subtract_call)
        .build()?
        .sign()
        .await?
        .execute(&algod)
        .await?;

    info!("confirmed in round {:?}", result.confirmed_round);
    for (i, r) in result.method_results.iter().enumerate() {
        info!("method {} return: {:?}", i, r.return_value);
    }

    // Other supported method types:
    // - Methods with no arguments: calculator.noop()
    // - Boolean arguments: calculator.echo_bool(true)
    // - String arguments: calculator.echo_string("hello".to_string())
    // - Byte array arguments: calculator.echo_bytes(vec![1, 2, 3])
    // - Address arguments: calculator.echo_address(alice.address())

    Ok(())
}
