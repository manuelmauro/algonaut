# Account

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**address** | **String** | the account public key | 
**amount** | **i32** | \\[algo\\] total number of MicroAlgos in the account | 
**amount_without_pending_rewards** | **i32** | specifies the amount of MicroAlgos in the account, without the pending rewards. | 
**apps_local_state** | Option<[**Vec<crate::models::ApplicationLocalState>**](ApplicationLocalState.md)> | \\[appl\\] applications local data stored in this account.  Note the raw object uses `map[int] -> AppLocalState` for this type. | [optional]
**apps_total_extra_pages** | Option<**i32**> | \\[teap\\] the sum of all extra application program pages for this account. | [optional]
**apps_total_schema** | Option<[**crate::models::ApplicationStateSchema**](ApplicationStateSchema.md)> |  | [optional]
**assets** | Option<[**Vec<crate::models::AssetHolding>**](AssetHolding.md)> | \\[asset\\] assets held by this account.  Note the raw object uses `map[int] -> AssetHolding` for this type. | [optional]
**auth_addr** | Option<**String**> | \\[spend\\] the address against which signing should be checked. If empty, the address of the current account is used. This field can be updated in any transaction by setting the RekeyTo field. | [optional]
**created_apps** | Option<[**Vec<crate::models::Application>**](Application.md)> | \\[appp\\] parameters of applications created by this account including app global data.  Note: the raw account uses `map[int] -> AppParams` for this type. | [optional]
**created_assets** | Option<[**Vec<crate::models::Asset>**](Asset.md)> | \\[apar\\] parameters of assets created by this account.  Note: the raw account uses `map[int] -> Asset` for this type. | [optional]
**min_balance** | **i32** | MicroAlgo balance required by the account.  The requirement grows based on asset and application usage. | 
**participation** | Option<[**crate::models::AccountParticipation**](AccountParticipation.md)> |  | [optional]
**pending_rewards** | **i32** | amount of MicroAlgos of pending rewards in this account. | 
**reward_base** | Option<**i32**> | \\[ebase\\] used as part of the rewards computation. Only applicable to accounts which are participating. | [optional]
**rewards** | **i32** | \\[ern\\] total rewards of MicroAlgos the account has received, including pending rewards. | 
**round** | **i32** | The round for which this information is relevant. | 
**sig_type** | Option<**String**> | Indicates what type of signature is used by this account, must be one of: * sig * msig * lsig | [optional]
**status** | **String** | \\[onl\\] delegation status of the account's MicroAlgos * Offline - indicates that the associated account is delegated. *  Online  - indicates that the associated account used as part of the delegation pool. *   NotParticipating - indicates that the associated account is neither a delegator nor a delegate. | 
**total_apps_opted_in** | **i32** | The count of all applications that have been opted in, equivalent to the count of application local data (AppLocalState objects) stored in this account. | 
**total_assets_opted_in** | **i32** | The count of all assets that have been opted in, equivalent to the count of AssetHolding objects held by this account. | 
**total_box_bytes** | Option<**i32**> | \\[tbxb\\] The total number of bytes used by this account's app's box keys and values. | [optional]
**total_boxes** | Option<**i32**> | \\[tbx\\] The number of existing boxes created by this account's app. | [optional]
**total_created_apps** | **i32** | The count of all apps (AppParams objects) created by this account. | 
**total_created_assets** | **i32** | The count of all assets (AssetParams objects) created by this account. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


