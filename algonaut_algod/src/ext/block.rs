use super::transaction::TransactionHeader;
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    #[serde(rename = "earn")]
    pub rewards_level: Option<u64>,
    #[serde(rename = "fees")]
    pub fee_sink: Option<String>,
    #[serde(rename = "frac")]
    pub rewards_residue: Option<u64>,
    #[serde(rename = "gen")]
    pub genesis_id: Option<String>,
    #[serde(rename = "gh")]
    pub genesis_hash: Option<String>,
    #[serde(rename = "prev")]
    pub branch: Option<String>,
    #[serde(rename = "proto")]
    pub current_protocol: Option<String>,
    #[serde(rename = "rate")]
    pub rewards_rate: Option<u64>,
    #[serde(rename = "rnd")]
    pub round: Option<u64>,
    #[serde(rename = "rwcalr")]
    pub rewards_recalculation_round: Option<u64>,
    #[serde(rename = "rwd")]
    pub rewards_pool: Option<String>,
    #[serde(rename = "seed")]
    pub seed: Option<String>,
    #[serde(rename = "ts")]
    pub timestamp: Option<u64>,
    #[serde(rename = "txn256")]
    pub txn_commitment: Option<String>,
    #[serde(rename = "txns")]
    pub txns: Option<Vec<TransactionHeader>>,
}
