//! # algorand-rs
//!
//! This crate is a WORK IN PROGRESS!
//!
//! **algorand-rs** aims at becoming a rusty algorand sdk.
//!
//! ```rust
//! use algorand_rs::Algod;
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

pub mod client;
pub mod core;
pub mod crypto;
pub(crate) mod encoding;
pub mod error;
pub mod transaction;

// Re-exports
pub use crate::core::{MicroAlgos, Round};
pub use client::algod::Algod;
pub use client::indexer::Indexer;
pub use client::kmd::Kmd;
