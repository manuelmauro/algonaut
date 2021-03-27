use algonaut_encoding::U8_32Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use static_assertions::_core::ops::{Add, Sub};
use std::fmt::{Debug, Display, Formatter};
use std::ops::Mul;

/// Algorand addresses
pub mod address;

pub const MICRO_ALGO_CONVERSION_FACTOR: f64 = 1e6;

/// MicroAlgos are the base unit of currency in Algorand
#[derive(Copy, Clone, Default, Debug, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct MicroAlgos(pub u64);

impl MicroAlgos {
    pub fn to_algos(self) -> f64 {
        self.0 as f64 / MICRO_ALGO_CONVERSION_FACTOR
    }

    pub fn from_algos(algos: f64) -> MicroAlgos {
        MicroAlgos((algos * MICRO_ALGO_CONVERSION_FACTOR) as u64)
    }
}

impl Display for MicroAlgos {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
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
// If you're multiplying a MicroAlgos by MicroAlgos, something has gone wrong in your math
// That would give you MicroAlgos squared and those don't exist
impl Mul<u64> for MicroAlgos {
    type Output = Self;

    fn mul(self, rhs: u64) -> Self::Output {
        MicroAlgos(self.0 * rhs)
    }
}

/// Round of the Algorand consensus protocol
#[derive(Copy, Clone, Default, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Round(pub u64);

impl Display for Round {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
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
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
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

/// VRF public key used in key registration transaction
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
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
