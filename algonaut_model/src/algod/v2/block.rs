use algonaut_crypto::HashDigest;
use serde::{Deserialize, Serialize};

/// Block
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    /// Block header data.
    pub block: BlockHeader,
}

/// Block with certificate
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockWithCertificate {
    /// Block header data
    pub block: BlockHeader,
    /// Certificate
    pub cert: BlockCertificate,
}

impl BlockWithCertificate {
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

/// BlockHeader
///
/// Note: fields seem to be managed as untyped map and currently not documented ([docs](https://developer.algorand.org/docs/rest-apis/algod/v2/#getblock-response-200)),
/// so everything optional. Some may be outdated or missing.
///
/// For now, also, byte array representations as strings,
/// different encodings and prefixes are used, hindering a standarized deserialization.
///
/// It probably makes sense to deserialize this and [BlockHeaderMsgPack]
/// to the same struct, but above makes it currently not possible.
///
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockHeader {
    #[serde(default)]
    pub earn: Option<u64>,
    #[serde(default)]
    pub fees: Option<String>,
    #[serde(default)]
    pub frac: Option<u64>,
    #[serde(default)]
    pub gen: Option<String>,
    #[serde(default)]
    pub gh: Option<String>,
    #[serde(default)]
    pub prev: Option<String>,
    #[serde(default)]
    pub proto: Option<String>,
    #[serde(default)]
    pub rate: Option<u64>,
    #[serde(default)]
    pub rnd: Option<u64>,
    #[serde(default)]
    pub rwcalr: Option<u64>,
    #[serde(default)]
    pub rwd: Option<String>,
    #[serde(default)]
    pub seed: Option<String>,
    #[serde(default)]
    pub ts: Option<u64>,
    #[serde(default)]
    pub txn256: Option<String>,
    #[serde(default)]
    pub txns: Option<Vec<TransactionHeader>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionHeader {
    pub hgi: Option<bool>,
    pub sig: Option<String>,
    pub txn: Option<Transaction>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    // Header
    pub fee: Option<u64>,
    pub fv: Option<u64>,
    pub gh: Option<String>,
    pub lv: Option<u64>,
    pub snd: Option<String>,
    #[serde(rename = "type")]
    pub t: Option<TransactionType>,
    pub gen: Option<String>,
    pub grp: Option<String>,
    pub lx: Option<String>,
    pub note: Option<String>,
    pub rekey: Option<String>,

    // Payment
    pub rcv: Option<String>,
    pub amt: Option<u64>,
    pub close: Option<String>,

    // Key Registration
    pub votekey: Option<String>,
    pub selkey: Option<String>,
    pub sprfkey: Option<String>,
    pub votefst: Option<u64>,
    pub votelst: Option<u64>,
    pub votekd: Option<u64>,
    pub nonpart: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TransactionType {
    #[serde(rename = "pay")]
    Payment,
    #[serde(rename = "keyreg")]
    KeyRegistration,
    #[serde(rename = "acfg")]
    AssetConfig,
    #[serde(rename = "axfer")]
    AssetTransfer,
    #[serde(rename = "afrz")]
    AssetFreeze,
    #[serde(rename = "appl")]
    Application,
}
