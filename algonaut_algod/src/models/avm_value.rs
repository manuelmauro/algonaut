/*
 * Algod REST API.
 *
 * Hand-written model — see ADR
 * `simulaterequest-model-needs-power-pack-fields`. Matches `AvmValue`
 * in algod's `algod.oas3.json`.
 */

/// AvmValue : Represents a TEAL value (uint64 or byte[]).
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct AvmValue {
    /// `1` = bytes, `2` = uint64. Other values are reserved.
    #[serde(rename = "type")]
    pub r#type: u64,
    /// Present only when `type == 2`.
    #[serde(rename = "uint", skip_serializing_if = "Option::is_none")]
    pub uint: Option<u64>,
    /// Base64 bytes. Present only when `type == 1`.
    #[serde(rename = "bytes", skip_serializing_if = "Option::is_none")]
    pub bytes: Option<String>,
}

impl AvmValue {
    pub fn new(r#type: u64) -> AvmValue {
        AvmValue {
            r#type,
            uint: None,
            bytes: None,
        }
    }
}
