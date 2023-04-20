# \ParticipatingApi

All URIs are relative to *<http://localhost>*

Method | HTTP request | Description
------------- | ------------- | -------------
[**add_participation_key**](ParticipatingApi.md#add_participation_key) | **POST** /v2/participation | Add a participation key to the node
[**append_keys**](ParticipatingApi.md#append_keys) | **POST** /v2/participation/{participation-id} | Append state proof keys to a participation key
[**delete_participation_key_by_id**](ParticipatingApi.md#delete_participation_key_by_id) | **DELETE** /v2/participation/{participation-id} | Delete a given participation key by ID
[**get_participation_key_by_id**](ParticipatingApi.md#get_participation_key_by_id) | **GET** /v2/participation/{participation-id} | Get participation key info given a participation ID
[**get_participation_keys**](ParticipatingApi.md#get_participation_keys) | **GET** /v2/participation | Return a list of participation keys
[**get_pending_transactions**](ParticipatingApi.md#get_pending_transactions) | **GET** /v2/transactions/pending | Get a list of unconfirmed transactions currently in the transaction pool.
[**get_pending_transactions_by_address**](ParticipatingApi.md#get_pending_transactions_by_address) | **GET** /v2/accounts/{address}/transactions/pending | Get a list of unconfirmed transactions currently in the transaction pool by address.
[**pending_transaction_information**](ParticipatingApi.md#pending_transaction_information) | **GET** /v2/transactions/pending/{txid} | Get a specific pending transaction.
[**raw_transaction**](ParticipatingApi.md#raw_transaction) | **POST** /v2/transactions | Broadcasts a raw transaction or transaction group to the network.

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

## get_pending_transactions

> crate::models::GetPendingTransactionsByAddress200Response get_pending_transactions(max, format)
Get a list of unconfirmed transactions currently in the transaction pool.

Get the list of pending transactions, sorted by priority, in decreasing order, truncated at the end at MAX. If MAX = 0, returns all pending transactions.

### Parameters

Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**max** | Option<**u64**> | Truncated number of transactions to display. If max=0, returns all pending txns. |  |
**format** | Option<**String**> | Configures whether the response object is JSON or MessagePack encoded. If not provided, defaults to JSON. |  |

### Return type

[**crate::models::GetPendingTransactionsByAddress200Response**](GetPendingTransactionsByAddress_200_response.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/msgpack

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

## get_pending_transactions_by_address

> crate::models::GetPendingTransactionsByAddress200Response get_pending_transactions_by_address(address, max, format)
Get a list of unconfirmed transactions currently in the transaction pool by address.

Get the list of pending transactions by address, sorted by priority, in decreasing order, truncated at the end at MAX. If MAX = 0, returns all pending transactions.

### Parameters

Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**address** | **String** | An account public key | [required] |
**max** | Option<**u64**> | Truncated number of transactions to display. If max=0, returns all pending txns. |  |
**format** | Option<**String**> | Configures whether the response object is JSON or MessagePack encoded. If not provided, defaults to JSON. |  |

### Return type

[**crate::models::GetPendingTransactionsByAddress200Response**](GetPendingTransactionsByAddress_200_response.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/msgpack

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

## pending_transaction_information

> crate::models::PendingTransactionResponse pending_transaction_information(txid, format)
Get a specific pending transaction.

Given a transaction ID of a recently submitted transaction, it returns information about it.  There are several cases when this might succeed: - transaction committed (committed round > 0) - transaction still in the pool (committed round = 0, pool error = \"\") - transaction removed from pool due to error (committed round = 0, pool error != \"\") Or the transaction may have happened sufficiently long ago that the node no longer remembers it, and this will return an error.

### Parameters

Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**txid** | **String** | A transaction ID | [required] |
**format** | Option<**String**> | Configures whether the response object is JSON or MessagePack encoded. If not provided, defaults to JSON. |  |

### Return type

[**crate::models::PendingTransactionResponse**](PendingTransactionResponse.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/msgpack

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

## raw_transaction

> crate::models::RawTransaction200Response raw_transaction(rawtxn)
Broadcasts a raw transaction or transaction group to the network.

### Parameters

Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**rawtxn** | **std::path::PathBuf** | The byte encoded signed transaction to broadcast to network | [required] |

### Return type

[**crate::models::RawTransaction200Response**](RawTransaction_200_response.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: application/x-binary
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
