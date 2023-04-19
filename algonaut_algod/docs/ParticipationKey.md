# ParticipationKey

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**address** | **String** | Address the key was generated for. | 
**effective_first_valid** | Option<**i32**> | When registered, this is the first round it may be used. | [optional]
**effective_last_valid** | Option<**i32**> | When registered, this is the last round it may be used. | [optional]
**id** | **String** | The key's ParticipationID. | 
**key** | [**crate::models::AccountParticipation**](AccountParticipation.md) |  | 
**last_block_proposal** | Option<**i32**> | Round when this key was last used to propose a block. | [optional]
**last_state_proof** | Option<**i32**> | Round when this key was last used to generate a state proof. | [optional]
**last_vote** | Option<**i32**> | Round when this key was last used to vote. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


