extern crate derive_more;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum AbiError {
    /// An ABI type string could not be parsed (e.g. "uint256[]").
    #[error("invalid ABI type {input:?}: {reason}")]
    TypeParse { input: String, reason: String },

    /// Encoding a value into its ABI byte representation failed.
    #[error("ABI encode error: {reason}")]
    Encode { reason: String },

    /// Decoding ABI bytes failed (corrupted framing, short input, etc.).
    #[error("ABI decode error: {reason}")]
    Decode { reason: String },

    /// A method selector / signature string could not be parsed.
    #[error("invalid ABI method signature {input:?}: {reason}")]
    MethodSignature { input: String, reason: String },

    /// A value was outside the range its ABI type allows.
    #[error("value out of range for {abi_type}: {reason}")]
    ValueOutOfRange { abi_type: String, reason: String },
}

/// A grammar error from [`algonaut_abi_sig`] is, from this crate's point of
/// view, a type-parse failure: the shared grammar is what `AbiType::from_str`
/// delegates to.
impl From<algonaut_abi_sig::SigError> for AbiError {
    fn from(e: algonaut_abi_sig::SigError) -> Self {
        AbiError::TypeParse {
            input: e.input,
            reason: e.reason,
        }
    }
}
