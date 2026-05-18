/*
 * Algod REST API.
 *
 * Hand-written model — see ADR
 * `simulaterequest-model-needs-power-pack-fields`. Matches
 * `ScratchChange` in algod's `algod.oas3.json`.
 */

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ScratchChange {
    /// The scratch-slot index that was written.
    #[serde(rename = "slot")]
    pub slot: u64,
    /// The new value of the slot after the write.
    #[serde(rename = "new-value")]
    pub new_value: Box<crate::models::AvmValue>,
}

impl ScratchChange {
    pub fn new(slot: u64, new_value: crate::models::AvmValue) -> ScratchChange {
        ScratchChange {
            slot,
            new_value: Box::new(new_value),
        }
    }
}
