/*
 * Algod REST API.
 *
 * API endpoint for algod operations.
 *
 * The version of the OpenAPI document: 0.0.1
 * Contact: contact@algorand.com
 * Generated by: https://openapi-generator.tech
 */

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct AccountAssetInformation200Response {
    #[serde(rename = "asset-holding", skip_serializing_if = "Option::is_none")]
    pub asset_holding: Option<Box<crate::models::AssetHolding>>,
    #[serde(rename = "created-asset", skip_serializing_if = "Option::is_none")]
    pub created_asset: Option<Box<crate::models::AssetParams>>,
    /// The round for which this information is relevant.
    #[serde(rename = "round")]
    pub round: u64,
}

impl AccountAssetInformation200Response {
    pub fn new(round: u64) -> AccountAssetInformation200Response {
        AccountAssetInformation200Response {
            asset_holding: None,
            created_asset: None,
            round,
        }
    }
}
