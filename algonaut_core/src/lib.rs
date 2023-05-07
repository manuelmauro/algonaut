use algonaut_crypto::HashDigest;
use algonaut_crypto::Signature;
use algonaut_encoding::U8_32Visitor;
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
        .map_err(|v| CoreError::General(format!("Couldn't convert vec: {:?} into u8 array", v)))
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
            _ => Err(CoreError::General(format!(
                "Couldn't convert tx type str: `{s}` to tx type"
            ))),
        }
    }
}

/// Returns the address corresponding to an application's escrow account.
pub fn to_app_address(app_id: u64) -> Address {
    let bytes = app_id.to_be_bytes();
    let all_bytes = ["appID".as_bytes(), &bytes].concat();
    let hash = sha2::Sha512_256::digest(all_bytes);
    Address(hash.into())
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
}
