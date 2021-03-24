use crate::error::{AlgorandError, BuilderError};
use derive_more::Display;

/// An API token.
#[derive(Display)]
#[display(fmt = "{}", token)]
pub struct ApiToken {
    token: String,
}

const TOKEN_LENGTH: usize = 64;

impl ApiToken {
    /// Parses a string slice representing an API token.
    pub fn parse(token: &str) -> Result<Self, AlgorandError> {
        if token.len() != TOKEN_LENGTH {
            return Err(BuilderError::BadToken.into());
        }

        Ok(ApiToken {
            token: token.to_string(),
        })
    }
}
