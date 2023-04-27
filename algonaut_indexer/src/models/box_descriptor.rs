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

/// BoxDescriptor : Box descriptor describes an app box without a value.

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct BoxDescriptor {
    /// Base64 encoded box name
    #[serde(rename = "name")]
    pub name: Bytes,
}

impl BoxDescriptor {
    /// Box descriptor describes an app box without a value.
    pub fn new(name: Bytes) -> BoxDescriptor {
        BoxDescriptor { name }
    }
}
