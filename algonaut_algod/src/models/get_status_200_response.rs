/*
 * Algod REST API.
 *
 * API endpoint for algod operations.
 *
 * The version of the OpenAPI document: 0.0.1
 * Contact: contact@algorand.com
 * Generated by: https://openapi-generator.tech
 */

/// GetStatus200Response : NodeStatus contains the information about a node status

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct GetStatus200Response {
    /// The current catchpoint that is being caught up to
    #[serde(rename = "catchpoint", skip_serializing_if = "Option::is_none")]
    pub catchpoint: Option<String>,
    /// The number of blocks that have already been obtained by the node as part of the catchup
    #[serde(
        rename = "catchpoint-acquired-blocks",
        skip_serializing_if = "Option::is_none"
    )]
    pub catchpoint_acquired_blocks: Option<u64>,
    /// The number of accounts from the current catchpoint that have been processed so far as part of the catchup
    #[serde(
        rename = "catchpoint-processed-accounts",
        skip_serializing_if = "Option::is_none"
    )]
    pub catchpoint_processed_accounts: Option<u64>,
    /// The number of key-values (KVs) from the current catchpoint that have been processed so far as part of the catchup
    #[serde(
        rename = "catchpoint-processed-kvs",
        skip_serializing_if = "Option::is_none"
    )]
    pub catchpoint_processed_kvs: Option<u64>,
    /// The total number of accounts included in the current catchpoint
    #[serde(
        rename = "catchpoint-total-accounts",
        skip_serializing_if = "Option::is_none"
    )]
    pub catchpoint_total_accounts: Option<u64>,
    /// The total number of blocks that are required to complete the current catchpoint catchup
    #[serde(
        rename = "catchpoint-total-blocks",
        skip_serializing_if = "Option::is_none"
    )]
    pub catchpoint_total_blocks: Option<u64>,
    /// The total number of key-values (KVs) included in the current catchpoint
    #[serde(
        rename = "catchpoint-total-kvs",
        skip_serializing_if = "Option::is_none"
    )]
    pub catchpoint_total_kvs: Option<u64>,
    /// The number of accounts from the current catchpoint that have been verified so far as part of the catchup
    #[serde(
        rename = "catchpoint-verified-accounts",
        skip_serializing_if = "Option::is_none"
    )]
    pub catchpoint_verified_accounts: Option<u64>,
    /// The number of key-values (KVs) from the current catchpoint that have been verified so far as part of the catchup
    #[serde(
        rename = "catchpoint-verified-kvs",
        skip_serializing_if = "Option::is_none"
    )]
    pub catchpoint_verified_kvs: Option<u64>,
    /// CatchupTime in nanoseconds
    #[serde(rename = "catchup-time")]
    pub catchup_time: u64,
    /// The last catchpoint seen by the node
    #[serde(rename = "last-catchpoint", skip_serializing_if = "Option::is_none")]
    pub last_catchpoint: Option<String>,
    /// LastRound indicates the last round seen
    #[serde(rename = "last-round")]
    pub last_round: u64,
    /// LastVersion indicates the last consensus version supported
    #[serde(rename = "last-version")]
    pub last_version: String,
    /// NextVersion of consensus protocol to use
    #[serde(rename = "next-version")]
    pub next_version: String,
    /// NextVersionRound is the round at which the next consensus version will apply
    #[serde(rename = "next-version-round")]
    pub next_version_round: u64,
    /// NextVersionSupported indicates whether the next consensus version is supported by this node
    #[serde(rename = "next-version-supported")]
    pub next_version_supported: bool,
    /// StoppedAtUnsupportedRound indicates that the node does not support the new rounds and has stopped making progress
    #[serde(rename = "stopped-at-unsupported-round")]
    pub stopped_at_unsupported_round: bool,
    /// TimeSinceLastRound in nanoseconds
    #[serde(rename = "time-since-last-round")]
    pub time_since_last_round: u64,
    /// Upgrade delay
    #[serde(rename = "upgrade-delay", skip_serializing_if = "Option::is_none")]
    pub upgrade_delay: Option<u64>,
    /// Next protocol round
    #[serde(
        rename = "upgrade-next-protocol-vote-before",
        skip_serializing_if = "Option::is_none"
    )]
    pub upgrade_next_protocol_vote_before: Option<u64>,
    /// No votes cast for consensus upgrade
    #[serde(rename = "upgrade-no-votes", skip_serializing_if = "Option::is_none")]
    pub upgrade_no_votes: Option<u64>,
    /// This node's upgrade vote
    #[serde(rename = "upgrade-node-vote", skip_serializing_if = "Option::is_none")]
    pub upgrade_node_vote: Option<bool>,
    /// Total voting rounds for current upgrade
    #[serde(
        rename = "upgrade-vote-rounds",
        skip_serializing_if = "Option::is_none"
    )]
    pub upgrade_vote_rounds: Option<u64>,
    /// Total votes cast for consensus upgrade
    #[serde(rename = "upgrade-votes", skip_serializing_if = "Option::is_none")]
    pub upgrade_votes: Option<u64>,
    /// Yes votes required for consensus upgrade
    #[serde(
        rename = "upgrade-votes-required",
        skip_serializing_if = "Option::is_none"
    )]
    pub upgrade_votes_required: Option<u64>,
    /// Yes votes cast for consensus upgrade
    #[serde(rename = "upgrade-yes-votes", skip_serializing_if = "Option::is_none")]
    pub upgrade_yes_votes: Option<u64>,
}

impl GetStatus200Response {
    /// NodeStatus contains the information about a node status
    pub fn new(
        catchup_time: u64,
        last_round: u64,
        last_version: String,
        next_version: String,
        next_version_round: u64,
        next_version_supported: bool,
        stopped_at_unsupported_round: bool,
        time_since_last_round: u64,
    ) -> GetStatus200Response {
        GetStatus200Response {
            catchpoint: None,
            catchpoint_acquired_blocks: None,
            catchpoint_processed_accounts: None,
            catchpoint_processed_kvs: None,
            catchpoint_total_accounts: None,
            catchpoint_total_blocks: None,
            catchpoint_total_kvs: None,
            catchpoint_verified_accounts: None,
            catchpoint_verified_kvs: None,
            catchup_time,
            last_catchpoint: None,
            last_round,
            last_version,
            next_version,
            next_version_round,
            next_version_supported,
            stopped_at_unsupported_round,
            time_since_last_round,
            upgrade_delay: None,
            upgrade_next_protocol_vote_before: None,
            upgrade_no_votes: None,
            upgrade_node_vote: None,
            upgrade_vote_rounds: None,
            upgrade_votes: None,
            upgrade_votes_required: None,
            upgrade_yes_votes: None,
        }
    }
}
