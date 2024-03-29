/*
 * Indexer
 *
 * Algorand ledger analytics API.
 *
 * The version of the OpenAPI document: 2.0
 *
 * Generated by: https://openapi-generator.tech
 */

use algonaut_crypto::{deserialize_hash, HashDigest};
use algonaut_encoding::Bytes;

/// Block : Block information.  Definition: data/bookkeeping/block.go : Block

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Block {
    /// \\[gh\\] hash to which this block belongs.
    #[serde(rename = "genesis-hash", deserialize_with = "deserialize_hash")]
    pub genesis_hash: HashDigest,
    /// \\[gen\\] ID to which this block belongs.
    #[serde(rename = "genesis-id")]
    pub genesis_id: String,
    #[serde(
        rename = "participation-updates",
        skip_serializing_if = "Option::is_none"
    )]
    pub participation_updates: Option<Box<crate::models::ParticipationUpdates>>,
    /// \\[prev\\] Previous block hash.
    #[serde(rename = "previous-block-hash", deserialize_with = "deserialize_hash")]
    pub previous_block_hash: HashDigest,
    #[serde(rename = "rewards", skip_serializing_if = "Option::is_none")]
    pub rewards: Option<Box<crate::models::BlockRewards>>,
    /// \\[rnd\\] Current round on which this block was appended to the chain.
    #[serde(rename = "round")]
    pub round: u64,
    /// \\[seed\\] Sortition seed.
    #[serde(rename = "seed")]
    pub seed: Bytes,
    /// Tracks the status of state proofs.
    #[serde(
        rename = "state-proof-tracking",
        skip_serializing_if = "Option::is_none"
    )]
    pub state_proof_tracking: Option<Vec<crate::models::StateProofTracking>>,
    /// \\[ts\\] Block creation timestamp in seconds since eposh
    #[serde(rename = "timestamp")]
    pub timestamp: u64,
    /// \\[txns\\] list of transactions corresponding to a given round.
    #[serde(rename = "transactions", skip_serializing_if = "Option::is_none")]
    pub transactions: Option<Vec<crate::models::Transaction>>,
    /// \\[txn\\] TransactionsRoot authenticates the set of transactions appearing in the block. More specifically, it's the root of a merkle tree whose leaves are the block's Txids, in lexicographic order. For the empty block, it's 0. Note that the TxnRoot does not authenticate the signatures on the transactions, only the transactions themselves. Two blocks with the same transactions but in a different order and with different signatures will have the same TxnRoot.
    #[serde(rename = "transactions-root")]
    pub transactions_root: Bytes,
    /// \\[txn256\\] TransactionsRootSHA256 is an auxiliary TransactionRoot, built using a vector commitment instead of a merkle tree, and SHA256 hash function instead of the default SHA512_256. This commitment can be used on environments where only the SHA256 function exists.
    #[serde(rename = "transactions-root-sha256")]
    pub transactions_root_sha256: Bytes,
    /// \\[tc\\] TxnCounter counts the number of transactions committed in the ledger, from the time at which support for this feature was introduced.  Specifically, TxnCounter is the number of the next transaction that will be committed after this block.  It is 0 when no transactions have ever been committed (since TxnCounter started being supported).
    #[serde(rename = "txn-counter", skip_serializing_if = "Option::is_none")]
    pub txn_counter: Option<u64>,
    #[serde(rename = "upgrade-state", skip_serializing_if = "Option::is_none")]
    pub upgrade_state: Option<Box<crate::models::BlockUpgradeState>>,
    #[serde(rename = "upgrade-vote", skip_serializing_if = "Option::is_none")]
    pub upgrade_vote: Option<Box<crate::models::BlockUpgradeVote>>,
}

impl Block {
    /// Block information.  Definition: data/bookkeeping/block.go : Block
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        genesis_hash: HashDigest,
        genesis_id: String,
        previous_block_hash: HashDigest,
        round: u64,
        seed: Bytes,
        timestamp: u64,
        transactions_root: Bytes,
        transactions_root_sha256: Bytes,
    ) -> Block {
        Block {
            genesis_hash,
            genesis_id,
            participation_updates: None,
            previous_block_hash,
            rewards: None,
            round,
            seed,
            state_proof_tracking: None,
            timestamp,
            transactions: None,
            transactions_root,
            transactions_root_sha256,
            txn_counter: None,
            upgrade_state: None,
            upgrade_vote: None,
        }
    }
}
