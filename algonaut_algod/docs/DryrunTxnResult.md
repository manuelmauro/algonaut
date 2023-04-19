# DryrunTxnResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**app_call_messages** | Option<**Vec<String>**> |  | [optional]
**app_call_trace** | Option<[**Vec<crate::models::DryrunState>**](DryrunState.md)> |  | [optional]
**budget_added** | Option<**i32**> | Budget added during execution of app call transaction. | [optional]
**budget_consumed** | Option<**i32**> | Budget consumed during execution of app call transaction. | [optional]
**disassembly** | **Vec<String>** | Disassembled program line by line. | 
**global_delta** | Option<[**Vec<crate::models::EvalDeltaKeyValue>**](EvalDeltaKeyValue.md)> | Application state delta. | [optional]
**local_deltas** | Option<[**Vec<crate::models::AccountStateDelta>**](AccountStateDelta.md)> |  | [optional]
**logic_sig_disassembly** | Option<**Vec<String>**> | Disassembled lsig program line by line. | [optional]
**logic_sig_messages** | Option<**Vec<String>**> |  | [optional]
**logic_sig_trace** | Option<[**Vec<crate::models::DryrunState>**](DryrunState.md)> |  | [optional]
**logs** | Option<**Vec<String>**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


