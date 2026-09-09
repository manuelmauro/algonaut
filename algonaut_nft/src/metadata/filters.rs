//! ARC-36 — the `filters` object used to filter a collection without rarity.
//!
//! Structurally parallel to ARC-16 [`traits`](super::traits): a name → value map
//! living inside the metadata `properties` object under a `filters` key, with the
//! same [`NONE`](super::traits::NONE) sentinel for an absent filter. ARC-36's
//! schema describes values as arrays of strings or numbers, while its example
//! uses scalars, so [`FilterValue`] accepts either.

use super::traits::TraitValue;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A filter value: a single string/number, or an array of them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilterValue {
    /// An array of values (the schema's stated form).
    Many(Vec<TraitValue>),
    /// A single scalar value (the spec's example form).
    One(TraitValue),
}

/// The ARC-36 `filters` map.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Filters(pub BTreeMap<String, FilterValue>);

impl Filters {
    /// Look up a filter by name.
    pub fn get(&self, name: &str) -> Option<&FilterValue> {
        self.0.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_scalar_and_array_values() {
        let json = br#"{"xp":120,"state":"REM","tags":["a","b"]}"#;
        let f: Filters = serde_json::from_slice(json).unwrap();
        assert_eq!(
            f.get("xp"),
            Some(&FilterValue::One(TraitValue::Number(120.0)))
        );
        assert_eq!(
            f.get("state"),
            Some(&FilterValue::One(TraitValue::Text("REM".into())))
        );
        assert!(matches!(f.get("tags"), Some(FilterValue::Many(v)) if v.len() == 2));
    }
}
