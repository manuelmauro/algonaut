/*
 * Indexer
 *
 * Algorand ledger analytics API.
 *
 * The version of the OpenAPI document: 2.0
 *
 * Generated by: https://openapi-generator.tech
 */

/// Application : Application index and its parameters

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Application {
    /// Round when this application was created.
    #[serde(rename = "created-at-round", skip_serializing_if = "Option::is_none")]
    pub created_at_round: Option<u64>,
    /// Whether or not this application is currently deleted.
    #[serde(rename = "deleted", skip_serializing_if = "Option::is_none")]
    pub deleted: Option<bool>,
    /// Round when this application was deleted.
    #[serde(rename = "deleted-at-round", skip_serializing_if = "Option::is_none")]
    pub deleted_at_round: Option<u64>,
    /// \\[appidx\\] application index.
    #[serde(rename = "id")]
    pub id: u64,
    #[serde(rename = "params")]
    pub params: Box<crate::models::ApplicationParams>,
}

impl Application {
    /// Application index and its parameters
    pub fn new(id: u64, params: crate::models::ApplicationParams) -> Application {
        Application {
            created_at_round: None,
            deleted: None,
            deleted_at_round: None,
            id,
            params: Box::new(params),
        }
    }
}
