/*
 * Algod REST API.
 *
 * Hand-edited: the OpenAPI generator emitted `txns: Vec<String>` (base64
 * blobs) as a placeholder for the heterogeneous `SignedTxn` value, but
 * algod's `/v2/transactions/simulate` endpoint expects each `txns[i]` to
 * be the nested `ApiSignedTransaction` object directly — not a base64
 * string. See ADR `simulaterequest-model-needs-power-pack-fields` for
 * the broader simulate-model rework.
 */

use serde::{Deserialize, Serialize};

use crate::transaction::ApiSignedTransaction;

/// SimulateRequestTransactionGroup : A transaction group to simulate.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SimulateRequestTransactionGroup {
    /// An atomic transaction group. Each entry is a full
    /// `ApiSignedTransaction` — the simulator inspects the inner
    /// transaction fields directly.
    #[serde(rename = "txns")]
    pub txns: Vec<ApiSignedTransaction>,
}

impl SimulateRequestTransactionGroup {
    /// A transaction group to simulate.
    pub fn new(txns: Vec<ApiSignedTransaction>) -> SimulateRequestTransactionGroup {
        SimulateRequestTransactionGroup { txns }
    }
}
