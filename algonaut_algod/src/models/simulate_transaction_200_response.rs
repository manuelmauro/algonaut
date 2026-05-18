/*
 * Algod REST API.
 *
 * API endpoint for algod operations.
 *
 * The version of the OpenAPI document: 0.0.1
 * Contact: contact@algorand.com
 *
 * Extended by hand for the simulate eval-overrides and exec-trace
 * fields — see ADR `simulaterequest-model-needs-power-pack-fields`.
 */

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SimulateTransaction200Response {
    /// The round immediately preceding this simulation. State changes through this round were used to run this simulation.
    #[serde(rename = "last-round")]
    pub last_round: u64,
    /// A result object for each transaction group that was simulated.
    #[serde(rename = "txn-groups")]
    pub txn_groups: Vec<crate::models::SimulateTransactionGroupResult>,
    /// The version of this response object.
    #[serde(rename = "version")]
    pub version: u64,
    /// Indicates whether the simulated transactions would have succeeded during an actual submission. If any transaction fails or is missing a signature, this will be false.
    #[serde(rename = "would-succeed")]
    pub would_succeed: bool,
    /// Echoes any per-eval overrides the simulator applied (max-log-calls,
    /// max-log-size, extra-opcode-budget, allow-empty-signatures,
    /// allow-unnamed-resources, fix-signers).
    #[serde(rename = "eval-overrides", skip_serializing_if = "Option::is_none")]
    pub eval_overrides: Option<Box<crate::models::SimulateEvalOverrides>>,
    /// Echoes the exec-trace config the simulator applied. Helpful for
    /// round-tripping a request body.
    #[serde(rename = "exec-trace-config", skip_serializing_if = "Option::is_none")]
    pub exec_trace_config: Option<Box<crate::models::SimulateTraceConfig>>,
    /// The protocol version under which the simulation ran (e.g.
    /// `https://github.com/algorandfoundation/specs/tree/<hash>`).
    #[serde(rename = "initial-states", skip_serializing_if = "Option::is_none")]
    pub initial_states: Option<serde_json::Value>,
}

impl SimulateTransaction200Response {
    pub fn new(
        last_round: u64,
        txn_groups: Vec<crate::models::SimulateTransactionGroupResult>,
        version: u64,
        would_succeed: bool,
    ) -> SimulateTransaction200Response {
        SimulateTransaction200Response {
            last_round,
            txn_groups,
            version,
            would_succeed,
            eval_overrides: None,
            exec_trace_config: None,
            initial_states: None,
        }
    }
}
