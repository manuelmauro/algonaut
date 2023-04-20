# GetTransactionProof200Response

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**hashtype** | **String** | The type of hash function used to create the proof, must be one of:  *sha512_256* sha256 |
**idx** | **u64** | Index of the transaction in the block's payset. |
**proof** | **String** | Proof of transaction membership. |
**stibhash** | **String** | Hash of SignedTxnInBlock for verifying proof. |
**treedepth** | **u64** | Represents the depth of the tree that is being proven, i.e. the number of edges from a leaf to the root. |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
