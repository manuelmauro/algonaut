# SimulateTransactionGroupResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**app_budget_added** | Option<**i32**> | Total budget added during execution of app calls in the transaction group. | [optional]
**app_budget_consumed** | Option<**i32**> | Total budget consumed during execution of app calls in the transaction group. | [optional]
**failed_at** | Option<**Vec<i32>**> | If present, indicates which transaction in this group caused the failure. This array represents the path to the failing transaction. Indexes are zero based, the first element indicates the top-level transaction, and successive elements indicate deeper inner transactions. | [optional]
**failure_message** | Option<**String**> | If present, indicates that the transaction group failed and specifies why that happened | [optional]
**txn_results** | [**Vec<crate::models::SimulateTransactionResult>**](SimulateTransactionResult.md) | Simulation result for individual transactions | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


