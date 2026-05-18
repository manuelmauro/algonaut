/*
 * Algod REST API.
 *
 * Hand-written model — see ADR
 * `simulaterequest-model-needs-power-pack-fields`. Matches
 * `SimulateEvalOverrides` in algod's `algod.oas3.json`. The response
 * echoes the request's power-pack toggles so the caller can verify
 * which overrides actually took effect.
 */

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SimulateEvalOverrides {
    #[serde(
        rename = "allow-empty-signatures",
        skip_serializing_if = "Option::is_none"
    )]
    pub allow_empty_signatures: Option<bool>,
    #[serde(
        rename = "allow-unnamed-resources",
        skip_serializing_if = "Option::is_none"
    )]
    pub allow_unnamed_resources: Option<bool>,
    /// Total extra opcode budget that was made available.
    #[serde(
        rename = "extra-opcode-budget",
        skip_serializing_if = "Option::is_none"
    )]
    pub extra_opcode_budget: Option<u64>,
    /// The maximum number of log calls per transaction that the
    /// simulation honoured.
    #[serde(rename = "max-log-calls", skip_serializing_if = "Option::is_none")]
    pub max_log_calls: Option<u64>,
    /// The maximum total log payload size per transaction.
    #[serde(rename = "max-log-size", skip_serializing_if = "Option::is_none")]
    pub max_log_size: Option<u64>,
    /// Whether the simulator replayed transactions with their rekey
    /// auth-address as the signer.
    #[serde(rename = "fix-signers", skip_serializing_if = "Option::is_none")]
    pub fix_signers: Option<bool>,
}

impl SimulateEvalOverrides {
    pub fn new() -> SimulateEvalOverrides {
        SimulateEvalOverrides::default()
    }
}
