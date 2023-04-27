/*
 * Indexer
 *
 * Algorand ledger analytics API.
 *
 * The version of the OpenAPI document: 2.0
 *
 * Generated by: https://openapi-generator.tech
 */

/// ApplicationLocalState : Stores local state associated with an application.

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ApplicationLocalState {
    /// Round when account closed out of the application.
    #[serde(
        rename = "closed-out-at-round",
        skip_serializing_if = "Option::is_none"
    )]
    pub closed_out_at_round: Option<u64>,
    /// Whether or not the application local state is currently deleted from its account.
    #[serde(rename = "deleted", skip_serializing_if = "Option::is_none")]
    pub deleted: Option<bool>,
    /// The application which this local state is for.
    #[serde(rename = "id")]
    pub id: u64,
    /// Represents a key-value store for use in an application.
    #[serde(rename = "key-value", skip_serializing_if = "Option::is_none")]
    pub key_value: Option<Vec<crate::models::TealKeyValue>>,
    /// Round when the account opted into the application.
    #[serde(rename = "opted-in-at-round", skip_serializing_if = "Option::is_none")]
    pub opted_in_at_round: Option<u64>,
    #[serde(rename = "schema")]
    pub schema: Box<crate::models::ApplicationStateSchema>,
}

impl ApplicationLocalState {
    /// Stores local state associated with an application.
    pub fn new(id: u64, schema: crate::models::ApplicationStateSchema) -> ApplicationLocalState {
        ApplicationLocalState {
            closed_out_at_round: None,
            deleted: None,
            id,
            key_value: None,
            opted_in_at_round: None,
            schema: Box::new(schema),
        }
    }
}
