//! The [`TransactionParams`] trait consumed by the transaction builders.
//!
//! The hand-written response wrappers that used to live here
//! (`SuggestedParams`, `NodeStatus`, `Supply`) were retired by ADR
//! `relocate-generated-models`: the relocated, renamed and domain-typed
//! generated models in [`crate::algod`] reproduce them, so client methods
//! now return those directly. Only the trait remains, because it is the
//! contract the builders depend on and several generated models implement it.

use algonaut_crypto::HashDigest;

/// The trait every transaction-builder's `build(&params)` consumes.
///
/// Lived in `algonaut_transaction::builder` until D3 of
/// `ideal-type-safe-ergonomic-api` moved it here so `algonaut_model` types
/// ([`crate::algod::SuggestedParams`] in particular) can implement it without
/// `algonaut_model` depending on `algonaut_transaction` (the workspace already
/// has `algonaut_transaction → algonaut_model`, so `algonaut_model →
/// algonaut_transaction` would cycle). Re-exported from
/// `algonaut_transaction::builder` for backward compatibility.
pub trait TransactionParams {
    fn last_round(&self) -> u64;
    fn min_fee(&self) -> u64;
    fn genesis_hash(&self) -> HashDigest;
    fn genesis_id(&self) -> &String;
}
