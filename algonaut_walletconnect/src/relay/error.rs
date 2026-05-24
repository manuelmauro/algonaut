//! Relay-specific error types.

use thiserror::Error;

/// Errors that can occur in the WalletConnect relay.
#[derive(Debug, Error)]
pub enum RelayError {
    /// Failed to connect to the relay server.
    #[error("failed to connect to relay: {0}")]
    Connection(String),

    /// WebSocket error.
    #[error("websocket error: {0}")]
    WebSocket(String),

    /// The relay server rejected the subscription.
    #[error("subscription rejected: {0}")]
    SubscriptionRejected(String),

    /// Timeout waiting for a response.
    #[error("timeout waiting for {operation}")]
    Timeout { operation: &'static str },

    /// The wallet rejected the session proposal.
    #[error("session proposal rejected by wallet")]
    SessionRejected,

    /// No accounts were provided by the wallet.
    #[error("wallet provided no accounts")]
    NoAccounts,

    /// The wallet returned an invalid account format.
    #[error("invalid account format: {0}")]
    InvalidAccount(String),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Encryption/decryption error.
    #[error("encryption error: {0}")]
    Encryption(String),

    /// Invalid symmetric key.
    #[error("invalid symmetric key")]
    InvalidKey,

    /// The session has expired or been disconnected.
    #[error("session expired")]
    SessionExpired,

    /// JSON-RPC error from the relay or wallet.
    #[error("JSON-RPC error {code}: {message}")]
    JsonRpc { code: i64, message: String },

    /// The relay returned an unexpected response.
    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),
}

impl From<tokio_tungstenite::tungstenite::Error> for RelayError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        RelayError::WebSocket(err.to_string())
    }
}
