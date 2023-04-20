# StateProofMessage

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**block_headers_commitment** | **String** | The vector commitment root on all light block headers within a state proof interval. |
**first_attested_round** | **u64** | The first round the message attests to. |
**last_attested_round** | **u64** | The last round the message attests to. |
**ln_proven_weight** | **u64** | An integer value representing the natural log of the proven weight with 16 bits of precision. This value would be used to verify the next state proof. |
**voters_commitment** | **String** | The vector commitment root of the top N accounts to sign the next StateProof. |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
