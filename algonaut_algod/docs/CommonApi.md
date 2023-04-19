# \CommonApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_genesis**](CommonApi.md#get_genesis) | **GET** /genesis | Gets the genesis information.
[**get_ready**](CommonApi.md#get_ready) | **GET** /ready | Returns OK if healthy and fully caught up.
[**get_version**](CommonApi.md#get_version) | **GET** /versions | 
[**health_check**](CommonApi.md#health_check) | **GET** /health | Returns OK if healthy.
[**metrics**](CommonApi.md#metrics) | **GET** /metrics | Return metrics about algod functioning.
[**swagger_json**](CommonApi.md#swagger_json) | **GET** /swagger.json | Gets the current swagger spec.



## get_genesis

> String get_genesis()
Gets the genesis information.

Returns the entire genesis file in json.

### Parameters

This endpoint does not need any parameter.

### Return type

**String**

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_ready

> get_ready()
Returns OK if healthy and fully caught up.

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


## get_version

> crate::models::Version get_version()


Retrieves the supported API versions, binary build versions, and genesis information.

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::Version**](Version.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## health_check

> health_check()
Returns OK if healthy.

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


## metrics

> metrics()
Return metrics about algod functioning.

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


## swagger_json

> String swagger_json()
Gets the current swagger spec.

Returns the entire swagger spec in json.

### Parameters

This endpoint does not need any parameter.

### Return type

**String**

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

