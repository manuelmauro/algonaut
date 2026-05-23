/*
 * Algod REST API.
 *
 * Hand-written model added in support of ADR
 * `simulaterequest-model-needs-power-pack-fields`. Matches the
 * `SimulateTraceConfig` definition in algod's `algod.oas3.json`.
 */

use serde::{Deserialize, Serialize};

/// SimulateTraceConfig : An object that configures simulation execution
/// trace.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SimulateTraceConfig {
    /// Master switch — when `Some(true)` the response includes execution
    /// traces. The individual `*_change` toggles are no-ops unless this
    /// is enabled.
    #[serde(rename = "enable", skip_serializing_if = "Option::is_none")]
    pub enable: Option<bool>,

    /// Emit stack changes (push / pop counts plus pushed values) in
    /// each opcode trace unit.
    #[serde(rename = "stack-change", skip_serializing_if = "Option::is_none")]
    pub stack_change: Option<bool>,

    /// Emit scratch-slot writes alongside stack changes.
    #[serde(rename = "scratch-change", skip_serializing_if = "Option::is_none")]
    pub scratch_change: Option<bool>,

    /// Emit application-state changes (global / local / box writes and
    /// deletes).
    #[serde(rename = "state-change", skip_serializing_if = "Option::is_none")]
    pub state_change: Option<bool>,
}

impl SimulateTraceConfig {
    pub fn new() -> SimulateTraceConfig {
        SimulateTraceConfig::default()
    }
}
