use serde::{Serialize, Serializer, Deserializer, Deserialize};
use crate::{HashDigest, VotePK, VRFPK, Ed25519PublicKey, MasterDerivationKey};
use serde::de::Visitor;
use data_encoding::BASE64;
use crate::crypto::{MultisigSignature, MultisigSubsig, Address, Signature};

impl Serialize for HashDigest {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
        S: Serializer {
        serializer.serialize_bytes(&self.0[..])
    }
}

impl Serialize for VotePK {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
        S: Serializer {
        serializer.serialize_bytes(&self.0[..])
    }
}

impl Serialize for VRFPK {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
        S: Serializer {
        serializer.serialize_bytes(&self.0[..])
    }
}

impl Serialize for Ed25519PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
        S: Serializer {
        serializer.serialize_bytes(&self.0[..])
    }
}

impl<'de> Deserialize<'de> for HashDigest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where
        D: Deserializer<'de> {
        Ok(HashDigest(deserializer.deserialize_bytes(U8_32Visitor)?))
    }
}

impl<'de> Deserialize<'de> for VotePK {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where
        D: Deserializer<'de> {
        Ok(VotePK(deserializer.deserialize_bytes(U8_32Visitor)?))
    }
}

impl<'de> Deserialize<'de> for VRFPK {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where
        D: Deserializer<'de> {
        Ok(VRFPK(deserializer.deserialize_bytes(U8_32Visitor)?))
    }
}

impl<'de> Deserialize<'de> for Ed25519PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where
        D: Deserializer<'de> {
        Ok(Ed25519PublicKey(deserializer.deserialize_bytes(U8_32Visitor)?))
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

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
        where
            S: Serializer,
    {
        serializer.serialize_bytes(&self.bytes[..])
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        Ok(Address {
            bytes: deserializer.deserialize_bytes(U8_32Visitor)?,
        })
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
        where
            S: Serializer,
    {
        serializer.serialize_bytes(&self.0[..])
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        Ok(Signature(deserializer.deserialize_bytes(SignatureVisitor)?))
    }
}

struct SignatureVisitor;
impl<'de> Visitor<'de> for SignatureVisitor {
    type Value = [u8; 64];

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a 64 byte array")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        if v.len() == 64 {
            let mut bytes = [0; 64];
            bytes.copy_from_slice(v);
            Ok(bytes)
        } else {
            Err(E::custom(format!("Invalid signature length: {}", v.len())))
        }
    }
}

pub(crate) struct U8_32Visitor;

impl<'de> Visitor<'de> for U8_32Visitor
    where
{
    type Value = [u8; 32];

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a 32 byte array")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E> where
        E: serde::de::Error, {
        if v.len() == 32 {
            let mut bytes = [0; 32];
            bytes.copy_from_slice(v);
            Ok(bytes)
        } else {
            Err(E::custom(format!("Invalid byte array length: {}", v.len())))
        }
    }
}

pub fn deserialize_hash<'de, D>(deserializer: D) -> Result<HashDigest, D::Error> where
    D: Deserializer<'de> {
    Ok(HashDigest(deserialize_bytes32(deserializer)?))
}

pub fn deserialize_mdk<'de, D>(deserializer: D) -> Result<MasterDerivationKey, D::Error> where
    D: Deserializer<'de> {
    Ok(MasterDerivationKey(deserialize_bytes32(deserializer)?))
}

pub fn deserialize_bytes32<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error> where
    D: Deserializer<'de> {
    let s = <&str>::deserialize(deserializer)?;
    let mut decoded = [0; 32];
    decoded.copy_from_slice(&BASE64.decode(s.as_bytes()).unwrap());
    Ok(decoded)
}

pub fn deserialize_bytes64<'de, D>(deserializer: D) -> Result<[u8; 64], D::Error> where
    D: Deserializer<'de> {
    use serde::de::Error;
    let s = <&str>::deserialize(deserializer)?;
    let mut decoded = [0; 64];
    let bytes = BASE64.decode(s.as_bytes()).map_err(D::Error::custom)?;
    decoded.copy_from_slice(&bytes);
    Ok(decoded)
}

pub fn deserialize_bytes<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error> where
    D: Deserializer<'de> {
    let s = <&str>::deserialize(deserializer)?;
    Ok(BASE64.decode(s.as_bytes()).unwrap())
}

pub fn serialize_bytes<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
    serializer.serialize_str(&BASE64.encode(bytes))
}
