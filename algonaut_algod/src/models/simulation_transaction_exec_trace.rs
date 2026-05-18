/*
 * Algod REST API.
 *
 * Hand-written model — see ADR
 * `simulaterequest-model-needs-power-pack-fields`. Matches
 * `SimulationTransactionExecTrace` in algod's `algod.oas3.json`.
 */

/// SimulationTransactionExecTrace : Per-transaction execution trace,
/// populated when the caller requested it via `exec-trace-config`.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SimulationTransactionExecTrace {
    /// Base64 SHA-256 hash of the approval program executed (when this
    /// transaction was an app call).
    #[serde(
        rename = "approval-program-hash",
        skip_serializing_if = "Option::is_none"
    )]
    pub approval_program_hash: Option<String>,
    /// Per-opcode trace of the approval program execution.
    #[serde(
        rename = "approval-program-trace",
        skip_serializing_if = "Option::is_none"
    )]
    pub approval_program_trace: Option<Vec<crate::models::SimulationOpcodeTraceUnit>>,
    /// Base64 SHA-256 hash of the clear-state program executed.
    #[serde(
        rename = "clear-state-program-hash",
        skip_serializing_if = "Option::is_none"
    )]
    pub clear_state_program_hash: Option<String>,
    /// Per-opcode trace of the clear-state program execution.
    #[serde(
        rename = "clear-state-program-trace",
        skip_serializing_if = "Option::is_none"
    )]
    pub clear_state_program_trace: Option<Vec<crate::models::SimulationOpcodeTraceUnit>>,
    /// Whether the clear-state program rolled back.
    #[serde(
        rename = "clear-state-rollback",
        skip_serializing_if = "Option::is_none"
    )]
    pub clear_state_rollback: Option<bool>,
    /// If the clear-state program rolled back due to an error, the
    /// error message.
    #[serde(
        rename = "clear-state-rollback-error",
        skip_serializing_if = "Option::is_none"
    )]
    pub clear_state_rollback_error: Option<String>,
    /// Base64 SHA-256 hash of the logic-sig program executed.
    #[serde(rename = "logic-sig-hash", skip_serializing_if = "Option::is_none")]
    pub logic_sig_hash: Option<String>,
    /// Per-opcode trace of the logic-sig execution.
    #[serde(rename = "logic-sig-trace", skip_serializing_if = "Option::is_none")]
    pub logic_sig_trace: Option<Vec<crate::models::SimulationOpcodeTraceUnit>>,
    /// Traces for inner transactions spawned during execution.
    #[serde(rename = "inner-trace", skip_serializing_if = "Option::is_none")]
    pub inner_trace: Option<Vec<SimulationTransactionExecTrace>>,
}

impl SimulationTransactionExecTrace {
    pub fn new() -> SimulationTransactionExecTrace {
        SimulationTransactionExecTrace::default()
    }
}
