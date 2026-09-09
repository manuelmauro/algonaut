//! ARC-16 — the `traits` object used for rarity computation.
//!
//! Traits live inside the metadata `properties` object as a `traits` key. Each
//! entry is a name → value pair whose value is a string or a number. If the NFT
//! belongs to a collection, *all* collection traits MUST be listed; a trait the
//! NFT lacks MUST be the string [`NONE`].

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The sentinel value for a trait the NFT does not have.
pub const NONE: &str = "none";

/// A trait value: per ARC-16, a string or a number.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TraitValue {
    /// A numeric trait value (e.g. `tattoos: 4`).
    Number(f64),
    /// A string trait value (e.g. `background: "red"`, or [`NONE`]).
    Text(String),
}

impl TraitValue {
    /// True if this value is the ARC-16 [`NONE`] sentinel.
    pub fn is_none(&self) -> bool {
        matches!(self, TraitValue::Text(s) if s == NONE)
    }
}

/// The ARC-16 `traits` map (insertion-independent, sorted by key).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Traits(pub BTreeMap<String, TraitValue>);

impl Traits {
    /// Look up a trait by name.
    pub fn get(&self, name: &str) -> Option<&TraitValue> {
        self.0.get(name)
    }

    /// The number of traits the NFT actually has (excluding [`NONE`] entries).
    pub fn present_count(&self) -> usize {
        self.0.values().filter(|v| !v.is_none()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_mixed_string_and_number() {
        let json = br#"{"background":"red","tattoos":4,"glasses":"none"}"#;
        let t: Traits = serde_json::from_slice(json).unwrap();
        assert_eq!(t.get("background"), Some(&TraitValue::Text("red".into())));
        assert_eq!(t.get("tattoos"), Some(&TraitValue::Number(4.0)));
        assert!(t.get("glasses").unwrap().is_none());
        assert_eq!(t.present_count(), 2);
    }
}
