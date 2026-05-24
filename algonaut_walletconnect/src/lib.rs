//! WalletConnect integration for algonaut.
//!
//! This crate provides a [`Signer`] implementation for WalletConnect-compatible
//! wallets like Pera Wallet. It handles the ARC-0001 `algo_signTxn` codec and
//! delegates the actual WalletConnect transport to an injected session trait.
//!
//! # Architecture
//!
//! The crate is structured as follows:
//!
//! - [`WalletConnectSession`] - trait for the transport layer (injected by caller)
//! - [`WalletConnectSigner`] - generic signer over any session implementation
//! - [`PeraSigner`] - thin preset configured for Pera Wallet
//! - [`codec`] - ARC-0001 `WalletTransaction` encoding/decoding
//!
//! # Example
//!
//! ```ignore
//! use algonaut_walletconnect::{PeraSigner, WalletConnectSession};
//! use std::sync::Arc;
//!
//! // Implement WalletConnectSession for your transport
//! struct MySession { /* ... */ }
//! impl WalletConnectSession for MySession { /* ... */ }
//!
//! // Create a PeraSigner with the connected address and session
//! let signer = PeraSigner::new(connected_address, Arc::new(my_session));
//!
//! // Use with the atomic transaction composer
//! let alice_signer: Arc<dyn Signer> = Arc::new(signer);
//! ```

pub mod codec;
pub mod error;
pub mod session;
pub mod signer;

pub use codec::{SignRequest, SignedTxnResponse, WalletTransaction};
pub use error::WalletConnectError;
pub use session::{SessionFuture, WalletConnectSession};
pub use signer::{PeraSigner, WalletConnectSigner};
