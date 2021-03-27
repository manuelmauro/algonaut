//! # algonaut
//!
//! This crate is a WORK IN PROGRESS!
//!
//! **algonaut** aims at becoming a rusty algorand sdk.
//!
//! ```rust
//! use algonaut::Algod;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let algod = Algod::new()
//!         .bind("http://localhost:4001")
//!         .auth("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
//!         .client_v1()?;
//!
//!     println!("Algod versions: {:?}", algod.versions()?.versions);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Objectives
//!
//! - [ ] Example-driven API development
//! - [ ] Clear error messages
//! - [ ] Async requests
//! - [ ] Builder pattern and sensible defaults
//! - [ ] Thorough test suite
//! - [ ] Comprehensive documentation

// TODO #![deny(missing_docs)]

// Re-exports
pub use algonaut_client::algod::Algod;
pub use algonaut_client::indexer::Indexer;
pub use algonaut_client::kmd::Kmd;
pub use algonaut_core as core;
pub use algonaut_crypto as crypto;
pub use algonaut_transaction as transaction;
