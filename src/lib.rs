//! # algonaut
//!
//! Rust **algonaut** is a rusty SDK for [Algorand](https://www.algorand.com/). Please, be aware that this crate is a work in progress.

// TODO #![deny(missing_docs)]

// Re-exports

pub use algonaut_abi as abi;
pub use algonaut_core as core;
pub use algonaut_crypto as crypto;
pub use algonaut_model as model;
pub use algonaut_transaction as transaction;

#[cfg(feature = "algod")]
pub mod algod;
#[cfg(feature = "indexer")]
pub mod indexer;
#[cfg(feature = "kmd")]
pub mod kmd;

// atomic / dryrun / simulate build on the algod models (now in
// `algonaut_model::algod`) and the algod transport, so they ride the `algod`
// gate.
#[cfg(feature = "algod")]
pub mod atomic;
#[cfg(feature = "algod")]
pub mod dryrun;
#[cfg(feature = "algod")]
pub mod simulate;

pub mod error;
pub use error::Error;

pub mod util;
