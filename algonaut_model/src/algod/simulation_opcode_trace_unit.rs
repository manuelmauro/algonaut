/*
 * Algod REST API.
 *
 * Hand-written model — see ADR
 * `simulaterequest-model-needs-power-pack-fields`. Matches
 * `SimulationOpcodeTraceUnit` in algod's `algod.oas3.json`.
 */

use serde::{Deserialize, Serialize};

/// SimulationOpcodeTraceUnit : One step of execution, captured when the
/// caller asked for an exec-trace.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SimulationOpcodeTraceUnit {
    /// Program counter of the opcode.
    #[serde(rename = "pc")]
    pub pc: u64,
    /// Indexes into the surrounding `inner-trace` slice for each inner
    /// transaction spawned at this PC.
    #[serde(rename = "spawned-inners", skip_serializing_if = "Option::is_none")]
    pub spawned_inners: Option<Vec<u64>>,
    /// Values pushed onto the stack. Only populated when
    /// `exec-trace-config.stack-change` is on.
    #[serde(rename = "stack-additions", skip_serializing_if = "Option::is_none")]
    pub stack_additions: Option<Vec<crate::algod::AvmValue>>,
    /// Number of values popped from the stack before the additions were
    /// pushed.
    #[serde(rename = "stack-pop-count", skip_serializing_if = "Option::is_none")]
    pub stack_pop_count: Option<u64>,
    /// Scratch-slot writes performed by this opcode. Only populated when
    /// `exec-trace-config.scratch-change` is on.
    #[serde(rename = "scratch-changes", skip_serializing_if = "Option::is_none")]
    pub scratch_changes: Option<Vec<crate::algod::ScratchChange>>,
    /// Application-state writes / deletes performed by this opcode.
    /// Only populated when `exec-trace-config.state-change` is on.
    #[serde(rename = "state-changes", skip_serializing_if = "Option::is_none")]
    pub state_changes: Option<Vec<crate::algod::ApplicationStateOperation>>,
}

impl SimulationOpcodeTraceUnit {
    pub fn new(pc: u64) -> SimulationOpcodeTraceUnit {
        SimulationOpcodeTraceUnit {
            pc,
            spawned_inners: None,
            stack_additions: None,
            stack_pop_count: None,
            scratch_changes: None,
            state_changes: None,
        }
    }
}
