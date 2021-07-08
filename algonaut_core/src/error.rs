extern crate derive_more;
use data_encoding::DecodeError;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Core error: {0}")]
    General(String),
}

impl From<DecodeError> for CoreError {
    fn from(e: DecodeError) -> Self {
        CoreError::General(format!("Decoding error: {}", e))
    }
}
