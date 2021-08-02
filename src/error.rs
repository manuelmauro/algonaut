use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum AlgonautError {
    /// URL parse error.
    #[error("Url parsing error.")]
    BadUrl(String),
    /// Token parse error.
    #[error("Token parsing error.")]
    BadToken,
    /// Header parse error.
    #[error("Headers parsing error.")]
    BadHeader(String),
    /// Missing the base URL of the REST API server.
    #[error("Set an URL before calling build.")]
    UnitializedUrl,
    /// Missing the authentication token for the REST API server.
    #[error("Set a token before calling build.")]
    UnitializedToken,
    /// HTTP calls errors
    #[error("http error: {0}")]
    Request(RequestError),
    /// Internal errors (please open an [issue](https://github.com/manuelmauro/algonaut/issues)!)
    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Error, Debug, PartialEq, Eq)]
#[error("{:?}, {}", url, details)]
pub struct RequestError {
    pub url: Option<String>,
    pub details: RequestErrorDetails,
}

impl RequestError {
    pub fn new(url: Option<String>, details: RequestErrorDetails) -> RequestError {
        RequestError { url, details }
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum RequestErrorDetails {
    /// Http call error with optional message (returned by remote API)
    #[error("Http error: {}, {}", status, message)]
    Http { status: u16, message: String },
    /// Timeout
    #[error("Timeout connecting to the server.")]
    Timeout,
    /// Client generated errors (while e.g. building request or decoding response)
    #[error("Client error: {}", description)]
    Client { description: String },
}

impl From<algonaut_client::error::ClientError> for AlgonautError {
    fn from(error: algonaut_client::error::ClientError) -> Self {
        match error {
            algonaut_client::error::ClientError::BadUrl(msg) => AlgonautError::BadUrl(msg),
            algonaut_client::error::ClientError::BadToken => AlgonautError::BadToken,
            algonaut_client::error::ClientError::BadHeader(msg) => AlgonautError::BadHeader(msg),
            algonaut_client::error::ClientError::Request(e) => AlgonautError::Request(e.into()),
        }
    }
}

impl From<algonaut_client::error::RequestError> for RequestError {
    fn from(error: algonaut_client::error::RequestError) -> Self {
        RequestError::new(error.url.clone(), error.details.into())
    }
}

impl From<algonaut_client::error::RequestErrorDetails> for RequestErrorDetails {
    fn from(details: algonaut_client::error::RequestErrorDetails) -> Self {
        match details {
            algonaut_client::error::RequestErrorDetails::Http { status, message } => {
                RequestErrorDetails::Http { status, message }
            }
            algonaut_client::error::RequestErrorDetails::Timeout => RequestErrorDetails::Timeout {},
            algonaut_client::error::RequestErrorDetails::Client { description } => {
                RequestErrorDetails::Client { description }
            }
        }
    }
}

impl From<rmp_serde::encode::Error> for AlgonautError {
    fn from(error: rmp_serde::encode::Error) -> Self {
        AlgonautError::Internal(error.to_string())
    }
}
