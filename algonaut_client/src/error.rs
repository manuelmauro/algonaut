use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    /// URL parse error.
    #[error("Url parsing error.")]
    BadUrl(#[from] url::ParseError),
    /// Token parse error.
    #[error("Token parsing error.")]
    BadToken,
    /// Missing the base URL of the REST API server.
    #[error("Bind the client to URL before calling client().")]
    UnitializedUrl,
    /// Missing the authentication token for the REST API server.
    #[error("Authenticate with a token before calling client().")]
    UnitializedToken,
    /// Http error
    #[error("http error: {0}")]
    HttpError(#[from] HttpError),

    // TODO remove after adding AlgonautError, client doesn't serialize to msgpack anymore
    /// Serialization error
    #[error("serde encode error: {0}")]
    RmpSerdeError(#[from] rmp_serde::encode::Error),
}

#[derive(Error, Debug)]
#[error("{}, {}", message, reqwest_error)]
pub struct HttpError {
    pub message: String,
    pub reqwest_error: reqwest::Error,
}

impl From<reqwest::Error> for ClientError {
    fn from(error: reqwest::Error) -> Self {
        ClientError::HttpError(HttpError {
            reqwest_error: error,
            message: "".to_owned(),
        })
    }
}
