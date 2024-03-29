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
pub struct LookupAccountById200Response {
    #[serde(rename = "account")]
    pub account: Box<crate::models::Account>,
    /// Round at which the results were computed.
    #[serde(rename = "current-round")]
    pub current_round: u64,
}

impl LookupAccountById200Response {
    pub fn new(
        account: crate::models::Account,
        current_round: u64,
    ) -> LookupAccountById200Response {
        LookupAccountById200Response {
            account: Box::new(account),
            current_round,
        }
    }
}
