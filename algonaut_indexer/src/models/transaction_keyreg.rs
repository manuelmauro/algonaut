/*
 * Indexer
 *
 * Algorand ledger analytics API.
 *
 * The version of the OpenAPI document: 2.0
 *
 * Generated by: https://openapi-generator.tech
 */

use algonaut_encoding::Bytes;

/// TransactionKeyreg : Fields for a keyreg transaction.  Definition: data/transactions/keyreg.go : KeyregTxnFields

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct TransactionKeyreg {
    /// \\[nonpart\\] Mark the account as participating or non-participating.
    #[serde(rename = "non-participation", skip_serializing_if = "Option::is_none")]
    pub non_participation: Option<bool>,
    /// \\[selkey\\] Public key used with the Verified Random Function (VRF) result during committee selection.
    #[serde(
        rename = "selection-participation-key",
        skip_serializing_if = "Option::is_none"
    )]
    pub selection_participation_key: Option<Bytes>,
    /// \\[sprfkey\\] State proof key used in key registration transactions.
    #[serde(rename = "state-proof-key", skip_serializing_if = "Option::is_none")]
    pub state_proof_key: Option<Bytes>,
    /// \\[votefst\\] First round this participation key is valid.
    #[serde(rename = "vote-first-valid", skip_serializing_if = "Option::is_none")]
    pub vote_first_valid: Option<u64>,
    /// \\[votekd\\] Number of subkeys in each batch of participation keys.
    #[serde(rename = "vote-key-dilution", skip_serializing_if = "Option::is_none")]
    pub vote_key_dilution: Option<u64>,
    /// \\[votelst\\] Last round this participation key is valid.
    #[serde(rename = "vote-last-valid", skip_serializing_if = "Option::is_none")]
    pub vote_last_valid: Option<u64>,
    /// \\[votekey\\] Participation public key used in key registration transactions.
    #[serde(
        rename = "vote-participation-key",
        skip_serializing_if = "Option::is_none"
    )]
    pub vote_participation_key: Option<Bytes>,
}

impl TransactionKeyreg {
    /// Fields for a keyreg transaction.  Definition: data/transactions/keyreg.go : KeyregTxnFields
    pub fn new() -> TransactionKeyreg {
        TransactionKeyreg {
            non_participation: None,
            selection_participation_key: None,
            state_proof_key: None,
            vote_first_valid: None,
            vote_key_dilution: None,
            vote_last_valid: None,
            vote_participation_key: None,
        }
    }
}
