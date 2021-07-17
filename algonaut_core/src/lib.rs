use algonaut_crypto::Ed25519PublicKey;
use algonaut_crypto::HashDigest;
use algonaut_crypto::Signature;
use algonaut_encoding::U8_32Visitor;
use data_encoding::BASE64;
use derive_more::{Add, Display, Sub};
use error::CoreError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use static_assertions::_core::ops::{Add, Sub};
use std::convert::TryInto;
use std::fmt::{self, Debug, Formatter};
use std::ops::Mul;

pub use address::Address;
pub use address::MultisigAddress;

mod address;
mod error;

pub const MICRO_ALGO_CONVERSION_FACTOR: f64 = 1e6;

/// MicroAlgos are the base unit of currency in Algorand
#[derive(
    Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize, Display, Add, Sub,
)]
pub struct MicroAlgos(pub u64);

impl MicroAlgos {
    pub fn to_algos(self) -> f64 {
        self.0 as f64 / MICRO_ALGO_CONVERSION_FACTOR
    }

    pub fn from_algos(algos: f64) -> MicroAlgos {
        MicroAlgos((algos * MICRO_ALGO_CONVERSION_FACTOR) as u64)
    }
}

impl Add<u64> for MicroAlgos {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        MicroAlgos(self.0 + rhs)
    }
}

impl Sub<u64> for MicroAlgos {
    type Output = Self;

    fn sub(self, rhs: u64) -> Self::Output {
        MicroAlgos(self.0 - rhs)
    }
}

// Intentionally not implementing Mul<Rhs=Self>
// If you're multiplying a MicroAlgos by MicroAlgos, something has gone wrong in your math
// That would give you MicroAlgos squared and those don't exist
impl Mul<u64> for MicroAlgos {
    type Output = Self;

    fn mul(self, rhs: u64) -> Self::Output {
        MicroAlgos(self.0 * rhs)
    }
}

/// Round of the Algorand consensus protocol
#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize, Display, Add, Sub)]
pub struct Round(pub u64);

impl Add<u64> for Round {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Round(self.0 + rhs)
    }
}

impl Sub<u64> for Round {
    type Output = Self;

    fn sub(self, rhs: u64) -> Self::Output {
        Round(self.0 - rhs)
    }
}

// Intentionally not implementing Mul<Rhs=Self>
// If you're multiplying a Round by a Round, something has gone wrong in your math
// That would give you Rounds squared and those don't exist
impl Mul<u64> for Round {
    type Output = Self;

    fn mul(self, rhs: u64) -> Self::Output {
        Round(self.0 * rhs)
    }
}

/// Participation public key used in key registration transactions
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct VotePk(pub [u8; 32]);

impl Serialize for VotePk {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0[..])
    }
}

impl<'de> Deserialize<'de> for VotePk {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(VotePk(deserializer.deserialize_bytes(U8_32Visitor)?))
    }
}

impl Debug for VotePk {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_base64_str())
    }
}

impl VotePk {
    pub fn from_base64_str(base64_str: &str) -> Result<VotePk, CoreError> {
        Ok(VotePk(base64_str_to_u8_array(base64_str)?))
    }

    pub fn to_base64_str(self) -> String {
        BASE64.encode(&self.0)
    }
}

/// VRF public key used in key registration transaction
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct VrfPk(pub [u8; 32]);

impl Serialize for VrfPk {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0[..])
    }
}

impl<'de> Deserialize<'de> for VrfPk {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(VrfPk(deserializer.deserialize_bytes(U8_32Visitor)?))
    }
}

impl Debug for VrfPk {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_base64_str())
    }
}

impl VrfPk {
    pub fn from_base64_str(base64_str: &str) -> Result<VrfPk, CoreError> {
        Ok(VrfPk(base64_str_to_u8_array(base64_str)?))
    }

    pub fn to_base64_str(self) -> String {
        BASE64.encode(&self.0)
    }
}

#[derive(Default, Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct MultisigSignature {
    #[serde(rename = "subsig")]
    pub subsigs: Vec<MultisigSubsig>,

    #[serde(rename = "thr")]
    pub threshold: u8,

    #[serde(rename = "v")]
    pub version: u8,
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

#[derive(Eq, PartialEq, Clone)]
pub struct SignedLogic {
    pub logic: CompiledTeal,
    pub args: Vec<Vec<u8>>,
    pub sig: LogicSignature,
}

impl Debug for SignedLogic {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "logic: {:?}, args: {:?}, sig: {:?}",
            BASE64.encode(&self.logic.0),
            self.args
                .iter()
                .map(|a| BASE64.encode(a))
                .collect::<Vec<String>>(),
            self.sig
        )
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct CompiledTeal(pub Vec<u8>);

impl CompiledTeal {
    pub fn bytes_to_sign(&self) -> Vec<u8> {
        let mut prefix_encoded_tx = b"Program".to_vec();
        prefix_encoded_tx.extend_from_slice(&self.0);
        prefix_encoded_tx
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct CompiledTealWithHash {
    /// base32 SHA512_256 of program bytes (Address style)
    pub hash: String,
    pub program: CompiledTeal,
}

// TODO rename
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum LogicSignature {
    ContractAccount,
    DelegatedSig(Signature),
    DelegatedMultiSig(MultisigSignature),
}

pub trait ToMsgPack: Serialize {
    fn to_msg_pack(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec_named(&self)
    }
}

fn base64_str_to_u8_array<const N: usize>(base64_str: &str) -> Result<[u8; N], CoreError> {
    BASE64
        .decode(base64_str.as_bytes())?
        .try_into()
        .map_err(|v| CoreError::General(format!("Couldn't convert vec: {:?} into u8 array", v)))
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SuggestedTransactionParams {
    pub genesis_id: String,
    pub genesis_hash: HashDigest,
    pub consensus_version: String,
    pub fee: MicroAlgos,
    pub min_fee: MicroAlgos,
    pub first_valid: Round,
    pub last_valid: Round,
}
