# SimulateTransactionResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**app_budget_consumed** | Option<**u64**> | Budget used during execution of an app call transaction. This value includes budged used by inner app calls spawned by this transaction. | [optional]
**logic_sig_budget_consumed** | Option<**u64**> | Budget used during execution of a logic sig transaction. | [optional]
**missing_signature** | Option<**bool**> | A boolean indicating whether this transaction is missing signatures | [optional]
**txn_result** | [**crate::models::PendingTransactionResponse**](PendingTransactionResponse.md) |  |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
