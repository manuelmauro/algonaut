//! # algonaut
//!
//! Rust **algonaut** is a rusty SDK for [Algorand](https://www.algorand.com/). Please, be aware that this crate is a work in progress.

// TODO #![deny(missing_docs)]

// Re-exports

pub use algonaut_abi as abi;
pub use algonaut_algod as algod_exp;
pub use algonaut_core as core;
pub use algonaut_crypto as crypto;
pub use algonaut_model as model;
pub use algonaut_transaction as transaction;

pub mod algod;
pub mod atomic_transaction_composer;
pub mod error;
pub use error::Error;

pub mod indexer;
pub mod kmd;
pub mod util;
