//! WalletConnect v2 relay client implementation.
//!
//! This module provides a concrete implementation of [`WalletConnectSession`]
//! that connects to the WalletConnect relay network and communicates with
//! Algorand wallets like Pera.
//!
//! # Protocol Overview
//!
//! WalletConnect v2 uses:
//! - WebSocket connection to the relay server (`wss://relay.walletconnect.com`)
//! - X25519 key exchange for session key derivation
//! - ChaCha20-Poly1305 symmetric encryption for message payloads
//! - JSON-RPC 2.0 for request/response framing
//!
//! # Usage
//!
//! ```ignore
//! use algonaut_walletconnect::relay::WalletConnectRelay;
//!
//! // Create a relay client with your WalletConnect project ID
//! let relay = WalletConnectRelay::new("your-project-id").await?;
//!
//! // Get the pairing URI for QR code / deep link
//! let uri = relay.pairing_uri();
//! println!("Scan this with Pera: {}", uri);
//!
//! // Wait for the wallet to connect
//! let address = relay.wait_for_session().await?;
//! println!("Connected to: {}", address);
//!
//! // Use the relay as a WalletConnectSession
//! let signer = PeraSigner::new(address, Arc::new(relay));
//! ```

mod auth;
mod client;
mod crypto;
mod error;
mod messages;
mod pairing;
mod session;

pub use client::WalletConnectRelay;
pub use error::RelayError;
pub use pairing::PairingUri;
pub use session::SessionProposalConfig;
