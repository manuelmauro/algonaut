//! Compose and run a multi-transaction atomic group with the Atomic
//! Transaction Composer.
//!
//! The group bundles three operations, committed all-or-nothing:
//!   1. a payment from alice to bob (signed by alice),
//!   2. a payment from bob to alice (signed by bob), and
//!   3. an ARC-4 `add(uint64,uint64)uint64` method call on an app
//!      (signed by alice).
//!
//! It walks the full typestate chain
//! `AtomicGroupBuilder -> UnsignedAtomicGroup -> SignedAtomicGroup`,
//! dry-runs the group with a non-destructive `simulate` *before* signing,
//! then `sign()`s and `execute()`s the very same group and decodes the
//! method call's ABI return value.
//!
//! Contrast with `examples/atomic_swap.rs`, which assembles and signs an
//! atomic group by hand with `TxGroup`; the composer also handles ABI
//! argument encoding, group-id assignment, per-transaction signers, and
//! return-value decoding for you.

use algonaut::algod::v2::Algod;
use algonaut::atomic_transaction_composer::{
    AbiArgValue, AtomicGroupBuilder, MethodCall, TransactionWithSigner,
};
use algonaut::core::{AppId, MicroAlgos};
use algonaut::transaction::account::Account;
use algonaut::transaction::{Pay, Signer};
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

    info!("creating accounts for alice and bob");
    let alice = Account::from_mnemonic(&env::var("ALICE_MNEMONIC")?)?;
    let bob = Account::from_mnemonic(&env::var("BOB_MNEMONIC")?)?;

    // A `Signer` is shared as `Arc<dyn Signer>`, so alice can authorize
    // both her payment and her method call from the same handle.
    let alice_signer: Arc<dyn Signer> = Arc::new(alice.clone());
    let bob_signer: Arc<dyn Signer> = Arc::new(bob.clone());

    info!("retrieving suggested params");
    let params = algod.txn_params().await?;

    // TODO point this at a real ARC-4 method on a deployed contract.
    // `add(uint64,uint64)uint64` takes two unsigned-64 args, returns one.
    let app_id = AppId(123);
    let method = AbiMethod::from_signature("add(uint64,uint64)uint64")?;

    info!("building the two payment legs");
    let alice_to_bob =
        Pay::new(alice.address(), bob.address(), MicroAlgos(1_000)).build(&params)?;
    let bob_to_alice =
        Pay::new(bob.address(), alice.address(), MicroAlgos(1_000)).build(&params)?;

    info!("building the method call add(2, 3)");
    let call = MethodCall::new(app_id, method, alice.address(), alice_signer.clone())
        .args(vec![
            AbiArgValue::AbiValue(AbiValue::Int(BigUint::from(2u64))),
            AbiArgValue::AbiValue(AbiValue::Int(BigUint::from(3u64))),
        ])
        .build(&params);

    info!("composing the atomic group");
    let unsigned = AtomicGroupBuilder::new()
        .add_transaction(TransactionWithSigner::new(alice_to_bob, alice_signer))
        .add_transaction(TransactionWithSigner::new(bob_to_alice, bob_signer))
        .add_method_call(call)
        .build()?;

    // `simulate` borrows the group (`&self`), so we can dry-run it first
    // and still sign + execute the very same value afterwards.
    info!("simulating (dry run) before signing");
    let sim = unsigned.simulate(&algod).await?;
    info!(
        "simulate ok: {} transaction(s), {} method result(s)",
        sim.tx_ids.len(),
        sim.method_results.len()
    );

    // Sign every transaction with its own signer, then submit and wait for
    // confirmation. (For fire-and-forget, `unsigned.sign().await?.submit(&algod)`
    // returns a `PendingSubmission` handle you can `.confirm()` later.)
    info!("signing and executing the group");
    let outcome = unsigned.sign().await?.execute(&algod).await?;
    info!("group confirmed in round {:?}", outcome.confirmed_round);
    for (i, result) in outcome.method_results.iter().enumerate() {
        info!("method call {i} returned: {:?}", result.return_value);
    }

    Ok(())
}
