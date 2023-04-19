# DryrunRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**accounts** | [**Vec<crate::models::Account>**](Account.md) |  | 
**apps** | [**Vec<crate::models::Application>**](Application.md) |  | 
**latest_timestamp** | **i64** | LatestTimestamp is available to some TEAL scripts. Defaults to the latest confirmed timestamp this algod is attached to. | 
**protocol_version** | **String** | ProtocolVersion specifies a specific version string to operate under, otherwise whatever the current protocol of the network this algod is running in. | 
**round** | **i32** | Round is available to some TEAL scripts. Defaults to the current round on the network this algod is attached to. | 
**sources** | [**Vec<crate::models::DryrunSource>**](DryrunSource.md) |  | 
**txns** | **Vec<String>** |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


