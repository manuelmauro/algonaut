//! Hand-named response types for the high-level `Algod` / `Indexer` /
//! `Kmd` clients. Each type wraps a generated `*200Response` from the
//! `algonaut_algod` / `algonaut_indexer` / `algonaut_kmd` crates, with a
//! `From` impl at the boundary, so the public client surface stops
//! exposing OpenAPI-generator names.
//!
//! See ADR `hide-generated-types` for the staged rollout. The current
//! cut covers the most-touched responses (`SuggestedParams`,
//! `NodeStatus`, `Supply`); the rest land as follow-ups, one method at
//! a time.

use algonaut_core::{MicroAlgos, Round};
use algonaut_crypto::HashDigest;

/// The trait every transaction-builder's `build(&params)` consumes.
///
/// Lived in `algonaut_transaction::builder` until D3 moved it here so
/// `algonaut_model` types ([`SuggestedParams`] in particular) can
/// implement it without `algonaut_model` depending on
/// `algonaut_transaction` (the workspace already has
/// `algonaut_transaction → algonaut_model`, so `algonaut_model →
/// algonaut_transaction` would cycle). Re-exported from
/// `algonaut_transaction::builder` for backward compatibility.
pub trait TransactionParams {
    fn last_round(&self) -> u64;
    fn min_fee(&self) -> u64;
    fn genesis_hash(&self) -> HashDigest;
    fn genesis_id(&self) -> &String;
}

/// Parameters the client needs to construct a new transaction —
/// renamed from the generated `TransactionParams200Response`.
#[derive(Debug, Clone, PartialEq)]
pub struct SuggestedParams {
    /// Consensus protocol version as of `last_round`.
    pub consensus_version: String,
    /// Suggested transaction fee, in micro-Algos per byte.
    pub fee_per_byte: MicroAlgos,
    /// Hash of the genesis block.
    pub genesis_hash: HashDigest,
    /// Genesis ID listed in the genesis block.
    pub genesis_id: String,
    /// Last round seen by the node.
    pub last_round: Round,
    /// Minimum transaction fee (not per byte) for the current protocol.
    pub min_fee: MicroAlgos,
}

impl TransactionParams for SuggestedParams {
    fn last_round(&self) -> u64 {
        self.last_round.0
    }

    fn min_fee(&self) -> u64 {
        self.min_fee.0
    }

    fn genesis_hash(&self) -> HashDigest {
        self.genesis_hash
    }

    fn genesis_id(&self) -> &String {
        &self.genesis_id
    }
}

/// Current node status — renamed from the generated `GetStatus200Response`,
/// with the most useful fields surfaced. Fields the SDK doesn't surface
/// are dropped from the hand-named type for now; if a caller needs them
/// they're a `From` impl extension away.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeStatus {
    pub catchup_time: u64,
    pub last_round: Round,
    pub last_version: String,
    pub next_version: String,
    pub next_version_round: Round,
    pub next_version_supported: bool,
    pub stopped_at_unsupported_round: bool,
    pub time_since_last_round: u64,
}

/// Network supply totals — renamed from the generated `GetSupply200Response`.
#[derive(Debug, Clone, PartialEq)]
pub struct Supply {
    pub current_round: Round,
    pub total_money: MicroAlgos,
    pub online_money: MicroAlgos,
}
