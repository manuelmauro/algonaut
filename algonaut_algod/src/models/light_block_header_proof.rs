/*
 * Algod REST API.
 *
 * API endpoint for algod operations.
 *
 * The version of the OpenAPI document: 0.0.1
 * Contact: contact@algorand.com
 * Generated by: https://openapi-generator.tech
 */

/// LightBlockHeaderProof : Proof of membership and position of a light block header.

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct LightBlockHeaderProof {
    /// The index of the light block header in the vector commitment tree
    #[serde(rename = "index")]
    pub index: u64,
    /// The encoded proof.
    #[serde(rename = "proof")]
    pub proof: String,
    /// Represents the depth of the tree that is being proven, i.e. the number of edges from a leaf to the root.
    #[serde(rename = "treedepth")]
    pub treedepth: u64,
}

impl LightBlockHeaderProof {
    /// Proof of membership and position of a light block header.
    pub fn new(index: u64, proof: String, treedepth: u64) -> LightBlockHeaderProof {
        LightBlockHeaderProof {
            index,
            proof,
            treedepth,
        }
    }
}