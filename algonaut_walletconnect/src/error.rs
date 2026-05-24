//! Error types for WalletConnect signing operations.

use algonaut_core::Address;
use thiserror::Error;

/// Errors that can occur during WalletConnect signing operations.
#[derive(Debug, Error)]
pub enum WalletConnectError {
    /// The user rejected the signing request in their wallet.
    #[error("user rejected the signing request")]
    UserRejected,

    /// The wallet session has expired or disconnected.
    #[error("wallet session expired or disconnected")]
    SessionExpired,

    /// The wallet returned an unexpected number of signed transactions.
    #[error("expected {expected} signed transactions, got {actual}")]
    SignedCountMismatch { expected: usize, actual: usize },

    /// The wallet returned a transaction that doesn't match the expected ID.
    #[error(
        "signed transaction at index {index} has unexpected ID: expected {expected}, got {actual}"
    )]
    TransactionIdMismatch {
        index: usize,
        expected: String,
        actual: String,
    },

    /// The transaction sender doesn't match the connected wallet address.
    #[error("transaction sender {sender} doesn't match connected address {connected}")]
    SenderMismatch { sender: Address, connected: Address },

    /// Failed to decode the signed transaction returned by the wallet.
    #[error("failed to decode signed transaction: {0}")]
    DecodingError(String),

    /// Failed to encode the signing request.
    #[error("failed to encode signing request: {0}")]
    EncodingError(String),

    /// The WalletConnect transport returned an error.
    #[error("transport error: {0}")]
    Transport(String),

    /// JSON-RPC error returned by the wallet.
    #[error("JSON-RPC error {code}: {message}")]
    JsonRpc { code: i64, message: String },

    /// The wallet doesn't support the requested method.
    #[error("method not supported: {0}")]
    MethodNotSupported(String),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// MessagePack deserialization error.
    #[error("msgpack deserialization error: {0}")]
    MsgpackDecode(#[from] rmp_serde::decode::Error),
}
