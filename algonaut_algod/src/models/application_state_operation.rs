/*
 * Algod REST API.
 *
 * Hand-written model — see ADR
 * `simulaterequest-model-needs-power-pack-fields`. Matches
 * `ApplicationStateOperation` in algod's `algod.oas3.json`.
 */

/// ApplicationStateOperation : Records one read/write/delete against an
/// application's global, local, or box state.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ApplicationStateOperation {
    /// One of `"g"` (global), `"l"` (local), `"b"` (box).
    #[serde(rename = "app-state-type")]
    pub app_state_type: String,
    /// Base64 key.
    #[serde(rename = "key")]
    pub key: String,
    /// One of `"w"` (write) or `"d"` (delete).
    #[serde(rename = "operation")]
    pub operation: String,
    /// Required for local-state operations: the address whose local
    /// state was touched.
    #[serde(rename = "account", skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    /// For writes, the new value.
    #[serde(rename = "new-value", skip_serializing_if = "Option::is_none")]
    pub new_value: Option<Box<crate::models::AvmValue>>,
}

impl ApplicationStateOperation {
    pub fn new(
        app_state_type: String,
        key: String,
        operation: String,
    ) -> ApplicationStateOperation {
        ApplicationStateOperation {
            app_state_type,
            key,
            operation,
            account: None,
            new_value: None,
        }
    }
}
