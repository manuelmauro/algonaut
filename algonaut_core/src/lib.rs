use algonaut_crypto::HashDigest;
use algonaut_crypto::Signature;
use algonaut_encoding::{U8_32Visitor, U8_64Visitor};
use data_encoding::BASE64;
use derive_more::{Add, Display, Sub};
use error::CoreError;
pub use multisig::MultisigSignature;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::Digest;
use static_assertions::_core::ops::{Add, Sub};
use std::convert::TryInto;
use std::fmt::{self, Debug, Formatter};
use std::ops::Mul;

pub use address::Address;
pub use address::MultisigAddress;
pub use multisig::MultisigSubsig;

mod address;
pub mod error;
mod multisig;

pub const MICRO_ALGO_CONVERSION_FACTOR: f64 = 1e6;

/// MicroAlgos are the base unit of currency in Algorand
#[derive(
    Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize, Display, Add, Sub,
)]
pub struct MicroAlgos(pub u64);

impl MicroAlgos {
    pub fn from_algos(algos: u64) -> Self {
        MicroAlgos(algos * 1_000_000)
    }

    pub fn from_millialgos(millialgos: u64) -> Self {
        MicroAlgos(millialgos * 1_000)
    }

    /// Checked addition of two amounts. Returns `None` if the result would
    /// overflow `u64`.
    pub fn checked_add(self, rhs: MicroAlgos) -> Option<MicroAlgos> {
        self.0.checked_add(rhs.0).map(MicroAlgos)
    }

    /// Checked subtraction of two amounts. Returns `None` if `rhs` is larger
    /// than `self`.
    pub fn checked_sub(self, rhs: MicroAlgos) -> Option<MicroAlgos> {
        self.0.checked_sub(rhs.0).map(MicroAlgos)
    }

    /// Checked multiplication by a scalar. Returns `None` if the result would
    /// overflow `u64`.
    pub fn checked_mul(self, rhs: u64) -> Option<MicroAlgos> {
        self.0.checked_mul(rhs).map(MicroAlgos)
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

impl From<u64> for Round {
    fn from(u: u64) -> Self {
        Self(u)
    }
}

/// Identifier of an Algorand application (smart contract).
///
/// A tuple struct over `u64`; unlike a bare integer it cannot be mixed up
/// with an [`AssetId`] or any other numeric value. Serializes transparently
/// as its inner `u64`, so the wire format matches a plain integer.
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize, Display,
)]
pub struct AppId(pub u64);

impl AppId {
    /// The address of this application's escrow account.
    ///
    /// Computed as `sha512_256("appID" || u64_be_bytes(self.0))`, packaged
    /// in a 32-byte [`Address`].
    pub fn address(self) -> Address {
        let bytes = self.0.to_be_bytes();
        let all_bytes = ["appID".as_bytes(), &bytes].concat();
        let hash = sha2::Sha512_256::digest(all_bytes);
        Address(hash.into())
    }
}

impl From<u64> for AppId {
    fn from(id: u64) -> Self {
        AppId(id)
    }
}

impl From<AppId> for u64 {
    fn from(id: AppId) -> Self {
        id.0
    }
}

/// Identifier of an Algorand Standard Asset.
///
/// A tuple struct over `u64`; unlike a bare integer it cannot be mixed up
/// with an [`AppId`] or any other numeric value. Serializes transparently
/// as its inner `u64`, so the wire format matches a plain integer.
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize, Display,
)]
pub struct AssetId(pub u64);

impl From<u64> for AssetId {
    fn from(id: u64) -> Self {
        AssetId(id)
    }
}

impl From<AssetId> for u64 {
    fn from(id: AssetId) -> Self {
        id.0
    }
}

/// Identifier of an Algorand transaction: the base32-encoded SHA-512/256
/// hash of the transaction.
///
/// A tuple struct over `String`. Serializes transparently as its inner
/// `String`.
#[derive(
    Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize, Display,
)]
pub struct TransactionId(pub String);

impl TransactionId {
    /// Borrows the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for TransactionId {
    fn from(id: String) -> Self {
        TransactionId(id)
    }
}

impl From<&str> for TransactionId {
    fn from(id: &str) -> Self {
        TransactionId(id.to_owned())
    }
}

impl From<TransactionId> for String {
    fn from(id: TransactionId) -> Self {
        id.0
    }
}

impl AsRef<str> for TransactionId {
    fn as_ref(&self) -> &str {
        &self.0
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
        // JSON: base64; msgpack: raw bytes. See ADR
        // domain-types-serialize-for-both-json-and-msgpack.
        if serializer.is_human_readable() {
            serializer.serialize_str(&BASE64.encode(&self.0))
        } else {
            serializer.serialize_bytes(&self.0[..])
        }
    }
}

impl<'de> Deserialize<'de> for VotePk {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            VotePk::from_base64_str(&s).map_err(serde::de::Error::custom)
        } else {
            Ok(VotePk(deserializer.deserialize_bytes(U8_32Visitor)?))
        }
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
        if serializer.is_human_readable() {
            serializer.serialize_str(&BASE64.encode(&self.0))
        } else {
            serializer.serialize_bytes(&self.0[..])
        }
    }
}

impl<'de> Deserialize<'de> for VrfPk {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            VrfPk::from_base64_str(&s).map_err(serde::de::Error::custom)
        } else {
            Ok(VrfPk(deserializer.deserialize_bytes(U8_32Visitor)?))
        }
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

/// State-proof (BLS) public key used in v2 online key registration
/// transactions. Serialized as the 64-byte `sprfkey` field in msgpack
/// and as base64 in JSON.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct StateProofPk(pub [u8; 64]);

impl Serialize for StateProofPk {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&BASE64.encode(&self.0))
        } else {
            serializer.serialize_bytes(&self.0[..])
        }
    }
}

impl<'de> Deserialize<'de> for StateProofPk {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            StateProofPk::from_base64_str(&s).map_err(serde::de::Error::custom)
        } else {
            Ok(StateProofPk(deserializer.deserialize_bytes(U8_64Visitor)?))
        }
    }
}

impl Debug for StateProofPk {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_base64_str())
    }
}

impl StateProofPk {
    pub fn from_base64_str(base64_str: &str) -> Result<StateProofPk, CoreError> {
        let bytes = BASE64.decode(base64_str.as_bytes())?;
        let arr: [u8; 64] = bytes
            .try_into()
            .map_err(|v: Vec<u8>| CoreError::InvalidArraySize {
                expected: 64,
                actual: v.len(),
            })?;
        Ok(StateProofPk(arr))
    }

    pub fn to_base64_str(self) -> String {
        BASE64.encode(&self.0)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct CompiledTeal(pub Vec<u8>);

impl CompiledTeal {
    pub fn bytes_to_sign(&self) -> Vec<u8> {
        let mut prefix_encoded_tx = b"Program".to_vec();
        prefix_encoded_tx.extend_from_slice(&self.0);
        prefix_encoded_tx
    }

    pub fn hash(&self) -> HashDigest {
        HashDigest(sha2::Sha512_256::digest(self.bytes_to_sign()).into())
    }
}

impl From<HashDigest> for Address {
    fn from(digest: HashDigest) -> Self {
        Address(digest.0)
    }
}

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
        .map_err(|v: Vec<u8>| CoreError::InvalidArraySize {
            expected: N,
            actual: v.len(),
        })
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SuggestedTransactionParams {
    pub genesis_id: String,
    pub genesis_hash: HashDigest,
    pub consensus_version: String,
    pub fee_per_byte: MicroAlgos,
    pub min_fee: MicroAlgos,
    pub first_valid: Round,
    pub last_valid: Round,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TransactionTypeEnum {
    Payment,
    KeyRegistration,
    AssetConfiguration,
    AssetTransfer,
    AssetFreeze,
    ApplicationCall,
    StateProof,
}

impl TransactionTypeEnum {
    pub fn to_api_str(&self) -> &str {
        match self {
            TransactionTypeEnum::Payment => "pay",
            TransactionTypeEnum::KeyRegistration => "keyreg",
            TransactionTypeEnum::AssetConfiguration => "acfg",
            TransactionTypeEnum::AssetTransfer => "axfer",
            TransactionTypeEnum::AssetFreeze => "afrz",
            TransactionTypeEnum::ApplicationCall => "appl",
            TransactionTypeEnum::StateProof => "stpf",
        }
    }

    pub fn from_api_str(s: &str) -> Result<Self, CoreError> {
        match s {
            "pay" => Ok(TransactionTypeEnum::Payment),
            "keyreg" => Ok(TransactionTypeEnum::KeyRegistration),
            "acfg" => Ok(TransactionTypeEnum::AssetConfiguration),
            "axfer" => Ok(TransactionTypeEnum::AssetTransfer),
            "afrz" => Ok(TransactionTypeEnum::AssetFreeze),
            "appl" => Ok(TransactionTypeEnum::ApplicationCall),
            "stpf" => Ok(TransactionTypeEnum::StateProof),
            _ => Err(CoreError::InvalidTransactionType(s.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_program_address_correctly() {
        let program = CompiledTeal(vec![
            4, 54, 26, 0, 128, 3, 1, 0, 255, 18, 54, 26, 1, 23, 129, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 1, 18, 16, 54, 26, 2, 128, 32, 98, 162, 25, 173, 185, 140, 183, 76, 228,
            114, 235, 172, 245, 191, 248, 121, 232, 54, 170, 229, 161, 91, 215, 180, 73, 219, 245,
            120, 155, 252, 59, 92, 18, 16,
        ]);

        let digest = program.hash();

        assert_eq!(
            HashDigest([
                45, 117, 175, 55, 21, 23, 57, 110, 158, 143, 60, 222, 234, 143, 168, 69, 75, 239,
                131, 112, 96, 73, 79, 174, 120, 245, 181, 40, 236, 158, 233, 234,
            ]),
            digest
        );

        // Note that this address is also the "address style hash" string we get from the API in ApiCompiledTeal
        assert_eq!(
            "FV226NYVC44W5HUPHTPOVD5IIVF67A3QMBEU7LTY6W2SR3E65HVOQ7JV44",
            Address::new(digest.0).to_string()
        );
    }

    #[test]
    fn votepk_json_round_trip_is_base64() {
        let pk = VotePk([5; 32]);
        let json = serde_json::to_string(&pk).unwrap();
        assert_eq!(json, format!("\"{}\"", pk.to_base64_str()));
        let parsed: VotePk = serde_json::from_str(&json).unwrap();
        assert_eq!(pk, parsed);
    }

    #[test]
    fn votepk_msgpack_is_raw_bytes() {
        let pk = VotePk([5; 32]);
        let bytes = rmp_serde::to_vec(&pk).unwrap();
        // 0xc4 = bin8 header, 0x20 = 32 bytes
        assert_eq!(bytes[0], 0xc4);
        assert_eq!(bytes[1], 0x20);
        assert_eq!(&bytes[2..], &[5u8; 32]);
        let parsed: VotePk = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(pk, parsed);
    }

    #[test]
    fn state_proof_pk_json_round_trip_is_base64() {
        let pk = StateProofPk([9; 64]);
        let json = serde_json::to_string(&pk).unwrap();
        assert_eq!(json, format!("\"{}\"", pk.to_base64_str()));
        let parsed: StateProofPk = serde_json::from_str(&json).unwrap();
        assert_eq!(pk, parsed);
    }

    #[test]
    fn state_proof_pk_msgpack_is_raw_bytes() {
        let pk = StateProofPk([9; 64]);
        let bytes = rmp_serde::to_vec(&pk).unwrap();
        assert_eq!(bytes[0], 0xc4); // bin8
        assert_eq!(bytes[1], 0x40); // 64 bytes
        assert_eq!(&bytes[2..], &[9u8; 64]);
        let parsed: StateProofPk = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(pk, parsed);
    }

    #[test]
    fn micro_algos_checked_add() {
        assert_eq!(
            MicroAlgos(10).checked_add(MicroAlgos(5)),
            Some(MicroAlgos(15))
        );
        assert_eq!(MicroAlgos(u64::MAX).checked_add(MicroAlgos(1)), None);
    }

    #[test]
    fn micro_algos_checked_sub() {
        assert_eq!(
            MicroAlgos(10).checked_sub(MicroAlgos(4)),
            Some(MicroAlgos(6))
        );
        assert_eq!(MicroAlgos(0).checked_sub(MicroAlgos(1)), None);
    }

    #[test]
    fn micro_algos_checked_mul() {
        assert_eq!(MicroAlgos(7).checked_mul(3), Some(MicroAlgos(21)));
        assert_eq!(MicroAlgos(u64::MAX).checked_mul(2), None);
    }

    #[test]
    fn id_newtypes_serialize_transparently() {
        // AppId / AssetId must be wire-identical to a bare u64 in both
        // JSON and msgpack, otherwise transaction signing breaks.
        assert_eq!(serde_json::to_string(&AppId(42)).unwrap(), "42");
        assert_eq!(serde_json::to_string(&AssetId(7)).unwrap(), "7");
        assert_eq!(
            rmp_serde::to_vec(&AppId(42)).unwrap(),
            rmp_serde::to_vec(&42u64).unwrap()
        );
        assert_eq!(
            rmp_serde::to_vec(&AssetId(7)).unwrap(),
            rmp_serde::to_vec(&7u64).unwrap()
        );
        assert_eq!(
            serde_json::to_string(&TransactionId("ABC".to_owned())).unwrap(),
            "\"ABC\""
        );
    }

    #[test]
    fn id_newtypes_round_trip_and_convert() {
        let app: AppId = 99u64.into();
        assert_eq!(u64::from(app), 99);
        let asset: AssetId = 99u64.into();
        assert_eq!(u64::from(asset), 99);
        let tx: TransactionId = "XYZ".into();
        assert_eq!(tx.as_str(), "XYZ");
        assert_eq!(String::from(tx.clone()), "XYZ");
        assert_eq!(serde_json::from_str::<AppId>("5").unwrap(), AppId(5));
    }
}
