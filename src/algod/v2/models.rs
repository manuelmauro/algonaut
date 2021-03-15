use crate::crypto::HashDigest;
use crate::serialization::deserialize_hash;
use serde::{Deserialize, Serialize};

/// Version contains the current algod version.
#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub versions: Vec<String>,
    pub genesis_id: String,
    #[serde(rename = "genesis_hash_b64", deserialize_with = "deserialize_hash")]
    pub genesis_hash: HashDigest,
}
