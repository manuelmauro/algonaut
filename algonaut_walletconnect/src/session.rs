//! WalletConnect session abstraction.
//!
//! This module defines the [`WalletConnectSession`] trait that abstracts
//! the transport layer. Algonaut owns the ARC-0001 codec; the caller
//! supplies the websocket/pairing client implementation.

use std::future::Future;
use std::pin::Pin;

use crate::codec::{SignedTxnResponse, WalletTransaction};
use crate::error::WalletConnectError;

/// Future type for async session methods.
pub type SessionFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, WalletConnectError>> + Send + 'a>>;

/// Abstraction over the WalletConnect transport layer.
///
/// Implementations handle the websocket relay, pairing, and JSON-RPC
/// communication. This trait exposes only the signing operation,
/// keeping the heavy relay/JSON-RPC surface out of algonaut.
///
/// # Implementors
///
/// Implement this trait against your preferred WalletConnect client library.
/// The trait is object-safe and `Send + Sync` to support async executors.
///
/// # Example
///
/// ```ignore
/// use algonaut_walletconnect::{WalletConnectSession, WalletTransaction, SignedTxnResponse};
/// use std::sync::Arc;
///
/// struct MyWcSession {
///     // Your WalletConnect client state
/// }
///
/// impl WalletConnectSession for MyWcSession {
///     fn sign_transactions<'a>(
///         &'a self,
///         transactions: Vec<WalletTransaction>,
///     ) -> SessionFuture<'a, Vec<SignedTxnResponse>> {
///         Box::pin(async move {
///             // Send algo_signTxn JSON-RPC request via your relay
///             // Parse and return the response
///             todo!()
///         })
///     }
/// }
/// ```
pub trait WalletConnectSession: std::fmt::Debug + Send + Sync {
    /// Send an `algo_signTxn` request to the connected wallet.
    ///
    /// # Arguments
    ///
    /// * `transactions` - ARC-0001 encoded transaction array
    ///
    /// # Returns
    ///
    /// The wallet's response: signed transactions for owned slots,
    /// `null` for display-only slots.
    fn sign_transactions<'a>(
        &'a self,
        transactions: Vec<WalletTransaction>,
    ) -> SessionFuture<'a, Vec<SignedTxnResponse>>;
}
