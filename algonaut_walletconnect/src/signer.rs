//! WalletConnect-based [`Signer`] implementations.
//!
//! This module provides [`WalletConnectSigner`], a generic signer over any
//! [`WalletConnectSession`] implementation, and [`PeraSigner`], a thin
//! preset configured for Pera Wallet.

use std::sync::Arc;

use algonaut_core::Address;
use algonaut_transaction::Signer;
use algonaut_transaction::error::TransactionError;
use algonaut_transaction::signer::{SigningFuture, SigningRequest};
use algonaut_transaction::transaction::SignedTransaction;
use data_encoding::BASE64;

use crate::codec::{SignRequest, SignedTxnResponse};
use crate::error::WalletConnectError;
use crate::session::WalletConnectSession;

/// Generic WalletConnect-based signer.
///
/// Holds a connected wallet address and an injected session. The session
/// handles the WalletConnect relay; this signer handles the ARC-0001 codec
/// and validation.
///
/// Use [`PeraSigner`] for a Pera-specific preset, or use this directly
/// for other WalletConnect-compatible wallets.
#[derive(Debug)]
pub struct WalletConnectSigner<S: WalletConnectSession> {
    /// The address of the connected wallet account.
    address: Address,
    /// The WalletConnect session (injected by caller).
    session: Arc<S>,
}

impl<S: WalletConnectSession> WalletConnectSigner<S> {
    /// Create a new signer with the given connected address and session.
    pub fn new(address: Address, session: Arc<S>) -> Self {
        Self { address, session }
    }

    /// The connected wallet address.
    pub fn address(&self) -> Address {
        self.address
    }

    /// The underlying session.
    pub fn session(&self) -> &Arc<S> {
        &self.session
    }
}

impl<S: WalletConnectSession + 'static> Signer for WalletConnectSigner<S> {
    fn sign_transactions<'a>(&'a self, request: SigningRequest<'a>) -> SigningFuture<'a> {
        Box::pin(async move {
            // Pre-flight: verify all requested transactions are from our address
            for &idx in request.indexes {
                let sender = request.transactions[idx].sender();
                if sender != self.address {
                    return Err(TransactionError::Signer(Box::new(
                        WalletConnectError::SenderMismatch {
                            sender,
                            connected: self.address,
                        },
                    )));
                }
            }

            // Encode the request per ARC-0001
            let sign_request = SignRequest {
                transactions: request.transactions.to_vec(),
                indexes_to_sign: request.indexes.to_vec(),
                signer_address: self.address,
                group_message: None,
            };

            let wallet_txns = sign_request
                .encode()
                .map_err(|e| TransactionError::Signer(Box::new(e)))?;

            // Send to wallet via session
            let responses = self
                .session
                .sign_transactions(wallet_txns)
                .await
                .map_err(|e| TransactionError::Signer(Box::new(e)))?;

            // Extract signed transactions for our indexes
            let mut signed = Vec::with_capacity(request.indexes.len());

            for &idx in request.indexes {
                let response = responses.get(idx).ok_or_else(|| {
                    TransactionError::Signer(Box::new(WalletConnectError::SignedCountMismatch {
                        expected: request.transactions.len(),
                        actual: responses.len(),
                    }))
                })?;

                let signed_b64 = match response {
                    SignedTxnResponse::Signed(s) => s,
                    SignedTxnResponse::Null => {
                        return Err(TransactionError::Signer(Box::new(
                            WalletConnectError::DecodingError(format!(
                                "expected signed transaction at index {idx}, got null"
                            )),
                        )));
                    }
                };

                // Decode the signed transaction
                let signed_bytes = BASE64.decode(signed_b64.as_bytes()).map_err(|e| {
                    TransactionError::Signer(Box::new(WalletConnectError::DecodingError(
                        e.to_string(),
                    )))
                })?;

                let signed_tx: SignedTransaction =
                    rmp_serde::from_slice(&signed_bytes).map_err(|e| {
                        TransactionError::Signer(Box::new(WalletConnectError::MsgpackDecode(e)))
                    })?;

                // Validate the transaction ID matches
                let expected_id = request.transactions[idx].id().map_err(|e| {
                    TransactionError::Signer(Box::new(WalletConnectError::EncodingError(
                        e.to_string(),
                    )))
                })?;

                if signed_tx.transaction_id() != &expected_id {
                    return Err(TransactionError::Signer(Box::new(
                        WalletConnectError::TransactionIdMismatch {
                            index: idx,
                            expected: expected_id.0,
                            actual: signed_tx.transaction_id().0.clone(),
                        },
                    )));
                }

                signed.push(signed_tx);
            }

            Ok(signed)
        })
    }
}

/// Pera Wallet signer preset.
///
/// A thin wrapper over [`WalletConnectSigner`] configured for Pera Wallet.
/// Pera uses standard WalletConnect v2 with ARC-0001 `algo_signTxn`.
///
/// # One-Prompt Contract
///
/// Per the ADR, one `Arc<PeraSigner>` equals one session equals one
/// approval prompt. When the same `Arc<PeraSigner>` is used for multiple
/// transactions in an atomic group, they all go to the wallet in a single
/// `algo_signTxn` call.
///
/// # Example
///
/// ```ignore
/// use algonaut_walletconnect::PeraSigner;
/// use algonaut::transaction::Signer;
/// use std::sync::Arc;
///
/// // After establishing a WalletConnect session with Pera
/// let pera = PeraSigner::new(connected_address, session);
/// let signer: Arc<dyn Signer> = Arc::new(pera);
///
/// // Use with atomic transaction composer
/// let group = AtomicGroupBuilder::new()
///     .add_transaction(TransactionWithSigner::new(tx, signer.clone()))
///     .build()?;
/// ```
#[derive(Debug)]
pub struct PeraSigner<S: WalletConnectSession> {
    inner: WalletConnectSigner<S>,
}

impl<S: WalletConnectSession> PeraSigner<S> {
    /// Create a new Pera signer with the connected address and session.
    pub fn new(address: Address, session: Arc<S>) -> Self {
        Self {
            inner: WalletConnectSigner::new(address, session),
        }
    }

    /// The connected Pera wallet address.
    pub fn address(&self) -> Address {
        self.inner.address()
    }

    /// The underlying WalletConnect session.
    pub fn session(&self) -> &Arc<S> {
        self.inner.session()
    }
}

impl<S: WalletConnectSession + 'static> Signer for PeraSigner<S> {
    fn sign_transactions<'a>(&'a self, request: SigningRequest<'a>) -> SigningFuture<'a> {
        self.inner.sign_transactions(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::SessionFuture;
    use algonaut_core::{MicroAlgos, Round};
    use algonaut_crypto::HashDigest;
    use algonaut_model::algod::SuggestedParams;
    use algonaut_transaction::Pay;
    use algonaut_transaction::account::Account;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Mock session that counts sign_transactions calls.
    #[derive(Debug)]
    struct MockSession {
        calls: AtomicUsize,
        /// Pre-configured responses (base64 signed txns or null).
        responses: Vec<SignedTxnResponse>,
    }

    impl MockSession {
        fn new(responses: Vec<SignedTxnResponse>) -> Self {
            Self {
                calls: AtomicUsize::new(0),
                responses,
            }
        }

        fn call_count(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    impl WalletConnectSession for MockSession {
        fn sign_transactions<'a>(
            &'a self,
            _transactions: Vec<crate::codec::WalletTransaction>,
        ) -> SessionFuture<'a, Vec<SignedTxnResponse>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let responses = self.responses.clone();
            Box::pin(async move { Ok(responses) })
        }
    }

    fn mock_params() -> SuggestedParams {
        SuggestedParams {
            genesis_id: "testnet-v1.0".to_string(),
            genesis_hash: HashDigest([0u8; 32]),
            consensus_version: "".to_string(),
            fee: MicroAlgos(0),
            min_fee: MicroAlgos(1000),
            last_round: Round(1000),
        }
    }

    #[tokio::test]
    async fn test_sender_mismatch_rejected() {
        let alice = Account::generate();
        let bob = Account::generate();

        let params = mock_params();
        let tx = Pay::new(alice.address(), bob.address(), MicroAlgos(1_000_000))
            .build(&params)
            .unwrap();

        // Signer is connected as Bob, but tx is from Alice
        let session = Arc::new(MockSession::new(vec![]));
        let signer = PeraSigner::new(bob.address(), session.clone());

        let request = SigningRequest {
            transactions: &[tx],
            indexes: &[0],
        };

        let result = signer.sign_transactions(request).await;
        assert!(result.is_err());

        // Session should not have been called
        assert_eq!(session.call_count(), 0);
    }
}
