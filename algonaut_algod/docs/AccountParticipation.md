# AccountParticipation

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**selection_participation_key** | **String** | \\[sel\\] Selection public key (if any) currently registered for this round. |
**state_proof_key** | Option<**String**> | \\[stprf\\] Root of the state proof key (if any) | [optional]
**vote_first_valid** | **u64** | \\[voteFst\\] First round for which this participation is valid. |
**vote_key_dilution** | **u64** | \\[voteKD\\] Number of subkeys in each batch of participation keys. |
**vote_last_valid** | **u64** | \\[voteLst\\] Last round for which this participation is valid. |
**vote_participation_key** | **String** | \\[vote\\] root participation public key (if any) currently registered for this round. |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
