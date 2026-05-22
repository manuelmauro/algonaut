extern crate derive_more;
use data_encoding::DecodeError;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum CoreError {
    /// Base64 decoding failed.
    #[error("base64 decode error: {0}")]
    Base64Decode(#[from] DecodeError),

    /// A byte slice could not be converted to a fixed-size array.
    #[error("expected {expected} bytes, got {actual}")]
    InvalidArraySize { expected: usize, actual: usize },

    /// A transaction type string was not recognized.
    #[error("invalid transaction type: `{0}`")]
    InvalidTransactionType(String),
}
