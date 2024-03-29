/*
 * Indexer
 *
 * Algorand ledger analytics API.
 *
 * The version of the OpenAPI document: 2.0
 *
 * Generated by: https://openapi-generator.tech
 */

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct LookupAccountCreatedApplications200Response {
    #[serde(rename = "applications")]
    pub applications: Vec<crate::models::Application>,
    /// Round at which the results were computed.
    #[serde(rename = "current-round")]
    pub current_round: u64,
    /// Used for pagination, when making another request provide this token with the next parameter.
    #[serde(rename = "next-token", skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

impl LookupAccountCreatedApplications200Response {
    pub fn new(
        applications: Vec<crate::models::Application>,
        current_round: u64,
    ) -> LookupAccountCreatedApplications200Response {
        LookupAccountCreatedApplications200Response {
            applications,
            current_round,
            next_token: None,
        }
    }
}
