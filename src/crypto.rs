use data_encoding::BASE32_NOPAD;
use serde::Deserialize;
use sha2::Digest;

use crate::Ed25519PublicKey;

type ChecksumAlg = sha2::Sha512Trunc256;

const CHECKSUM_LEN: usize = 4;
const HASH_LEN: usize = 32;

/// Public key address
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Address(pub [u8; HASH_LEN]);

impl Address {
    pub fn new(bytes: [u8; HASH_LEN]) -> Address {
        Address(bytes)
    }

    /// Decode address from base64 string with checksum
    pub fn from_string(string: &str) -> Result<Address, String> {
        let checksum_address = match BASE32_NOPAD.decode(string.as_bytes()) {
            Ok(decoded) => decoded,
            Err(err) => return Err(format!("Error decoding base32: {:?}", err)),
        };
        if checksum_address.len() != (HASH_LEN + CHECKSUM_LEN) {
            return Err("Input string is an invalid address. Wrong length".to_string());
        }
        let (address, checksum) = checksum_address.split_at(HASH_LEN);
        let hashed = ChecksumAlg::digest(&address);
        if &hashed[(HASH_LEN - CHECKSUM_LEN)..] == checksum {
            let mut bytes = [0; HASH_LEN];
            bytes.copy_from_slice(address);
            Ok(Address::new(bytes))
        } else {
            Err("Input checksum did not validate".to_string())
        }
    }

    /// Encode address to base64 string with checksum
    pub fn encode_string(&self) -> String {
        let hashed = ChecksumAlg::digest(&self.0);
        let checksum = &hashed[(HASH_LEN - CHECKSUM_LEN)..];
        let checksum_address = [&self.0, checksum].concat();
        BASE32_NOPAD.encode(&checksum_address)
    }
}

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
