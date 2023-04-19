# ApplicationParams

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**approval_program** | **String** | \\[approv\\] approval program. | 
**clear_state_program** | **String** | \\[clearp\\] approval program. | 
**creator** | **String** | The address that created this application. This is the address where the parameters and global state for this application can be found. | 
**extra_program_pages** | Option<**i32**> | \\[epp\\] the amount of extra program pages available to this app. | [optional]
**global_state** | Option<[**Vec<crate::models::TealKeyValue>**](TealKeyValue.md)> | Represents a key-value store for use in an application. | [optional]
**global_state_schema** | Option<[**crate::models::ApplicationStateSchema**](ApplicationStateSchema.md)> |  | [optional]
**local_state_schema** | Option<[**crate::models::ApplicationStateSchema**](ApplicationStateSchema.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


