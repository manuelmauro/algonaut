//! Pera Wallet signing example with WalletConnect.
//!
//! This example demonstrates how to use the `algonaut_walletconnect` crate
//! to sign transactions with Pera Wallet via WalletConnect.
//!
//! # Architecture
//!
//! The WalletConnect integration is split into two parts:
//!
//! 1. **algonaut_walletconnect** (this crate) - Owns the ARC-0001 codec and
//!    provides the `PeraSigner` which implements the `Signer` trait.
//!
//! 2. **Your WalletConnect client** - You implement `WalletConnectSession`
//!    against your preferred WalletConnect library (e.g., walletconnect-rs,
//!    or a custom implementation).
//!
//! This separation keeps the heavy relay/JSON-RPC surface out of algonaut
//! while giving you full control over the transport layer.
//!
//! # Running
//!
//! This example uses a mock session that simulates wallet signing. In a real
//! application, you would replace `MockPeraSession` with your WalletConnect
//! implementation.
//!
//! ```bash
//! cargo run --example pera_wallet --features walletconnect,algod
//! ```

use algonaut::atomic::{AtomicGroupBuilder, TransactionWithSigner};
use algonaut::core::{Address, MicroAlgos, Round, ToMsgPack};
use algonaut::crypto::HashDigest;
use algonaut::model::algod::SuggestedParams;
use algonaut::transaction::account::Account;
use algonaut::transaction::{Pay, Signer};
use algonaut::walletconnect::{
    PeraSigner, SessionFuture, SignedTxnResponse, WalletConnectSession, WalletTransaction,
};
use data_encoding::BASE64;
use std::error::Error;
use std::sync::Arc;

/// Mock Pera wallet session for demonstration.
///
/// In a real application, this would be your WalletConnect v2 client
/// implementation that connects to the Pera Wallet relay.
#[derive(Debug)]
struct MockPeraSession {
    /// The connected wallet address (the account that will sign).
    connected_address: Address,
    /// The signing key (in real usage, this lives in the wallet, not here).
    /// We use this to simulate what the wallet would return.
    signing_account: Account,
}

impl MockPeraSession {
    fn new(account: Account) -> Self {
        Self {
            connected_address: account.address(),
            signing_account: account,
        }
    }
}

impl WalletConnectSession for MockPeraSession {
    fn sign_transactions<'a>(
        &'a self,
        transactions: Vec<WalletTransaction>,
    ) -> SessionFuture<'a, Vec<SignedTxnResponse>> {
        Box::pin(async move {
            println!(
                "📱 Pera Wallet received signing request for {} transaction(s)",
                transactions.len()
            );

            let mut responses = Vec::with_capacity(transactions.len());

            for (i, wallet_tx) in transactions.iter().enumerate() {
                // Check if this transaction should be signed
                let should_sign = match &wallet_tx.signers {
                    Some(signers) if signers.is_empty() => {
                        // Display-only: signers is empty array
                        println!("   Transaction {i}: display-only (not signing)");
                        false
                    }
                    Some(signers) => {
                        // Should sign if our address is in signers
                        let my_addr = self.connected_address.to_string();
                        let is_ours = signers.contains(&my_addr);
                        if is_ours {
                            println!("   Transaction {i}: signing ✓");
                        }
                        is_ours
                    }
                    None => {
                        // No signers field: wallet decides based on sender
                        println!("   Transaction {i}: auto-detect signing");
                        true
                    }
                };

                if should_sign {
                    // Decode the unsigned transaction
                    let txn_bytes = BASE64.decode(wallet_tx.txn.as_bytes()).map_err(|e| {
                        algonaut::walletconnect::WalletConnectError::DecodingError(e.to_string())
                    })?;

                    let unsigned_tx: algonaut::transaction::transaction::Transaction =
                        rmp_serde::from_slice(&txn_bytes).map_err(|e| {
                            algonaut::walletconnect::WalletConnectError::MsgpackDecode(e)
                        })?;

                    // Sign with our account
                    let signed = self.signing_account.sign(unsigned_tx).map_err(|e| {
                        algonaut::walletconnect::WalletConnectError::EncodingError(e.to_string())
                    })?;

                    // Encode as base64
                    let signed_bytes = signed.to_msg_pack().map_err(|e| {
                        algonaut::walletconnect::WalletConnectError::EncodingError(e.to_string())
                    })?;

                    responses.push(SignedTxnResponse::Signed(BASE64.encode(&signed_bytes)));
                } else {
                    // Display-only slot
                    responses.push(SignedTxnResponse::Null);
                }
            }

            println!("📱 Pera Wallet approved and signed!");
            Ok(responses)
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🔷 Pera Wallet Signing Example\n");

    // In a real app, you would:
    // 1. Initialize your WalletConnect client
    // 2. Display a QR code or deep link for the user to scan with Pera
    // 3. Handle the pairing and session establishment
    // 4. Get the connected address from the session

    // For this example, we create a mock session with a test account
    let alice = Account::generate();
    let bob = Account::generate();

    println!("Test addresses:");
    println!("  Alice (connected to Pera): {}", alice.address());
    println!("  Bob: {}\n", bob.address());

    // Create the mock Pera session
    // In production: this would be your real WalletConnect session
    let pera_session = Arc::new(MockPeraSession::new(alice.clone()));

    // Create the PeraSigner with the connected address and session
    let pera_signer = PeraSigner::new(alice.address(), pera_session);
    let alice_signer: Arc<dyn Signer> = Arc::new(pera_signer);

    // Bob uses a local account for this example
    let bob_signer: Arc<dyn Signer> = Arc::new(bob.clone());

    // Mock transaction parameters (in production: use algod.suggested_params())
    let params = SuggestedParams {
        genesis_id: "testnet-v1.0".to_string(),
        genesis_hash: HashDigest([0u8; 32]),
        consensus_version: "".to_string(),
        fee: MicroAlgos(0),
        min_fee: MicroAlgos(1000),
        last_round: Round(1000),
    };

    // Build an atomic swap: Alice sends 1 ALGO to Bob, Bob sends 0.5 ALGO to Alice
    println!("Building atomic swap transaction group...\n");

    let alice_to_bob =
        Pay::new(alice.address(), bob.address(), MicroAlgos(1_000_000)).build(&params)?;

    let bob_to_alice =
        Pay::new(bob.address(), alice.address(), MicroAlgos(500_000)).build(&params)?;

    // Compose the atomic group
    // Note: alice_signer is Arc<dyn Signer> backed by PeraSigner
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
    // This will call each signer once. The PeraSigner sends all its transactions
    // to the Pera Wallet in a single algo_signTxn request (one approval prompt).
    println!("\nSigning transactions...\n");

    let signed = unsigned.sign().await?;

    println!("\n✅ All transactions signed!");
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
    println!("\nIn production, you would:");
    println!("  1. Replace MockPeraSession with your WalletConnect v2 client");
    println!("  2. Use algod.suggested_params() for transaction parameters");
    println!("  3. Call signed.execute(&algod).await? to submit to the network");

    Ok(())
}
