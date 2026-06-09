//! OnComplete actions, shared by a contract's bare actions and a method's
//! per-call actions.

use serde::{Deserialize, Serialize};

/// A set of OnComplete actions, split into those valid on create vs. on call.
///
/// Action strings are the AVM OnComplete names (`"NoOp"`, `"OptIn"`,
/// `"CloseOut"`, `"UpdateApplication"`, `"DeleteApplication"`); kept as strings
/// so the model stays forward-compatible and dependency-free.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Actions {
    /// Actions valid when creating the app.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub create: Vec<String>,

    /// Actions valid when calling an existing app.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub call: Vec<String>,
}
