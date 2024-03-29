/*
 * Algod REST API.
 *
 * API endpoint for algod operations.
 *
 * The version of the OpenAPI document: 0.0.1
 * Contact: contact@algorand.com
 * Generated by: https://openapi-generator.tech
 */

/// ApplicationLocalState : Stores local state associated with an application.

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ApplicationLocalState {
    /// The application which this local state is for.
    #[serde(rename = "id")]
    pub id: u64,
    /// Represents a key-value store for use in an application.
    #[serde(rename = "key-value", skip_serializing_if = "Option::is_none")]
    pub key_value: Option<Vec<crate::models::TealKeyValue>>,
    #[serde(rename = "schema")]
    pub schema: Box<crate::models::ApplicationStateSchema>,
}

impl ApplicationLocalState {
    /// Stores local state associated with an application.
    pub fn new(id: u64, schema: crate::models::ApplicationStateSchema) -> ApplicationLocalState {
        ApplicationLocalState {
            id,
            key_value: None,
            schema: Box::new(schema),
        }
    }
}
