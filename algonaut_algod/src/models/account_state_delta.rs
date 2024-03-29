/*
 * Algod REST API.
 *
 * API endpoint for algod operations.
 *
 * The version of the OpenAPI document: 0.0.1
 * Contact: contact@algorand.com
 * Generated by: https://openapi-generator.tech
 */

/// AccountStateDelta : Application state delta.

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct AccountStateDelta {
    #[serde(rename = "address")]
    pub address: String,
    /// Application state delta.
    #[serde(rename = "delta")]
    pub delta: Vec<crate::models::EvalDeltaKeyValue>,
}

impl AccountStateDelta {
    /// Application state delta.
    pub fn new(address: String, delta: Vec<crate::models::EvalDeltaKeyValue>) -> AccountStateDelta {
        AccountStateDelta { address, delta }
    }
}
