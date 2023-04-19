# AssetParams

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**clawback** | Option<**String**> | \\[c\\] Address of account used to clawback holdings of this asset.  If empty, clawback is not permitted. | [optional]
**creator** | **String** | The address that created this asset. This is the address where the parameters for this asset can be found, and also the address where unwanted asset units can be sent in the worst case. | 
**decimals** | **i32** | \\[dc\\] The number of digits to use after the decimal point when displaying this asset. If 0, the asset is not divisible. If 1, the base unit of the asset is in tenths. If 2, the base unit of the asset is in hundredths, and so on. This value must be between 0 and 19 (inclusive). | 
**default_frozen** | Option<**bool**> | \\[df\\] Whether holdings of this asset are frozen by default. | [optional]
**freeze** | Option<**String**> | \\[f\\] Address of account used to freeze holdings of this asset.  If empty, freezing is not permitted. | [optional]
**manager** | Option<**String**> | \\[m\\] Address of account used to manage the keys of this asset and to destroy it. | [optional]
**metadata_hash** | Option<**String**> | \\[am\\] A commitment to some unspecified asset metadata. The format of this metadata is up to the application. | [optional]
**name** | Option<**String**> | \\[an\\] Name of this asset, as supplied by the creator. Included only when the asset name is composed of printable utf-8 characters. | [optional]
**name_b64** | Option<**String**> | Base64 encoded name of this asset, as supplied by the creator. | [optional]
**reserve** | Option<**String**> | \\[r\\] Address of account holding reserve (non-minted) units of this asset. | [optional]
**total** | **i32** | \\[t\\] The total number of units of this asset. | 
**unit_name** | Option<**String**> | \\[un\\] Name of a unit of this asset, as supplied by the creator. Included only when the name of a unit of this asset is composed of printable utf-8 characters. | [optional]
**unit_name_b64** | Option<**String**> | Base64 encoded name of a unit of this asset, as supplied by the creator. | [optional]
**url** | Option<**String**> | \\[au\\] URL where more information about the asset can be retrieved. Included only when the URL is composed of printable utf-8 characters. | [optional]
**url_b64** | Option<**String**> | Base64 encoded URL where more information about the asset can be retrieved. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


