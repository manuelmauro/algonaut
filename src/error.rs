use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Clone, Debug, PartialEq, Eq)]
pub enum ServiceError {
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

impl ServiceError {
    /// Returns if the error is a `RequestError` that failed with a status code of 404.
    pub fn is_404(&self) -> bool {
        if let Some(e) = self.as_request_error() {
            e.is_404()
        } else {
            false
        }
    }

    /// Gets the details of a request error, or none otherwise.
    fn as_request_error(&self) -> Option<&RequestError> {
        match self {
            Self::Request(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Error, Clone, Debug, PartialEq, Eq)]
#[error("{:?}, {}", url, details)]
pub struct RequestError {
    pub url: Option<String>,
    pub details: RequestErrorDetails,
}

impl RequestError {
    pub fn new(url: Option<String>, details: RequestErrorDetails) -> RequestError {
        RequestError { url, details }
    }

    /// Returns if the cause of the error is a 404 response from the client.
    fn is_404(&self) -> bool {
        self.details.status() == Some(404)
    }
}

#[derive(Error, Clone, Debug, PartialEq, Eq)]
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

impl RequestErrorDetails {
    /// Gets the status code of the request.
    ///
    /// Returns `None` if the request did not receive a response.
    fn status(&self) -> Option<u16> {
        match self {
            Self::Http { status, .. } => Some(*status),
            _ => None,
        }
    }
}

impl From<algonaut_client::error::ClientError> for ServiceError {
    fn from(error: algonaut_client::error::ClientError) -> Self {
        match error {
            algonaut_client::error::ClientError::BadUrl(msg) => ServiceError::BadUrl(msg),
            algonaut_client::error::ClientError::BadToken => ServiceError::BadToken,
            algonaut_client::error::ClientError::BadHeader(msg) => ServiceError::BadHeader(msg),
            algonaut_client::error::ClientError::Request(e) => ServiceError::Request(e.into()),
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

impl From<rmp_serde::encode::Error> for ServiceError {
    fn from(error: rmp_serde::encode::Error) -> Self {
        ServiceError::Internal(error.to_string())
    }
}

impl From<String> for ServiceError {
    fn from(error: String) -> Self {
        ServiceError::Internal(error)
    }
}

#[test]
fn check_404() {
    let not_found_error = ServiceError::Request(RequestError::new(
        Some("testing".to_owned()),
        RequestErrorDetails::Http {
            status: 404,
            message: "not found".to_owned(),
        },
    ));

    let bad_request_error = ServiceError::Request(RequestError::new(
        None,
        RequestErrorDetails::Http {
            status: 400,
            message: "bad request".to_owned(),
        },
    ));

    let unrelated_error = ServiceError::UnitializedToken;

    assert!(
        not_found_error.is_404(),
        "a 404 request error is saying that it is not a 404 error"
    );
    assert!(
        !bad_request_error.is_404(),
        "a 400 request error is saying that it is a 404 error"
    );
    assert!(
        !unrelated_error.is_404(),
        "an unrelated request error is saying that it is a 404 error"
    );
}
