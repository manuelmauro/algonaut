use crate::abi_error::AbiError;
use num_bigint::BigUint;
use std::ops::Shl;

pub trait BigUintExt {
    /// Encode as a big endian byte vector with `len` length (padded with leading 0s if needed).
    /// Errors if value cannot be represented with `len` bytes.
    fn to_bytes_be_padded(&self, len: usize) -> Result<Vec<u8>, AbiError>;
}

impl BigUintExt for BigUint {
    fn to_bytes_be_padded(&self, len: usize) -> Result<Vec<u8>, AbiError> {
        if self >= &BigUint::from(1u8).shl(len * 8) {
            return Err(AbiError::Msg(format!(
                "Encode int to byte: integer size for: {self} exceeds the given byte number: {len}"
            )));
        }

        let bytes = self.to_bytes_be();
        let mut new_buffer = vec![0; len];
        let start = len - bytes.len();
        // This panics if slices have different lengths, but checks / logic in this function prevent this from happening.
        new_buffer[start..].clone_from_slice(&bytes);
        Ok(new_buffer)
    }
}
