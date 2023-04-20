# \PrivateApi

All URIs are relative to *<http://localhost>*

Method | HTTP request | Description
------------- | ------------- | -------------
[**abort_catchup**](PrivateApi.md#abort_catchup) | **DELETE** /v2/catchup/{catchpoint} | Aborts a catchpoint catchup.
[**add_participation_key**](PrivateApi.md#add_participation_key) | **POST** /v2/participation | Add a participation key to the node
[**append_keys**](PrivateApi.md#append_keys) | **POST** /v2/participation/{participation-id} | Append state proof keys to a participation key
[**delete_participation_key_by_id**](PrivateApi.md#delete_participation_key_by_id) | **DELETE** /v2/participation/{participation-id} | Delete a given participation key by ID
[**get_participation_key_by_id**](PrivateApi.md#get_participation_key_by_id) | **GET** /v2/participation/{participation-id} | Get participation key info given a participation ID
[**get_participation_keys**](PrivateApi.md#get_participation_keys) | **GET** /v2/participation | Return a list of participation keys
[**shutdown_node**](PrivateApi.md#shutdown_node) | **POST** /v2/shutdown |
[**start_catchup**](PrivateApi.md#start_catchup) | **POST** /v2/catchup/{catchpoint} | Starts a catchpoint catchup.

## abort_catchup

> crate::models::AbortCatchup200Response abort_catchup(catchpoint)
Aborts a catchpoint catchup.

Given a catchpoint, it aborts catching up to this catchpoint

### Parameters

Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**catchpoint** | **String** | A catch point | [required] |

### Return type

[**crate::models::AbortCatchup200Response**](AbortCatchup_200_response.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

## add_participation_key

> crate::models::AddParticipationKey200Response add_participation_key(participationkey)
Add a participation key to the node

### Parameters

Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**participationkey** | **std::path::PathBuf** | The participation key to add to the node | [required] |

### Return type

[**crate::models::AddParticipationKey200Response**](AddParticipationKey_200_response.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: application/msgpack
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

## append_keys

> crate::models::ParticipationKey append_keys(participation_id, keymap)
Append state proof keys to a participation key

Given a participation ID, append state proof keys to a particular set of participation keys

### Parameters

Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**participation_id** | **String** |  | [required] |
**keymap** | **std::path::PathBuf** | The state proof keys to add to an existing participation ID | [required] |

### Return type

[**crate::models::ParticipationKey**](ParticipationKey.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: application/msgpack
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

## delete_participation_key_by_id

> delete_participation_key_by_id(participation_id)
Delete a given participation key by ID

Delete a given participation key by ID

### Parameters

Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**participation_id** | **String** |  | [required] |

### Return type

 (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

## get_participation_key_by_id

> crate::models::ParticipationKey get_participation_key_by_id(participation_id)
Get participation key info given a participation ID

Given a participation ID, return information about that participation key

### Parameters

Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**participation_id** | **String** |  | [required] |

### Return type

[**crate::models::ParticipationKey**](ParticipationKey.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

## get_participation_keys

> Vec<crate::models::ParticipationKey> get_participation_keys()
Return a list of participation keys

Return a list of participation keys

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<crate::models::ParticipationKey>**](ParticipationKey.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

## shutdown_node

> serde_json::Value shutdown_node(timeout)

Special management endpoint to shutdown the node. Optionally provide a timeout parameter to indicate that the node should begin shutting down after a number of seconds.

### Parameters

Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**timeout** | Option<**u64**> |  |  |[default to 0]

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

## start_catchup

> crate::models::StartCatchup200Response start_catchup(catchpoint)
Starts a catchpoint catchup.

Given a catchpoint, it starts catching up to this catchpoint

### Parameters

Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**catchpoint** | **String** | A catch point | [required] |

### Return type

[**crate::models::StartCatchup200Response**](StartCatchup_200_response.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
