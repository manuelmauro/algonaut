# \PublicApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**account_application_information**](PublicApi.md#account_application_information) | **GET** /v2/accounts/{address}/applications/{application-id} | Get account information about a given app.
[**account_asset_information**](PublicApi.md#account_asset_information) | **GET** /v2/accounts/{address}/assets/{asset-id} | Get account information about a given asset.
[**account_information**](PublicApi.md#account_information) | **GET** /v2/accounts/{address} | Get account information.
[**experimental_check**](PublicApi.md#experimental_check) | **GET** /v2/experimental | Returns OK if experimental API is enabled.
[**get_application_box_by_name**](PublicApi.md#get_application_box_by_name) | **GET** /v2/applications/{application-id}/box | Get box information for a given application.
[**get_application_boxes**](PublicApi.md#get_application_boxes) | **GET** /v2/applications/{application-id}/boxes | Get all box names for a given application.
[**get_application_by_id**](PublicApi.md#get_application_by_id) | **GET** /v2/applications/{application-id} | Get application information.
[**get_asset_by_id**](PublicApi.md#get_asset_by_id) | **GET** /v2/assets/{asset-id} | Get asset information.
[**get_block**](PublicApi.md#get_block) | **GET** /v2/blocks/{round} | Get the block for the given round.
[**get_block_hash**](PublicApi.md#get_block_hash) | **GET** /v2/blocks/{round}/hash | Get the block hash for the block on the given round.
[**get_genesis**](PublicApi.md#get_genesis) | **GET** /genesis | Gets the genesis information.
[**get_ledger_state_delta**](PublicApi.md#get_ledger_state_delta) | **GET** /v2/deltas/{round} | Get a LedgerStateDelta object for a given round
[**get_light_block_header_proof**](PublicApi.md#get_light_block_header_proof) | **GET** /v2/blocks/{round}/lightheader/proof | Gets a proof for a given light block header inside a state proof commitment
[**get_pending_transactions**](PublicApi.md#get_pending_transactions) | **GET** /v2/transactions/pending | Get a list of unconfirmed transactions currently in the transaction pool.
[**get_pending_transactions_by_address**](PublicApi.md#get_pending_transactions_by_address) | **GET** /v2/accounts/{address}/transactions/pending | Get a list of unconfirmed transactions currently in the transaction pool by address.
[**get_ready**](PublicApi.md#get_ready) | **GET** /ready | Returns OK if healthy and fully caught up.
[**get_state_proof**](PublicApi.md#get_state_proof) | **GET** /v2/stateproofs/{round} | Get a state proof that covers a given round
[**get_status**](PublicApi.md#get_status) | **GET** /v2/status | Gets the current node status.
[**get_supply**](PublicApi.md#get_supply) | **GET** /v2/ledger/supply | Get the current supply reported by the ledger.
[**get_sync_round**](PublicApi.md#get_sync_round) | **GET** /v2/ledger/sync | Returns the minimum sync round the ledger is keeping in cache.
[**get_transaction_proof**](PublicApi.md#get_transaction_proof) | **GET** /v2/blocks/{round}/transactions/{txid}/proof | Get a proof for a transaction in a block.
[**get_version**](PublicApi.md#get_version) | **GET** /versions | 
[**health_check**](PublicApi.md#health_check) | **GET** /health | Returns OK if healthy.
[**metrics**](PublicApi.md#metrics) | **GET** /metrics | Return metrics about algod functioning.
[**pending_transaction_information**](PublicApi.md#pending_transaction_information) | **GET** /v2/transactions/pending/{txid} | Get a specific pending transaction.
[**raw_transaction**](PublicApi.md#raw_transaction) | **POST** /v2/transactions | Broadcasts a raw transaction or transaction group to the network.
[**set_sync_round**](PublicApi.md#set_sync_round) | **POST** /v2/ledger/sync/{round} | Given a round, tells the ledger to keep that round in its cache.
[**simulate_transaction**](PublicApi.md#simulate_transaction) | **POST** /v2/transactions/simulate | Simulates a raw transaction or transaction group as it would be evaluated on the network. The simulation will use blockchain state from the latest committed round.
[**swagger_json**](PublicApi.md#swagger_json) | **GET** /swagger.json | Gets the current swagger spec.
[**teal_compile**](PublicApi.md#teal_compile) | **POST** /v2/teal/compile | Compile TEAL source code to binary, produce its hash
[**teal_disassemble**](PublicApi.md#teal_disassemble) | **POST** /v2/teal/disassemble | Disassemble program bytes into the TEAL source code.
[**teal_dryrun**](PublicApi.md#teal_dryrun) | **POST** /v2/teal/dryrun | Provide debugging information for a transaction (or group).
[**transaction_params**](PublicApi.md#transaction_params) | **GET** /v2/transactions/params | Get parameters for constructing a new transaction
[**unset_sync_round**](PublicApi.md#unset_sync_round) | **DELETE** /v2/ledger/sync | Removes minimum sync round restriction from the ledger.
[**wait_for_block**](PublicApi.md#wait_for_block) | **GET** /v2/status/wait-for-block-after/{round} | Gets the node status after waiting for a round after the given round.



## account_application_information

> crate::models::AccountApplicationInformation200Response account_application_information(address, application_id, format)
Get account information about a given app.

Given a specific account public key and application ID, this call returns the account's application local state and global state (AppLocalState and AppParams, if either exists). Global state will only be returned if the provided address is the application's creator.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**address** | **String** | An account public key | [required] |
**application_id** | **i32** | An application identifier | [required] |
**format** | Option<**String**> | Configures whether the response object is JSON or MessagePack encoded. If not provided, defaults to JSON. |  |

### Return type

[**crate::models::AccountApplicationInformation200Response**](AccountApplicationInformation_200_response.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/msgpack

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## account_asset_information

> crate::models::AccountAssetInformation200Response account_asset_information(address, asset_id, format)
Get account information about a given asset.

Given a specific account public key and asset ID, this call returns the account's asset holding and asset parameters (if either exist). Asset parameters will only be returned if the provided address is the asset's creator.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**address** | **String** | An account public key | [required] |
**asset_id** | **i32** | An asset identifier | [required] |
**format** | Option<**String**> | Configures whether the response object is JSON or MessagePack encoded. If not provided, defaults to JSON. |  |

### Return type

[**crate::models::AccountAssetInformation200Response**](AccountAssetInformation_200_response.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/msgpack

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## account_information

> crate::models::Account account_information(address, format, exclude)
Get account information.

Given a specific account public key, this call returns the accounts status, balance and spendable amounts

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**address** | **String** | An account public key | [required] |
**format** | Option<**String**> | Configures whether the response object is JSON or MessagePack encoded. If not provided, defaults to JSON. |  |
**exclude** | Option<**String**> | When set to `all` will exclude asset holdings, application local state, created asset parameters, any created application parameters. Defaults to `none`. |  |

### Return type

[**crate::models::Account**](Account.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/msgpack

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


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


## get_application_box_by_name

> crate::models::Box get_application_box_by_name(application_id, name)
Get box information for a given application.

Given an application ID and box name, it returns the box name and value (each base64 encoded). Box names must be in the goal app call arg encoding form 'encoding:value'. For ints, use the form 'int:1234'. For raw bytes, use the form 'b64:A=='. For printable strings, use the form 'str:hello'. For addresses, use the form 'addr:XYZ...'.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**application_id** | **i32** | An application identifier | [required] |
**name** | **String** | A box name, in the goal app call arg form 'encoding:value'. For ints, use the form 'int:1234'. For raw bytes, use the form 'b64:A=='. For printable strings, use the form 'str:hello'. For addresses, use the form 'addr:XYZ...'. | [required] |

### Return type

[**crate::models::Box**](Box.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_application_boxes

> crate::models::GetApplicationBoxes200Response get_application_boxes(application_id, max)
Get all box names for a given application.

Given an application ID, return all Box names. No particular ordering is guaranteed. Request fails when client or server-side configured limits prevent returning all Box names.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**application_id** | **i32** | An application identifier | [required] |
**max** | Option<**i32**> | Max number of box names to return. If max is not set, or max == 0, returns all box-names. |  |

### Return type

[**crate::models::GetApplicationBoxes200Response**](GetApplicationBoxes_200_response.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_application_by_id

> crate::models::Application get_application_by_id(application_id)
Get application information.

Given a application ID, it returns application information including creator, approval and clear programs, global and local schemas, and global state.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**application_id** | **i32** | An application identifier | [required] |

### Return type

[**crate::models::Application**](Application.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_asset_by_id

> crate::models::Asset get_asset_by_id(asset_id)
Get asset information.

Given a asset ID, it returns asset information including creator, name, total supply and special addresses.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**asset_id** | **i32** | An asset identifier | [required] |

### Return type

[**crate::models::Asset**](Asset.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_block

> crate::models::GetBlock200Response get_block(round, format)
Get the block for the given round.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**round** | **i32** | The round from which to fetch block information. | [required] |
**format** | Option<**String**> | Configures whether the response object is JSON or MessagePack encoded. If not provided, defaults to JSON. |  |

### Return type

[**crate::models::GetBlock200Response**](GetBlock_200_response.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/msgpack

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_block_hash

> crate::models::GetBlockHash200Response get_block_hash(round)
Get the block hash for the block on the given round.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**round** | **i32** | The round from which to fetch block hash information. | [required] |

### Return type

[**crate::models::GetBlockHash200Response**](GetBlockHash_200_response.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


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


## get_light_block_header_proof

> crate::models::LightBlockHeaderProof get_light_block_header_proof(round)
Gets a proof for a given light block header inside a state proof commitment

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**round** | **i32** | The round to which the light block header belongs. | [required] |

### Return type

[**crate::models::LightBlockHeaderProof**](LightBlockHeaderProof.md)

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
**max** | Option<**i32**> | Truncated number of transactions to display. If max=0, returns all pending txns. |  |
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
**max** | Option<**i32**> | Truncated number of transactions to display. If max=0, returns all pending txns. |  |
**format** | Option<**String**> | Configures whether the response object is JSON or MessagePack encoded. If not provided, defaults to JSON. |  |

### Return type

[**crate::models::GetPendingTransactionsByAddress200Response**](GetPendingTransactionsByAddress_200_response.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/msgpack

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


## get_state_proof

> crate::models::StateProof get_state_proof(round)
Get a state proof that covers a given round

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**round** | **i32** | The round for which a state proof is desired. | [required] |

### Return type

[**crate::models::StateProof**](StateProof.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_status

> crate::models::GetStatus200Response get_status()
Gets the current node status.

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::GetStatus200Response**](GetStatus_200_response.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_supply

> crate::models::GetSupply200Response get_supply()
Get the current supply reported by the ledger.

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::GetSupply200Response**](GetSupply_200_response.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

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


## get_transaction_proof

> crate::models::GetTransactionProof200Response get_transaction_proof(round, txid, hashtype, format)
Get a proof for a transaction in a block.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**round** | **i32** | The round in which the transaction appears. | [required] |
**txid** | **String** | The transaction ID for which to generate a proof. | [required] |
**hashtype** | Option<**String**> | The type of hash function used to create the proof, must be one of:  * sha512_256  * sha256 |  |
**format** | Option<**String**> | Configures whether the response object is JSON or MessagePack encoded. If not provided, defaults to JSON. |  |

### Return type

[**crate::models::GetTransactionProof200Response**](GetTransactionProof_200_response.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

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


## simulate_transaction

> crate::models::SimulateTransaction200Response simulate_transaction(request, format)
Simulates a raw transaction or transaction group as it would be evaluated on the network. The simulation will use blockchain state from the latest committed round.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**request** | [**SimulateRequest**](SimulateRequest.md) | The transactions to simulate, along with any other inputs. | [required] |
**format** | Option<**String**> | Configures whether the response object is JSON or MessagePack encoded. If not provided, defaults to JSON. |  |

### Return type

[**crate::models::SimulateTransaction200Response**](SimulateTransaction_200_response.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: application/json, application/msgpack
- **Accept**: application/json, application/msgpack

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


## teal_compile

> crate::models::TealCompile200Response teal_compile(source, sourcemap)
Compile TEAL source code to binary, produce its hash

Given TEAL source code in plain text, return base64 encoded program bytes and base32 SHA512_256 hash of program bytes (Address style). This endpoint is only enabled when a node's configuration file sets EnableDeveloperAPI to true.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**source** | **std::path::PathBuf** | TEAL source code to be compiled | [required] |
**sourcemap** | Option<**bool**> | When set to `true`, returns the source map of the program as a JSON. Defaults to `false`. |  |

### Return type

[**crate::models::TealCompile200Response**](TealCompile_200_response.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: text/plain
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## teal_disassemble

> crate::models::TealDisassemble200Response teal_disassemble(source)
Disassemble program bytes into the TEAL source code.

Given the program bytes, return the TEAL source code in plain text. This endpoint is only enabled when a node's configuration file sets EnableDeveloperAPI to true.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**source** | **String** | TEAL program binary to be disassembled | [required] |

### Return type

[**crate::models::TealDisassemble200Response**](TealDisassemble_200_response.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: application/x-binary
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## teal_dryrun

> crate::models::TealDryrun200Response teal_dryrun(request)
Provide debugging information for a transaction (or group).

Executes TEAL program(s) in context and returns debugging information about the execution. This endpoint is only enabled when a node's configuration file sets EnableDeveloperAPI to true.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**request** | Option<[**DryrunRequest**](DryrunRequest.md)> | Transaction (or group) and any accompanying state-simulation data. |  |

### Return type

[**crate::models::TealDryrun200Response**](TealDryrun_200_response.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: application/json, application/msgpack
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## transaction_params

> crate::models::TransactionParams200Response transaction_params()
Get parameters for constructing a new transaction

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::TransactionParams200Response**](TransactionParams_200_response.md)

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


## wait_for_block

> crate::models::GetStatus200Response wait_for_block(round)
Gets the node status after waiting for a round after the given round.

Waits for a block to appear after round {round} and returns the node's status at the time.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**round** | **i32** | The round to wait until returning status | [required] |

### Return type

[**crate::models::GetStatus200Response**](GetStatus_200_response.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

