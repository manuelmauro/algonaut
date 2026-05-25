//! # algonaut
//!
//! Rust **algonaut** is a rusty SDK for [Algorand](https://www.algorand.com/). Please, be aware that this crate is a work in progress.

// TODO #![deny(missing_docs)]

// Re-exports

pub use algonaut_abi as abi;
/// Generate a typed contract client from an ARC-4 ABI JSON file.
///
/// See [`algonaut_abi::contract`] for full documentation.
pub use algonaut_abi::contract;
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

// Convenience re-exports for ergonomic top-level imports.
// The versioned paths (e.g., `algod::v2::Algod`) remain available for
// backwards compatibility and explicit version selection.
#[cfg(feature = "algod")]
pub use algod::v2::{Algod, PendingSubmission, SourceMap};
#[cfg(feature = "indexer")]
pub use indexer::v2::Indexer;
#[cfg(feature = "kmd")]
pub use kmd::v1::Kmd;

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
