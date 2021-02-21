use crate::models::Ed25519PublicKey;
use data_encoding::BASE32_NOPAD;
use serde::Deserialize;

type ChecksumAlg = sha2::Sha512Trunc256;

const CHECKSUM_LEN: usize = 4;
const HASH_LEN: usize = 32;

mod address;
pub use address::{Address, MultisigAddress};

/// An Ed25519 Signature
#[derive(Copy, Clone)]
pub struct Signature(pub [u8; 64]);

#[derive(Default, Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct MultisigSignature {
    #[serde(rename = "subsig")]
    pub subsigs: Vec<MultisigSubsig>,
    #[serde(rename = "thr")]
    pub threshold: u8,
    #[serde(rename = "v")]
    pub version: u8,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct MultisigSubsig {
    #[serde(rename = "pk")]
    pub key: Ed25519PublicKey,
    #[serde(rename = "s")]
    pub sig: Option<Signature>,
}
