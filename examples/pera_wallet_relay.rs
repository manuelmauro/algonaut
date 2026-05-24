//! Pera Wallet signing example using the built-in WalletConnect relay.
//!
//! This example demonstrates how to use the `algonaut_walletconnect` crate's
//! built-in relay client to sign transactions with Pera Wallet.
//!
//! # Architecture
//!
//! The `relay` feature provides a batteries-included WalletConnect v2 client
//! that handles:
//! - WebSocket connection to `wss://relay.walletconnect.com`
//! - X25519 key exchange for session encryption
//! - ChaCha20-Poly1305 message encryption
//! - Pairing URI generation (for QR codes and deep links)
//! - Session establishment and management
//!
//! # Prerequisites
//!
//! 1. Get a WalletConnect Cloud project ID from https://cloud.walletconnect.com
//! 2. Install Pera Wallet on your mobile device
//!
//! # Running
//!
//! ```bash
//! WALLETCONNECT_PROJECT_ID=your-project-id cargo run --example pera_wallet_relay --features walletconnect,algod
//! ```

use algonaut::atomic::{AtomicGroupBuilder, TransactionWithSigner};
use algonaut::core::{MicroAlgos, Round};
use algonaut::crypto::HashDigest;
use algonaut::model::algod::SuggestedParams;
use algonaut::transaction::account::Account;
use algonaut::transaction::{Pay, Signer};
use algonaut::walletconnect::{PeraSigner, WalletConnectRelay};
use std::error::Error;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🔷 Pera Wallet Signing Example (with Relay)\n");

    // Get project ID from environment
    let project_id = std::env::var("WALLETCONNECT_PROJECT_ID")
        .expect("Set WALLETCONNECT_PROJECT_ID environment variable");

    println!("Connecting to WalletConnect relay...");

    // Create the relay client
    let relay = Arc::new(WalletConnectRelay::new(project_id).await?);

    // Get pairing URI for QR code / deep link
    let pairing_uri = relay.pairing_uri();

    println!("\n📱 Scan with Pera Wallet or open deep link:\n");
    println!("URI: {}\n", pairing_uri.to_uri());
    println!("{}", pairing_uri.to_qr_string());
    println!("\nPera deep link: {}\n", pairing_uri.to_pera_deeplink());

    // Wait for wallet to connect
    println!("Waiting for wallet connection...");
    let connected_address = relay.wait_for_session().await?;

    println!("✅ Connected! Address: {}\n", connected_address);

    // Create the PeraSigner with the relay
    let pera_signer = PeraSigner::new(connected_address, relay);
    let alice_signer: Arc<dyn Signer> = Arc::new(pera_signer);

    // Bob uses a local account for this example
    let bob = Account::generate();
    let bob_signer: Arc<dyn Signer> = Arc::new(bob.clone());

    println!("Addresses:");
    println!("  Alice (Pera): {}", connected_address);
    println!("  Bob (local): {}\n", bob.address());

    // Mock transaction parameters (in production: use algod.suggested_params())
    let params = SuggestedParams {
        genesis_id: "testnet-v1.0".to_string(),
        genesis_hash: HashDigest([0u8; 32]),
        consensus_version: "".to_string(),
        fee: MicroAlgos(0),
        min_fee: MicroAlgos(1000),
        last_round: Round(1000),
    };

    // Build an atomic swap
    println!("Building atomic swap transaction group...\n");

    let alice_to_bob =
        Pay::new(connected_address, bob.address(), MicroAlgos(1_000_000)).build(&params)?;

    let bob_to_alice =
        Pay::new(bob.address(), connected_address, MicroAlgos(500_000)).build(&params)?;

    // Compose the atomic group
    let unsigned = AtomicGroupBuilder::new()
        .add_transaction(TransactionWithSigner::new(
            alice_to_bob,
            alice_signer.clone(),
        ))
        .add_transaction(TransactionWithSigner::new(bob_to_alice, bob_signer))
        .build()?;

    println!(
        "Atomic group built with {} transactions",
        unsigned.transactions().len()
    );

    // Sign the group
    println!("\nRequesting signature from Pera Wallet...\n");
    println!("📱 Please approve the transaction in your wallet\n");

    let signed = unsigned.sign().await?;

    println!("✅ All transactions signed!");
    println!(
        "   Signed transaction count: {}",
        signed.signed_transactions().len()
    );

    for (i, stx) in signed.signed_transactions().iter().enumerate() {
        println!("   Transaction {}: {}", i, stx.transaction_id());
    }

    // In production, you would submit this to algod:
    // let outcome = signed.execute(&algod).await?;
    // println!("Confirmed in round: {:?}", outcome.confirmed_round);

    println!("\n🎉 Example complete!");

    Ok(())
}
