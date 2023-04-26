/*
 * Indexer
 *
 * Algorand ledger analytics API.
 *
 * The version of the OpenAPI document: 2.0
 *
 * Generated by: https://openapi-generator.tech
 */

/// TransactionSignatureMultisig : \\[msig\\] structure holding multiple subsignatures.  Definition: crypto/multisig.go : MultisigSig

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct TransactionSignatureMultisig {
    /// \\[subsig\\] holds pairs of public key and signatures.
    #[serde(rename = "subsignature", skip_serializing_if = "Option::is_none")]
    pub subsignature: Option<Vec<crate::models::TransactionSignatureMultisigSubsignature>>,
    /// \\[thr\\]
    #[serde(rename = "threshold", skip_serializing_if = "Option::is_none")]
    pub threshold: Option<i32>,
    /// \\[v\\]
    #[serde(rename = "version", skip_serializing_if = "Option::is_none")]
    pub version: Option<i32>,
}

impl TransactionSignatureMultisig {
    /// \\[msig\\] structure holding multiple subsignatures.  Definition: crypto/multisig.go : MultisigSig
    pub fn new() -> TransactionSignatureMultisig {
        TransactionSignatureMultisig {
            subsignature: None,
            threshold: None,
            version: None,
        }
    }
}
