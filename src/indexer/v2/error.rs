use std::fmt::Debug;

use algonaut_indexer::apis;
use thiserror::Error;

/// Indexer client error with preserved source chain.
#[derive(Error, Debug)]
pub enum IndexerError {
    /// HTTP/transport error.
    #[error("request error")]
    Reqwest(#[from] reqwest::Error),

    /// JSON decode error.
    #[error("JSON decode error")]
    Decode(#[from] serde_json::Error),

    /// I/O error.
    #[error("I/O error")]
    Io(#[from] std::io::Error),

    /// API returned an error response.
    #[error("API error (status {status}): {content}")]
    ResponseError { status: u16, content: String },
}

impl<T: Debug> From<apis::Error<T>> for IndexerError {
    fn from(error: apis::Error<T>) -> Self {
        match error {
            apis::Error::Reqwest(e) => IndexerError::Reqwest(e),
            apis::Error::Serde(e) => IndexerError::Decode(e),
            apis::Error::Io(e) => IndexerError::Io(e),
            apis::Error::ResponseError(resp) => IndexerError::ResponseError {
                status: resp.status.as_u16(),
                content: resp.content,
            },
        }
    }
}
