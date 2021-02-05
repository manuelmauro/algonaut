//! This file mostly just hides away various trait implementations that would clutter up and distract from the more important code elsewhere
use crate::crypto::{Address, MultisigSignature, MultisigSubsig, Signature};
use crate::error::{AlgorandError, BuilderError};
use crate::kmd::v1::responses::ExportKeyResponse;
use crate::models::{
    Ed25519PublicKey, HashDigest, MasterDerivationKey, MicroAlgos, Round, VotePK, VRFPK,
};
use crate::transaction::{Transaction, TransactionType};
use data_encoding::BASE64;
use derive_more::Display;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use static_assertions::_core::ops::{Add, Sub};
use std::fmt::{Debug, Display, Formatter};
use std::ops::Mul;

impl Serialize for Transaction {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let type_len = match &self.txn_type {
            TransactionType::Payment(payment) => {
                1 + if payment.close_remainder_to.is_some() {
                    1
                } else {
                    0
                } + if payment.amount.0 != 0 { 1 } else { 0 }
            }
            TransactionType::KeyRegistration(_) => 5,
        };
        let len = 6
            + type_len
            + if self.note.is_empty() { 0 } else { 1 }
            + if self.genesis_id.is_empty() { 0 } else { 1 };
        let mut state = serializer.serialize_struct("Transaction", len)?;
        if let TransactionType::Payment(payment) = &self.txn_type {
            if payment.amount.0 != 0 {
                state.serialize_field("amt", &payment.amount)?;
            }
        }
        if let TransactionType::Payment(payment) = &self.txn_type {
            if payment.close_remainder_to.is_some() {
                state.serialize_field("close", &payment.close_remainder_to)?;
            }
        }
        state.serialize_field("fee", &self.fee)?;
        state.serialize_field("fv", &self.first_valid)?;
        if !self.genesis_id.is_empty() {
            state.serialize_field("gen", &self.genesis_id)?;
        }
        state.serialize_field("gh", &self.genesis_hash)?;
        state.serialize_field("lv", &self.last_valid)?;
        if !self.note.is_empty() {
            state.serialize_field("note", &serde_bytes::ByteBuf::from(self.note.clone()))?;
        }
        if let TransactionType::Payment(payment) = &self.txn_type {
            state.serialize_field("rcv", &payment.receiver)?;
        }
        if let TransactionType::KeyRegistration(key_registration) = &self.txn_type {
            state.serialize_field("selkey", &key_registration.selection_pk)?;
        }
        state.serialize_field("snd", &self.sender)?;
        match &self.txn_type {
            TransactionType::Payment(_payment) => {
                state.serialize_field("type", "pay")?;
            }
            TransactionType::KeyRegistration(_key_registration) => {
                state.serialize_field("type", "keyreg")?;
            }
        }
        if let TransactionType::KeyRegistration(key_registration) = &self.txn_type {
            state.serialize_field("votefst", &key_registration.vote_first)?;
        }
        if let TransactionType::KeyRegistration(key_registration) = &self.txn_type {
            state.serialize_field("votekd", &key_registration.vote_key_dilution)?;
        }
        if let TransactionType::KeyRegistration(key_registration) = &self.txn_type {
            state.serialize_field("votekey", &key_registration.vote_pk)?;
        }
        if let TransactionType::KeyRegistration(key_registration) = &self.txn_type {
            state.serialize_field("votelst", &key_registration.vote_last)?;
        }
        state.end()
    }
}

impl Serialize for HashDigest {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0[..])
    }
}

impl Serialize for VotePK {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0[..])
    }
}

impl Serialize for VRFPK {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0[..])
    }
}

impl Serialize for Ed25519PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0[..])
    }
}

impl<'de> Deserialize<'de> for HashDigest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(HashDigest(deserializer.deserialize_bytes(U8_32Visitor)?))
    }
}

impl<'de> Deserialize<'de> for VotePK {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(VotePK(deserializer.deserialize_bytes(U8_32Visitor)?))
    }
}

impl<'de> Deserialize<'de> for VRFPK {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(VRFPK(deserializer.deserialize_bytes(U8_32Visitor)?))
    }
}

impl<'de> Deserialize<'de> for Ed25519PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Ed25519PublicKey(
            deserializer.deserialize_bytes(U8_32Visitor)?,
        ))
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
        serializer.serialize_bytes(&self.0[..])
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Address(deserializer.deserialize_bytes(U8_32Visitor)?))
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

impl<'de> Visitor<'de> for U8_32Visitor {
    type Value = [u8; 32];

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a 32 byte array")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.len() == 32 {
            let mut bytes = [0; 32];
            bytes.copy_from_slice(v);
            Ok(bytes)
        } else {
            Err(E::custom(format!("Invalid byte array length: {}", v.len())))
        }
    }
}

pub fn deserialize_hash<'de, D>(deserializer: D) -> Result<HashDigest, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(HashDigest(deserialize_bytes32(deserializer)?))
}

pub fn deserialize_mdk<'de, D>(deserializer: D) -> Result<MasterDerivationKey, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(MasterDerivationKey(deserialize_bytes32(deserializer)?))
}

pub fn deserialize_bytes32<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
where
    D: Deserializer<'de>,
{
    let s = <&str>::deserialize(deserializer)?;
    let mut decoded = [0; 32];
    decoded.copy_from_slice(&BASE64.decode(s.as_bytes()).unwrap());
    Ok(decoded)
}

pub fn deserialize_bytes64<'de, D>(deserializer: D) -> Result<[u8; 64], D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let s = <&str>::deserialize(deserializer)?;
    let mut decoded = [0; 64];
    let bytes = BASE64.decode(s.as_bytes()).map_err(D::Error::custom)?;
    decoded.copy_from_slice(&bytes);
    Ok(decoded)
}

pub fn deserialize_bytes<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = <&str>::deserialize(deserializer)?;
    Ok(BASE64.decode(s.as_bytes()).unwrap())
}

pub fn serialize_bytes<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&BASE64.encode(bytes))
}

impl Display for MicroAlgos {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Display for Round {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Debug for ExportKeyResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExportKeyResponse")
            .field("private_key", &self.private_key.to_vec())
            .finish()
    }
}

impl Debug for Signature {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Signature").field(&self.0.to_vec()).finish()
    }
}

impl Add for Round {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Round(self.0 + rhs.0)
    }
}

impl Add<u64> for Round {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Round(self.0 + rhs)
    }
}

impl Add for MicroAlgos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        MicroAlgos(self.0 + rhs.0)
    }
}

impl Add<u64> for MicroAlgos {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        MicroAlgos(self.0 + rhs)
    }
}

impl Sub for Round {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Round(self.0 - rhs.0)
    }
}

impl Sub<u64> for Round {
    type Output = Self;

    fn sub(self, rhs: u64) -> Self::Output {
        Round(self.0 - rhs)
    }
}

impl Sub for MicroAlgos {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        MicroAlgos(self.0 - rhs.0)
    }
}

impl Sub<u64> for MicroAlgos {
    type Output = Self;

    fn sub(self, rhs: u64) -> Self::Output {
        MicroAlgos(self.0 - rhs)
    }
}

// Intentionally not implementing Mul<Rhs=Self>
// If you're multiplying a Round by a Round or MicroAlgos by MicroAlgos, something has gone wrong in your math
// That would give you MicroAlgos squared or Rounds squared and those don't exist
impl Mul<u64> for Round {
    type Output = Self;

    fn mul(self, rhs: u64) -> Self::Output {
        Round(self.0 * rhs)
    }
}

impl Mul<u64> for MicroAlgos {
    type Output = Self;

    fn mul(self, rhs: u64) -> Self::Output {
        MicroAlgos(self.0 * rhs)
    }
}

impl PartialEq for Signature {
    fn eq(&self, other: &Self) -> bool {
        for i in 0..64 {
            if self.0[i] != other.0[i] {
                return false;
            }
        }
        true
    }
}

impl Eq for Signature {}

/// An API token.
#[derive(Display)]
#[display(fmt = "{}", token)]
pub struct ApiToken {
    token: String,
}

const TOKEN_LENGTH: usize = 64;

impl ApiToken {
    /// Parses a string slice representing an API token.
    pub fn parse(token: &str) -> Result<Self, AlgorandError> {
        if token.len() != TOKEN_LENGTH {
            return Err(BuilderError::BadToken.into());
        }

        Ok(ApiToken {
            token: token.to_string(),
        })
    }
}
