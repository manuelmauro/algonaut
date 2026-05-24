use crate::Signature;
use crate::domain_separator::{MESSAGE_PREFIX, MULTISIG_ADDR_PREFIX};
use algonaut_crypto::Ed25519PublicKey;
use algonaut_encoding::U8_32Visitor;
use data_encoding::BASE32_NOPAD;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::Digest;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::str::FromStr;

type ChecksumAlg = sha2::Sha512_256;

const CHECKSUM_LEN: usize = 4;
const HASH_LEN: usize = 32;

/// Public key address
#[derive(Copy, Clone, Eq)]
pub struct Address(pub [u8; HASH_LEN]);

impl Address {
    pub fn new(bytes: [u8; HASH_LEN]) -> Address {
        Address(bytes)
    }

    /// Decode from base32 string with checksum
    fn decode_from_string(string: &str) -> Result<Address, String> {
        let checksum_address = match BASE32_NOPAD.decode(string.as_bytes()) {
            Ok(decoded) => decoded,
            Err(err) => return Err(format!("Error decoding base32: {:?}", err)),
        };
        if checksum_address.len() != (HASH_LEN + CHECKSUM_LEN) {
            return Err("Input string is an invalid address. Wrong length".to_string());
        }
        let (address, checksum) = checksum_address.split_at(HASH_LEN);
        let hashed = ChecksumAlg::digest(address);
        if &hashed[(HASH_LEN - CHECKSUM_LEN)..] == checksum {
            let mut bytes = [0; HASH_LEN];
            bytes.copy_from_slice(address);
            Ok(Address::new(bytes))
        } else {
            Err("Input checksum did not validate".to_string())
        }
    }

    pub fn as_public_key(&self) -> Ed25519PublicKey {
        Ed25519PublicKey(self.0)
    }

    /// Encode to base32 string with checksum
    fn encode_as_string(&self) -> String {
        let hashed = ChecksumAlg::digest(self.0);
        let checksum = &hashed[(HASH_LEN - CHECKSUM_LEN)..];
        let checksum_address = [&self.0, checksum].concat();
        BASE32_NOPAD.encode(&checksum_address)
    }

    pub fn verify_bytes(&self, message: &[u8], signature: &Signature) -> bool {
        let mut message_to_verify = MESSAGE_PREFIX.to_vec();
        message_to_verify.extend_from_slice(message);
        self.as_public_key().verify(&message_to_verify, signature)
    }
}

impl Hash for Address {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl PartialEq for Address {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl FromStr for Address {
    type Err = String;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        Address::decode_from_string(string)
    }
}

impl Debug for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.encode_as_string())
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.encode_as_string())
    }
}

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        // JSON emits the base32-checksum string; msgpack emits the raw
        // 32 bytes. See ADR domain-types-serialize-for-both-json-and-msgpack.
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.encode_as_string())
        } else {
            serializer.serialize_bytes(&self.0[..])
        }
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            s.parse().map_err(serde::de::Error::custom)
        } else {
            Ok(Address(deserializer.deserialize_bytes(U8_32Visitor)?))
        }
    }
}

/// Convenience struct for handling multisig public identities
#[derive(Debug, Clone, PartialEq, Eq)]
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

    pub fn contains(&self, address: &Address) -> bool {
        self.public_keys.contains(&address.as_public_key())
    }

    /// Generates a checksum from the contained public keys usable as an address
    pub fn address(&self) -> Address {
        let mut buf = MULTISIG_ADDR_PREFIX.to_vec();
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

#[cfg(test)]
mod tests {
    use rand::{Rng, rngs::OsRng};

    use super::*;

    /// Trying to decode a valid base32 address should succeed.
    #[test]
    fn decode() {
        let s = "737777777777777777777777777777777777777777777777777UFEJ2CI";

        let addr = s
            .parse::<Address>()
            .expect("failed to decode address from string");
        assert_eq!(s, addr.to_string());
    }

    /// Trying to decode a base32 address with an invalid checksum must fail.
    #[test]
    fn decode_invalid_checksum() {
        let invalid_csum = "737777777777777777777777777777777777777777777777777UFEJ2CJ";

        assert!(invalid_csum.parse::<Address>().is_err());
    }

    #[test]
    fn encode() {
        let expected = "7777777777777777777777777777777777777777777777777774MSJUVU";
        assert_eq!(expected, Address([255; 32]).to_string())
    }

    #[test]
    fn encode_decode_str() {
        for _ in 0..1_000 {
            let bytes: [u8; 32] = OsRng.r#gen();
            let addr = Address(bytes);
            let addr_str = addr.to_string();
            let reenc_addr = Address::from_str(&addr_str).unwrap();
            assert_eq!(reenc_addr, addr)
        }
    }

    #[test]
    fn serializes_deserializes() {
        let addr = Address(OsRng.r#gen());
        // arbitrary serde serializer
        let bytes = rmp_serde::to_vec_named(&addr).unwrap();
        let deserialized: Address = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(deserialized, addr);
    }

    #[test]
    fn json_round_trip_is_base32() {
        let addr = Address(OsRng.r#gen());
        let json = serde_json::to_string(&addr).unwrap();
        // JSON emits the canonical base32 string (with quotes), not an
        // integer array.
        assert_eq!(json, format!("\"{}\"", addr.encode_as_string()));
        assert!(!json.starts_with('['), "JSON form must not be a byte array");
        let parsed: Address = serde_json::from_str(&json).unwrap();
        assert_eq!(addr, parsed);
    }

    #[test]
    fn msgpack_layout_unchanged() {
        // Regression guard: the msgpack form is a 32-byte bin value
        // (header 0xc4 0x20 followed by 32 raw bytes). On-chain signing
        // depends on this exact byte layout.
        let addr = Address([7; 32]);
        let bytes = rmp_serde::to_vec(&addr).unwrap();
        assert_eq!(bytes[0], 0xc4);
        assert_eq!(bytes[1], 0x20);
        assert_eq!(&bytes[2..], &[7u8; 32]);
    }
}
