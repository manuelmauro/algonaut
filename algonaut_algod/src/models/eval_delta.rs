/*
 * Algod REST API.
 *
 * API endpoint for algod operations.
 *
 * The version of the OpenAPI document: 0.0.1
 * Contact: contact@algorand.com
 * Generated by: https://openapi-generator.tech
 */

/// EvalDelta : Represents a TEAL value delta.

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct EvalDelta {
    /// \\[at\\] delta action.
    #[serde(rename = "action")]
    pub action: u64,
    /// \\[bs\\] bytes value.
    #[serde(rename = "bytes", skip_serializing_if = "Option::is_none")]
    pub bytes: Option<String>,
    /// \\[ui\\] uint value.
    #[serde(rename = "uint", skip_serializing_if = "Option::is_none")]
    pub uint: Option<u64>,
}

impl EvalDelta {
    /// Represents a TEAL value delta.
    pub fn new(action: u64) -> EvalDelta {
        EvalDelta {
            action,
            bytes: None,
            uint: None,
        }
    }
}
