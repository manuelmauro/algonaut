# SimulateTransaction200Response

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**last_round** | **i32** | The round immediately preceding this simulation. State changes through this round were used to run this simulation. | 
**txn_groups** | [**Vec<crate::models::SimulateTransactionGroupResult>**](SimulateTransactionGroupResult.md) | A result object for each transaction group that was simulated. | 
**version** | **i32** | The version of this response object. | 
**would_succeed** | **bool** | Indicates whether the simulated transactions would have succeeded during an actual submission. If any transaction fails or is missing a signature, this will be false. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


