use algonaut_crypto::Ed25519PublicKey;
use algonaut_crypto::Signature;
use serde::{Deserialize, Serialize, Serializer};
use std::fmt::Debug;

pub const MULTISIG_VERSION: u8 = 1;

#[derive(Default, Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct MultisigSignature {
    #[serde(rename = "subsig")]
    pub subsigs: Vec<MultisigSubsig>,

    #[serde(rename = "thr")]
    pub threshold: u8,

    #[serde(rename = "v")]
    pub version: u8,
}

impl MultisigSignature {
    pub fn verify(&self, message: &[u8]) -> bool {
        if self.version != MULTISIG_VERSION || self.threshold == 0 || self.subsigs.is_empty() {
            return false;
        }
        if self.threshold as usize > self.subsigs.len() {
            return false;
        }
        self.verify_subsigs(message)
    }

    /// Checks threshold subsigs are signed and that the signatures are valid.
    fn verify_subsigs(&self, message: &[u8]) -> bool {
        self.subsigs
            .iter()
            .filter(|subsig| {
                subsig
                    .sig
                    .map(|sig| subsig.key.verify(message, &sig))
                    .unwrap_or(false) // not signed yet
            })
            .count()
            == self.threshold as usize
    }
}

impl Serialize for MultisigSignature {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        // For some reason SerializeStruct ends up serializing as an array, so this explicitly serializes as a map
        use serde::ser::SerializeMap;
        let mut state = serializer.serialize_map(Some(3))?;
        state.serialize_entry("subsig", &self.subsigs)?;
        state.serialize_entry("thr", &self.threshold)?;
        state.serialize_entry("v", &self.version)?;
        state.end()
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct MultisigSubsig {
    #[serde(rename = "pk")]
    pub key: Ed25519PublicKey,

    #[serde(rename = "s")]
    pub sig: Option<Signature>,
}

impl Serialize for MultisigSubsig {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;
        let len = if self.sig.is_some() { 2 } else { 1 };
        let mut state = serializer.serialize_map(Some(len))?;
        state.serialize_entry("pk", &self.key)?;
        if let Some(sig) = &self.sig {
            state.serialize_entry("s", sig)?;
        }
        state.end()
    }
}
