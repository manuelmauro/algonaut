use super::{ChecksumAlg, HASH_LEN, BASE32_NOPAD, CHECKSUM_LEN};
use sha2::Digest;

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

#[cfg(test)]
mod tests {
    use super::*;


    /// Trying to decode a valid base32 address should succeed.
    #[test]
    fn decode() {
        let s = "737777777777777777777777777777777777777777777777777UFEJ2CI";

        let addr = Address::from_string(s).expect("failed to decode address from string");
        assert_eq!(s, addr.encode_string());
    }

    /// Tryng to decode a base32 address with an invalid checksum must fail.
    #[test]
    fn decode_invalid_checksum() {
        let invalid_csum = "737777777777777777777777777777777777777777777777777UFEJ2CJ";

        assert!(Address::from_string(invalid_csum).is_err());
    }


}