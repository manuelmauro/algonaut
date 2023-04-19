# TransactionParams200Response

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**consensus_version** | **String** | ConsensusVersion indicates the consensus protocol version as of LastRound. | 
**fee** | **i32** | Fee is the suggested transaction fee Fee is in units of micro-Algos per byte. Fee may fall to zero but transactions must still have a fee of at least MinTxnFee for the current network protocol. | 
**genesis_hash** | **String** | GenesisHash is the hash of the genesis block. | 
**genesis_id** | **String** | GenesisID is an ID listed in the genesis block. | 
**last_round** | **i32** | LastRound indicates the last round seen | 
**min_fee** | **i32** | The minimum transaction fee (not per byte) required for the txn to validate for the current network protocol. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


