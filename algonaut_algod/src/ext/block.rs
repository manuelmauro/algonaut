use super::transaction::TransactionHeader;
use super::wire::{deserialize_opt_bytes_str, deserialize_opt_text};
use algonaut_crypto::HashDigest;
use serde::{Deserialize, Serialize};

/// Block
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockResponse {
    /// Block header data.
    pub block: Block,
}

/// Block with certificate
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockWithCertificateResponse {
    /// Block header data
    pub block: Block,
    /// Certificate
    pub cert: BlockCertificate,
}

impl BlockWithCertificateResponse {
    pub fn hash(&self) -> HashDigest {
        self.cert.prop.hash
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockCertificate {
    pub prop: BlockCertificateProp,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockCertificateProp {
    #[serde(rename = "dig")]
    pub hash: HashDigest,
}

/// An algod block header.
///
/// The byte-valued fields (`fees`, `gh`, `prev`, `rwd`, `seed`, `txn256`) are
/// decoded leniently into `String`s: across algod's JSON and msgpack wire
/// formats — and the cross-SDK test fixtures — they appear as base64 strings,
/// base32 addresses, `blk-…` block-hash strings, or raw `bin`. A `bin` value
/// is base64-encoded; any string is kept verbatim. Only the JSON-rendered
/// base64 form (`rewards_pool`) is asserted on by the response features.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    #[serde(rename = "earn")]
    pub rewards_level: Option<u64>,
    #[serde(
        rename = "fees",
        default,
        deserialize_with = "deserialize_opt_bytes_str"
    )]
    pub fee_sink: Option<String>,
    #[serde(rename = "frac")]
    pub rewards_residue: Option<u64>,
    #[serde(rename = "gen", default, deserialize_with = "deserialize_opt_text")]
    pub genesis_id: Option<String>,
    #[serde(rename = "gh", default, deserialize_with = "deserialize_opt_bytes_str")]
    pub genesis_hash: Option<String>,
    #[serde(
        rename = "prev",
        default,
        deserialize_with = "deserialize_opt_bytes_str"
    )]
    pub branch: Option<String>,
    #[serde(rename = "proto", default, deserialize_with = "deserialize_opt_text")]
    pub current_protocol: Option<String>,
    #[serde(rename = "rate")]
    pub rewards_rate: Option<u64>,
    #[serde(rename = "rnd")]
    pub round: Option<u64>,
    #[serde(rename = "rwcalr")]
    pub rewards_recalculation_round: Option<u64>,
    #[serde(
        rename = "rwd",
        default,
        deserialize_with = "deserialize_opt_bytes_str"
    )]
    pub rewards_pool: Option<String>,
    #[serde(
        rename = "seed",
        default,
        deserialize_with = "deserialize_opt_bytes_str"
    )]
    pub seed: Option<String>,
    #[serde(rename = "ts")]
    pub timestamp: Option<u64>,
    #[serde(
        rename = "txn256",
        default,
        deserialize_with = "deserialize_opt_bytes_str"
    )]
    pub txn_commitment: Option<String>,
    #[serde(rename = "txns")]
    pub txns: Option<Vec<TransactionHeader>>,
}

impl Block {
    /// The rewards pool as algod's JSON-canonical base64 string.
    ///
    /// When the block arrived as msgpack the 32-byte `rwd` value was
    /// base64-encoded on decode; when it arrived as JSON the string is
    /// returned as algod sent it.
    pub fn rewards_pool_base64(&self) -> Option<&str> {
        self.rewards_pool.as_deref()
    }
}
