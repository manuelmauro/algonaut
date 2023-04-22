use crate::error::ClientError;
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
    pub fn parse(token: &str) -> Result<Self, ClientError> {
        if token.len() != TOKEN_LENGTH {
            return Err(ClientError::BadToken);
        }

        Ok(ApiToken {
            token: token.to_string(),
        })
    }
}
