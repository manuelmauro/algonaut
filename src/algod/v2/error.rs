use std::fmt::Debug;

use algonaut_algod::apis;
use thiserror::Error;

/// Algod client error with preserved source chain.
#[derive(Error, Debug)]
pub enum AlgodError {
    /// HTTP/transport error.
    #[error("request error")]
    Reqwest(#[from] reqwest::Error),

    /// JSON decode error.
    #[error("JSON decode error")]
    Decode(#[from] serde_json::Error),

    /// MessagePack decode error.
    #[error("msgpack decode error")]
    Msgpack(#[from] rmp_serde::decode::Error),

    /// I/O error.
    #[error("I/O error")]
    Io(#[from] std::io::Error),

    /// API returned an error response.
    #[error("API error (status {status}): {content}")]
    ResponseError { status: u16, content: String },
}

impl<T: Debug> From<apis::Error<T>> for AlgodError {
    fn from(error: apis::Error<T>) -> Self {
        match error {
            apis::Error::Reqwest(e) => AlgodError::Reqwest(e),
            apis::Error::Serde(e) => AlgodError::Decode(e),
            apis::Error::Msgpack(e) => AlgodError::Msgpack(e),
            apis::Error::Io(e) => AlgodError::Io(e),
            apis::Error::ResponseError(resp) => AlgodError::ResponseError {
                status: resp.status.as_u16(),
                content: resp.content,
            },
        }
    }
}
