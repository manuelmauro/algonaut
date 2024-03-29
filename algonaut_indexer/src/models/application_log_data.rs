/*
 * Indexer
 *
 * Algorand ledger analytics API.
 *
 * The version of the OpenAPI document: 2.0
 *
 * Generated by: https://openapi-generator.tech
 */

use algonaut_encoding::Bytes;

/// ApplicationLogData : Stores the global information associated with an application.

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ApplicationLogData {
    /// \\[lg\\] Logs for the application being executed by the transaction.
    #[serde(rename = "logs")]
    pub logs: Vec<Bytes>,
    /// Transaction ID
    #[serde(rename = "txid")]
    pub txid: String,
}

impl ApplicationLogData {
    /// Stores the global information associated with an application.
    pub fn new(logs: Vec<Bytes>, txid: String) -> ApplicationLogData {
        ApplicationLogData { logs, txid }
    }
}
