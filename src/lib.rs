//! # algorand-rs
//!
//! This crate is a WORK IN PROGRESS!
//!
//! **algorand-rs** aims at becoming a rusty algorand sdk.
//!
//! ```rust
//! use algorand_rs::algod::Algod;
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
//! - [ ] Examples guiding API development
//! - [ ] Async requests
//! - [ ] Builder pattern and sensible defaults
//! - [ ] Clear error messages
//! - [ ] Thorough test suite
//! - [ ] Comprehensive documentation

// TODO #![deny(missing_docs)]

pub mod account;
/// Algorand protocol daemon
pub mod algod;
pub mod auction;
pub mod crypto;
pub mod error;
/// Key management daemon
pub mod kmd;

/// Algorand's indexer
pub mod indexer;

/// Support for turning 32 byte keys into human-readable mnemonics and back
pub mod mnemonic;
pub mod models;
pub mod transaction;

pub(crate) mod util;

// Re-exports
pub use algod::Algod;
pub use crypto::Address;
pub use indexer::Indexer;
pub use kmd::Kmd;
pub use models::{HashDigest, MasterDerivationKey, MicroAlgos, Round};
