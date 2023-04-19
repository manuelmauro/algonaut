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
pub struct TealDryrun200Response {
    #[serde(rename = "error")]
    pub error: String,
    /// Protocol version is the protocol version Dryrun was operated under.
    #[serde(rename = "protocol-version")]
    pub protocol_version: String,
    #[serde(rename = "txns")]
    pub txns: Vec<crate::models::DryrunTxnResult>,
}

impl TealDryrun200Response {
    pub fn new(error: String, protocol_version: String, txns: Vec<crate::models::DryrunTxnResult>) -> TealDryrun200Response {
        TealDryrun200Response {
            error,
            protocol_version,
            txns,
        }
    }
}


