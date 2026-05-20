use super::transaction::TransactionHeader;
use algonaut_crypto::HashDigest;
use algonaut_encoding::{Bytes, Text, deserialize_opt_lenient_str};
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
/// Field types follow algod's wire-format conventions:
///
/// - Clean byte fields (`gh`, `seed`, `txn256`) are [`Bytes`] — base64 in
///   JSON, raw `bin` in msgpack.
/// - Textual fields (`gen`, `proto`) are [`Text`] — string in JSON, `bin`
///   (lossy UTF-8) in msgpack.
/// - The mixed-format address-ish fields (`fees`, `prev`, `rwd`) stay
///   `Option<String>` via [`deserialize_opt_lenient_str`]: algod renders
///   them as base32-checksum addresses or `blk-…` strings in JSON and raw
///   `bin` in msgpack, so they can't be type-narrowed without losing
///   information. The lenient deserializer keeps strings verbatim and
///   base64-encodes raw bytes.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    #[serde(rename = "earn")]
    pub rewards_level: Option<u64>,
    #[serde(
        rename = "fees",
        default,
        deserialize_with = "deserialize_opt_lenient_str"
    )]
    pub fee_sink: Option<String>,
    #[serde(rename = "frac")]
    pub rewards_residue: Option<u64>,
    #[serde(rename = "gen", default)]
    pub genesis_id: Option<Text>,
    #[serde(rename = "gh", default)]
    pub genesis_hash: Option<Bytes>,
    #[serde(
        rename = "prev",
        default,
        deserialize_with = "deserialize_opt_lenient_str"
    )]
    pub branch: Option<String>,
    #[serde(rename = "proto", default)]
    pub current_protocol: Option<Text>,
    #[serde(rename = "rate")]
    pub rewards_rate: Option<u64>,
    #[serde(rename = "rnd")]
    pub round: Option<u64>,
    #[serde(rename = "rwcalr")]
    pub rewards_recalculation_round: Option<u64>,
    #[serde(
        rename = "rwd",
        default,
        deserialize_with = "deserialize_opt_lenient_str"
    )]
    pub rewards_pool: Option<String>,
    #[serde(rename = "seed", default)]
    pub seed: Option<Bytes>,
    #[serde(rename = "ts")]
    pub timestamp: Option<u64>,
    #[serde(rename = "txn256", default)]
    pub txn_commitment: Option<Bytes>,
    #[serde(rename = "txns")]
    pub txns: Option<Vec<TransactionHeader>>,
}

impl Block {
    /// The rewards pool as algod's JSON-canonical base64 string.
    ///
    /// When the block arrived as msgpack the 32-byte `rwd` value was
    /// base64-encoded on decode; when it arrived as JSON the string is
    /// returned as algod sent it (a base32-checksum address).
    pub fn rewards_pool_base64(&self) -> Option<&str> {
        self.rewards_pool.as_deref()
    }
}
