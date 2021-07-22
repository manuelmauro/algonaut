//! # algonaut
//!
//! Rust **algonaut** aims at becoming a rusty SDK for [Algorand](https://www.algorand.com/).
//! Please, be aware that this crate is a work in progress.
//!
//! ## Objectives
//!
//! - Example-driven API development
//! - Async requests
//! - Builder pattern and sensible defaults
//! - Modularity
//! - Clear error messages
//! - Thorough test suite
//! - Comprehensive documentation

// TODO #![deny(missing_docs)]

// Re-exports

pub use algonaut_core as core;
pub use algonaut_crypto as crypto;
pub use algonaut_model as model;
pub use algonaut_transaction as transaction;

pub mod algod;
pub mod error;
pub mod indexer;
pub mod kmd;
