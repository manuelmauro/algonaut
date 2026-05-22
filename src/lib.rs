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

// Raw generated clients. These re-exports are a transitional escape hatch:
// they ride their client's feature gate today and are removed once the
// hand-named-types migration completes (see ADR `hide-generated-types`).
#[cfg(feature = "algod")]
pub use algonaut_algod as openapi_algod;
#[cfg(feature = "indexer")]
pub use algonaut_indexer as openapi_indexer;
#[cfg(feature = "kmd")]
pub use algonaut_kmd as openapi_kmd;

#[cfg(feature = "algod")]
pub mod algod;
#[cfg(feature = "indexer")]
pub mod indexer;
#[cfg(feature = "kmd")]
pub mod kmd;

// atomic / dryrun / simulate consume `algonaut_algod::models` directly, so
// they ride the `algod` gate until that coupling is removed (ADR D3).
#[cfg(feature = "algod")]
pub mod atomic;
#[cfg(feature = "algod")]
pub mod dryrun;
#[cfg(feature = "algod")]
pub mod simulate;

pub mod error;
pub use error::Error;

pub mod util;
