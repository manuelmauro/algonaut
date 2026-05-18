/*
 * Algod REST API.
 *
 * API endpoint for algod operations.
 *
 * The version of the OpenAPI document: 0.0.1
 * Contact: contact@algorand.com
 *
 * Extended by hand for the simulate exec-trace and fixed-signer fields
 * — see ADR `simulaterequest-model-needs-power-pack-fields`.
 */

/// SimulateTransactionResult : Simulation result for an individual transaction

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SimulateTransactionResult {
    /// Budget used during execution of an app call transaction. This value includes budged used by inner app calls spawned by this transaction.
    #[serde(
        rename = "app-budget-consumed",
        skip_serializing_if = "Option::is_none"
    )]
    pub app_budget_consumed: Option<u64>,
    /// Budget used during execution of a logic sig transaction.
    #[serde(
        rename = "logic-sig-budget-consumed",
        skip_serializing_if = "Option::is_none"
    )]
    pub logic_sig_budget_consumed: Option<u64>,
    /// A boolean indicating whether this transaction is missing signatures
    #[serde(rename = "missing-signature", skip_serializing_if = "Option::is_none")]
    pub missing_signature: Option<bool>,
    /// Per-transaction execution trace; only present when the simulate
    /// request enabled `exec-trace-config`.
    #[serde(rename = "exec-trace", skip_serializing_if = "Option::is_none")]
    pub exec_trace: Option<Box<crate::models::SimulationTransactionExecTrace>>,
    /// If the sender was rekeyed and the simulator replayed the txn
    /// with the auth-address as the signer, this is the address used.
    #[serde(rename = "fixed-signer", skip_serializing_if = "Option::is_none")]
    pub fixed_signer: Option<String>,
    #[serde(rename = "txn-result")]
    pub txn_result: Box<crate::models::PendingTransactionResponse>,
}

impl SimulateTransactionResult {
    /// Simulation result for an individual transaction
    pub fn new(txn_result: crate::models::PendingTransactionResponse) -> SimulateTransactionResult {
        SimulateTransactionResult {
            app_budget_consumed: None,
            logic_sig_budget_consumed: None,
            missing_signature: None,
            exec_trace: None,
            fixed_signer: None,
            txn_result: Box::new(txn_result),
        }
    }
}
