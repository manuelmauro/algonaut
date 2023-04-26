/*
 * Indexer
 *
 * Algorand ledger analytics API.
 *
 * The version of the OpenAPI document: 2.0
 *
 * Generated by: https://openapi-generator.tech
 */

/// AssetParams : AssetParams specifies the parameters for an asset.  \\[apar\\] when part of an AssetConfig transaction.  Definition: data/transactions/asset.go : AssetParams

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct AssetParams {
    /// \\[c\\] Address of account used to clawback holdings of this asset.  If empty, clawback is not permitted.
    #[serde(rename = "clawback", skip_serializing_if = "Option::is_none")]
    pub clawback: Option<String>,
    /// The address that created this asset. This is the address where the parameters for this asset can be found, and also the address where unwanted asset units can be sent in the worst case.
    #[serde(rename = "creator")]
    pub creator: String,
    /// \\[dc\\] The number of digits to use after the decimal point when displaying this asset. If 0, the asset is not divisible. If 1, the base unit of the asset is in tenths. If 2, the base unit of the asset is in hundredths, and so on. This value must be between 0 and 19 (inclusive).
    #[serde(rename = "decimals")]
    pub decimals: i32,
    /// \\[df\\] Whether holdings of this asset are frozen by default.
    #[serde(rename = "default-frozen", skip_serializing_if = "Option::is_none")]
    pub default_frozen: Option<bool>,
    /// \\[f\\] Address of account used to freeze holdings of this asset.  If empty, freezing is not permitted.
    #[serde(rename = "freeze", skip_serializing_if = "Option::is_none")]
    pub freeze: Option<String>,
    /// \\[m\\] Address of account used to manage the keys of this asset and to destroy it.
    #[serde(rename = "manager", skip_serializing_if = "Option::is_none")]
    pub manager: Option<String>,
    /// \\[am\\] A commitment to some unspecified asset metadata. The format of this metadata is up to the application.
    #[serde(rename = "metadata-hash", skip_serializing_if = "Option::is_none")]
    pub metadata_hash: Option<String>,
    /// \\[an\\] Name of this asset, as supplied by the creator. Included only when the asset name is composed of printable utf-8 characters.
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Base64 encoded name of this asset, as supplied by the creator.
    #[serde(rename = "name-b64", skip_serializing_if = "Option::is_none")]
    pub name_b64: Option<String>,
    /// \\[r\\] Address of account holding reserve (non-minted) units of this asset.
    #[serde(rename = "reserve", skip_serializing_if = "Option::is_none")]
    pub reserve: Option<String>,
    /// \\[t\\] The total number of units of this asset.
    #[serde(rename = "total")]
    pub total: i32,
    /// \\[un\\] Name of a unit of this asset, as supplied by the creator. Included only when the name of a unit of this asset is composed of printable utf-8 characters.
    #[serde(rename = "unit-name", skip_serializing_if = "Option::is_none")]
    pub unit_name: Option<String>,
    /// Base64 encoded name of a unit of this asset, as supplied by the creator.
    #[serde(rename = "unit-name-b64", skip_serializing_if = "Option::is_none")]
    pub unit_name_b64: Option<String>,
    /// \\[au\\] URL where more information about the asset can be retrieved. Included only when the URL is composed of printable utf-8 characters.
    #[serde(rename = "url", skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Base64 encoded URL where more information about the asset can be retrieved.
    #[serde(rename = "url-b64", skip_serializing_if = "Option::is_none")]
    pub url_b64: Option<String>,
}

impl AssetParams {
    /// AssetParams specifies the parameters for an asset.  \\[apar\\] when part of an AssetConfig transaction.  Definition: data/transactions/asset.go : AssetParams
    pub fn new(creator: String, decimals: i32, total: i32) -> AssetParams {
        AssetParams {
            clawback: None,
            creator,
            decimals,
            default_frozen: None,
            freeze: None,
            manager: None,
            metadata_hash: None,
            name: None,
            name_b64: None,
            reserve: None,
            total,
            unit_name: None,
            unit_name_b64: None,
            url: None,
            url_b64: None,
        }
    }
}
