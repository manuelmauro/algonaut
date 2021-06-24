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
    /// Related with HTTP calls
    #[error("http error: {0}")]
    RequestError(#[from] RequestError),

    // TODO remove after adding AlgonautError, client doesn't serialize to msgpack anymore
    /// Serialization error
    #[error("serde encode error: {0}")]
    RmpSerdeError(#[from] rmp_serde::encode::Error),
}

#[derive(Error, Debug)]
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

#[derive(Error, Debug)]
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
        ClientError::RequestError(request_error)
    }
}
