use reqwest::header::{InvalidHeaderName, InvalidHeaderValue};
use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ClientError {
    /// URL parse error.
    #[error("Url parsing error: {0}")]
    BadUrl(String),
    /// Token parse error.
    #[error("Token parsing error.")]
    BadToken,
    /// Invalid characters in headers, e.g. "\n"
    /// Can only happen with custom endpoints
    #[error("Header parsing error: {0}")]
    BadHeader(String),
    /// HTTP calls.
    #[error("http error: {0}")]
    Request(#[from] RequestError),
    /// General text-only errors. Dedicated error variants can be created, if needed.
    #[error("Msg: {0}")]
    Msg(String),
}

#[derive(Error, Debug, Clone)]
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

#[derive(Error, Debug, Clone)]
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

impl From<url::ParseError> for ClientError {
    fn from(error: url::ParseError) -> Self {
        ClientError::BadUrl(error.to_string())
    }
}

impl From<rmp_serde::decode::Error> for ClientError {
    fn from(error: rmp_serde::decode::Error) -> Self {
        ClientError::Msg(error.to_string())
    }
}

impl From<reqwest::Error> for ClientError {
    fn from(error: reqwest::Error) -> Self {
        let url_str = error.url().map(|u| u.to_string());
        let request_error = if let Some(status) = error.status() {
            RequestError::new(
                url_str,
                RequestErrorDetails::Http {
                    status: status.as_u16(),
                    message: "".to_owned(),
                },
            )
        } else if error.is_timeout() {
            RequestError::new(url_str, RequestErrorDetails::Timeout)
        } else {
            RequestError::new(
                url_str,
                RequestErrorDetails::Client {
                    description: error.to_string(),
                },
            )
        };
        ClientError::Request(request_error)
    }
}

impl From<InvalidHeaderName> for ClientError {
    fn from(error: InvalidHeaderName) -> Self {
        ClientError::BadHeader(error.to_string())
    }
}

impl From<InvalidHeaderValue> for ClientError {
    fn from(error: InvalidHeaderValue) -> Self {
        ClientError::BadHeader(error.to_string())
    }
}
