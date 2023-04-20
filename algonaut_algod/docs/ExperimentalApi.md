# \ExperimentalApi

All URIs are relative to *<http://localhost>*

Method | HTTP request | Description
------------- | ------------- | -------------
[**experimental_check**](ExperimentalApi.md#experimental_check) | **GET** /v2/experimental | Returns OK if experimental API is enabled.
[**get_ledger_state_delta**](ExperimentalApi.md#get_ledger_state_delta) | **GET** /v2/deltas/{round} | Get a LedgerStateDelta object for a given round

## experimental_check

> experimental_check()
Returns OK if experimental API is enabled.

### Parameters

This endpoint does not need any parameter.

### Return type

 (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

## get_ledger_state_delta

> serde_json::Value get_ledger_state_delta(round, format)
Get a LedgerStateDelta object for a given round

Get ledger deltas for a round.

### Parameters

Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**round** | **u64** | The round for which the deltas are desired. | [required] |
**format** | Option<**String**> | Configures whether the response object is JSON or MessagePack encoded. If not provided, defaults to JSON. |  |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/msgpack

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
