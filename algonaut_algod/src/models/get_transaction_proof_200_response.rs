/*
 * Algod REST API.
 *
 * API endpoint for algod operations.
 *
 * The version of the OpenAPI document: 0.0.1
 * Contact: contact@algorand.com
 * Generated by: https://openapi-generator.tech
 */

use algonaut_encoding::Bytes;

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct GetTransactionProof200Response {
    /// The type of hash function used to create the proof, must be one of:  * sha512_256  * sha256
    #[serde(rename = "hashtype")]
    pub hashtype: Hashtype,
    /// Index of the transaction in the block's payset.
    #[serde(rename = "idx")]
    pub idx: u64,
    /// Proof of transaction membership.
    #[serde(rename = "proof")]
    pub proof: Bytes,
    /// Hash of SignedTxnInBlock for verifying proof.
    #[serde(rename = "stibhash")]
    pub stibhash: Bytes,
    /// Represents the depth of the tree that is being proven, i.e. the number of edges from a leaf to the root.
    #[serde(rename = "treedepth")]
    pub treedepth: u64,
}

impl GetTransactionProof200Response {
    pub fn new(
        hashtype: Hashtype,
        idx: u64,
        proof: Bytes,
        stibhash: Bytes,
        treedepth: u64,
    ) -> GetTransactionProof200Response {
        GetTransactionProof200Response {
            hashtype,
            idx,
            proof,
            stibhash,
            treedepth,
        }
    }
}

/// The type of hash function used to create the proof, must be one of:  * sha512_256  * sha256
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Hashtype {
    #[serde(rename = "sha512_256")]
    Sha512256,
    #[serde(rename = "sha256")]
    Sha256,
}

impl Default for Hashtype {
    fn default() -> Hashtype {
        Self::Sha512256
    }
}
