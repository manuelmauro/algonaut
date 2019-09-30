use serde::{Serialize, Deserialize};

/// Support for turning 32 byte keys into human-readable mnemonics and back
pub mod mnemonic;
pub mod account;
pub mod crypto;
pub mod transaction;
pub mod auction;
/// Key management daemon
pub mod kmd;
/// Algorand protocol daemon
pub mod algod;
pub(crate) mod util;

pub const MICRO_ALGO_CONVERSION_FACTOR: f64 = 1e6;

pub use kmd::KmdClient;
pub use algod::AlgodClient;
/// MicroAlgos are the base unit of currency in Algorand
#[derive(Copy, Clone, Default, Debug, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct MicroAlgos(pub u64);
/// Round of the Algorand consensus protocol
#[derive(Copy, Clone, Default, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Round(pub u64);
/// A SHA512_256 hash
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct HashDigest(pub [u8; 32]);
/// Participation public key used in key registration transactions
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct VotePK(pub [u8; 32]);
/// VRF public key used in key registration transaction
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct VRFPK(pub [u8; 32]);
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Ed25519PublicKey(pub [u8; 32]);
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MasterDerivationKey(pub [u8; 32]);

impl MicroAlgos {
    pub fn to_algos(self) -> f64 {
        self.0 as f64 / MICRO_ALGO_CONVERSION_FACTOR
    }

    pub fn from_algos(algos: f64) -> MicroAlgos {
        MicroAlgos((algos * MICRO_ALGO_CONVERSION_FACTOR) as u64)
    }
}

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    Encode(rmp_serde::encode::Error),
    Json(serde_json::Error),
    Api(String),
}

