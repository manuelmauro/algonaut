/*
 * Indexer
 *
 * Algorand ledger analytics API.
 *
 * The version of the OpenAPI document: 2.0
 *
 * Generated by: https://openapi-generator.tech
 */

/// Asset : Specifies both the unique identifier and the parameters for an asset

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Asset {
    /// Round during which this asset was created.
    #[serde(rename = "created-at-round", skip_serializing_if = "Option::is_none")]
    pub created_at_round: Option<i32>,
    /// Whether or not this asset is currently deleted.
    #[serde(rename = "deleted", skip_serializing_if = "Option::is_none")]
    pub deleted: Option<bool>,
    /// Round during which this asset was destroyed.
    #[serde(rename = "destroyed-at-round", skip_serializing_if = "Option::is_none")]
    pub destroyed_at_round: Option<i32>,
    /// unique asset identifier
    #[serde(rename = "index")]
    pub index: i32,
    #[serde(rename = "params")]
    pub params: Box<crate::models::AssetParams>,
}

impl Asset {
    /// Specifies both the unique identifier and the parameters for an asset
    pub fn new(index: i32, params: crate::models::AssetParams) -> Asset {
        Asset {
            created_at_round: None,
            deleted: None,
            destroyed_at_round: None,
            index,
            params: Box::new(params),
        }
    }
}
