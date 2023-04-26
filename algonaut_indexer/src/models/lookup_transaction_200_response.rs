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
pub struct LookupTransaction200Response {
    /// Round at which the results were computed.
    #[serde(rename = "current-round")]
    pub current_round: i32,
    #[serde(rename = "transaction")]
    pub transaction: Box<crate::models::Transaction>,
}

impl LookupTransaction200Response {
    pub fn new(
        current_round: i32,
        transaction: crate::models::Transaction,
    ) -> LookupTransaction200Response {
        LookupTransaction200Response {
            current_round,
            transaction: Box::new(transaction),
        }
    }
}
