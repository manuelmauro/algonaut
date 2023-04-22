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

/// AccountParticipation : AccountParticipation describes the parameters used by this account in consensus protocol.

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct AccountParticipation {
    /// \\[sel\\] Selection public key (if any) currently registered for this round.
    #[serde(rename = "selection-participation-key")]
    pub selection_participation_key: Bytes,
    /// \\[stprf\\] Root of the state proof key (if any)
    #[serde(rename = "state-proof-key", skip_serializing_if = "Option::is_none")]
    pub state_proof_key: Option<Bytes>,
    /// \\[voteFst\\] First round for which this participation is valid.
    #[serde(rename = "vote-first-valid")]
    pub vote_first_valid: u64,
    /// \\[voteKD\\] Number of subkeys in each batch of participation keys.
    #[serde(rename = "vote-key-dilution")]
    pub vote_key_dilution: u64,
    /// \\[voteLst\\] Last round for which this participation is valid.
    #[serde(rename = "vote-last-valid")]
    pub vote_last_valid: u64,
    /// \\[vote\\] root participation public key (if any) currently registered for this round.
    #[serde(rename = "vote-participation-key")]
    pub vote_participation_key: Bytes,
}

impl AccountParticipation {
    /// AccountParticipation describes the parameters used by this account in consensus protocol.
    pub fn new(
        selection_participation_key: Bytes,
        vote_first_valid: u64,
        vote_key_dilution: u64,
        vote_last_valid: u64,
        vote_participation_key: Bytes,
    ) -> AccountParticipation {
        AccountParticipation {
            selection_participation_key,
            state_proof_key: None,
            vote_first_valid,
            vote_key_dilution,
            vote_last_valid,
            vote_participation_key,
        }
    }
}
