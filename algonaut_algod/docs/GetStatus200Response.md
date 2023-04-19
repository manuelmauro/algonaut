# GetStatus200Response

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**catchpoint** | Option<**String**> | The current catchpoint that is being caught up to | [optional]
**catchpoint_acquired_blocks** | Option<**i32**> | The number of blocks that have already been obtained by the node as part of the catchup | [optional]
**catchpoint_processed_accounts** | Option<**i32**> | The number of accounts from the current catchpoint that have been processed so far as part of the catchup | [optional]
**catchpoint_processed_kvs** | Option<**i32**> | The number of key-values (KVs) from the current catchpoint that have been processed so far as part of the catchup | [optional]
**catchpoint_total_accounts** | Option<**i32**> | The total number of accounts included in the current catchpoint | [optional]
**catchpoint_total_blocks** | Option<**i32**> | The total number of blocks that are required to complete the current catchpoint catchup | [optional]
**catchpoint_total_kvs** | Option<**i32**> | The total number of key-values (KVs) included in the current catchpoint | [optional]
**catchpoint_verified_accounts** | Option<**i32**> | The number of accounts from the current catchpoint that have been verified so far as part of the catchup | [optional]
**catchpoint_verified_kvs** | Option<**i32**> | The number of key-values (KVs) from the current catchpoint that have been verified so far as part of the catchup | [optional]
**catchup_time** | **i32** | CatchupTime in nanoseconds | 
**last_catchpoint** | Option<**String**> | The last catchpoint seen by the node | [optional]
**last_round** | **i32** | LastRound indicates the last round seen | 
**last_version** | **String** | LastVersion indicates the last consensus version supported | 
**next_version** | **String** | NextVersion of consensus protocol to use | 
**next_version_round** | **i32** | NextVersionRound is the round at which the next consensus version will apply | 
**next_version_supported** | **bool** | NextVersionSupported indicates whether the next consensus version is supported by this node | 
**stopped_at_unsupported_round** | **bool** | StoppedAtUnsupportedRound indicates that the node does not support the new rounds and has stopped making progress | 
**time_since_last_round** | **i32** | TimeSinceLastRound in nanoseconds | 
**upgrade_delay** | Option<**i32**> | Upgrade delay | [optional]
**upgrade_next_protocol_vote_before** | Option<**i32**> | Next protocol round | [optional]
**upgrade_no_votes** | Option<**i32**> | No votes cast for consensus upgrade | [optional]
**upgrade_node_vote** | Option<**bool**> | This node's upgrade vote | [optional]
**upgrade_vote_rounds** | Option<**i32**> | Total voting rounds for current upgrade | [optional]
**upgrade_votes** | Option<**i32**> | Total votes cast for consensus upgrade | [optional]
**upgrade_votes_required** | Option<**i32**> | Yes votes required for consensus upgrade | [optional]
**upgrade_yes_votes** | Option<**i32**> | Yes votes cast for consensus upgrade | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


