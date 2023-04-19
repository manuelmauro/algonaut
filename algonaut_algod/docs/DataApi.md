# \DataApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_ledger_state_delta**](DataApi.md#get_ledger_state_delta) | **GET** /v2/deltas/{round} | Get a LedgerStateDelta object for a given round
[**get_sync_round**](DataApi.md#get_sync_round) | **GET** /v2/ledger/sync | Returns the minimum sync round the ledger is keeping in cache.
[**set_sync_round**](DataApi.md#set_sync_round) | **POST** /v2/ledger/sync/{round} | Given a round, tells the ledger to keep that round in its cache.
[**unset_sync_round**](DataApi.md#unset_sync_round) | **DELETE** /v2/ledger/sync | Removes minimum sync round restriction from the ledger.



## get_ledger_state_delta

> serde_json::Value get_ledger_state_delta(round, format)
Get a LedgerStateDelta object for a given round

Get ledger deltas for a round.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**round** | **i32** | The round for which the deltas are desired. | [required] |
**format** | Option<**String**> | Configures whether the response object is JSON or MessagePack encoded. If not provided, defaults to JSON. |  |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/msgpack

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_sync_round

> crate::models::GetSyncRound200Response get_sync_round()
Returns the minimum sync round the ledger is keeping in cache.

Gets the minimum sync round for the ledger.

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::GetSyncRound200Response**](GetSyncRound_200_response.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## set_sync_round

> set_sync_round(round)
Given a round, tells the ledger to keep that round in its cache.

Sets the minimum sync round on the ledger.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**round** | **i32** | The round for which the deltas are desired. | [required] |

### Return type

 (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## unset_sync_round

> unset_sync_round()
Removes minimum sync round restriction from the ledger.

Unset the ledger sync round.

### Parameters

This endpoint does not need any parameter.

### Return type

 (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

