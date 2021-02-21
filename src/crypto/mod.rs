use crate::models::Ed25519PublicKey;
use data_encoding::BASE32_NOPAD;
use serde::Deserialize;
use sha2::Digest;

type ChecksumAlg = sha2::Sha512Trunc256;

const CHECKSUM_LEN: usize = 4;
const HASH_LEN: usize = 32;

mod address;
pub use address::Address;

/// Convenience struct for handling multisig public identities
#[derive(Debug, Clone)]
pub struct MultisigAddress {
    /// the version of this multisig
    pub version: u8,
    /// how many signatures are needed to fully sign as this address
    pub threshold: u8,
    /// ordered list of public keys that could potentially sign a message
    pub public_keys: Vec<Ed25519PublicKey>,
}

impl MultisigAddress {
    pub fn new(
        version: u8,
        threshold: u8,
        addresses: &[Address],
    ) -> Result<MultisigAddress, String> {
        if version != 1 {
            Err("Unknown msig version".to_string())
        } else if threshold == 0 || addresses.is_empty() || threshold > addresses.len() as u8 {
            Err("Invalid threshold".to_string())
        } else {
            Ok(MultisigAddress {
                version,
                threshold,
                public_keys: addresses
                    .iter()
                    .map(|address| Ed25519PublicKey(address.0))
                    .collect(),
            })
        }
    }

    /// Generates a checksum from the contained public keys usable as an address
    pub fn address(&self) -> Address {
        let mut buf = b"MultisigAddr".to_vec();
        buf.push(self.version);
        buf.push(self.threshold);
        for key in &self.public_keys {
            buf.extend_from_slice(&key.0);
        }
        let hashed = ChecksumAlg::digest(&buf);
        let mut bytes = [0; HASH_LEN];
        bytes.copy_from_slice(&hashed);
        Address::new(bytes)
    }
}

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
