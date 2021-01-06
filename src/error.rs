use derive_more::{Display, Error};
use std::fmt::Debug;

#[derive(Debug, Display, Error)]
pub enum TokenParsingError {
    #[display(fmt = "Token too short or too long.")]
    WrongLength,
}

#[derive(Debug, Display, Error)]
pub enum AlgodBuildError {
    #[display(fmt = "Url parsing error.")]
    BadUrl,
    #[display(fmt = "Token parsing error.")]
    BadToken,
    #[display(fmt = "Bind the client to URL before calling client().")]
    UnitializedUrl,
    #[display(fmt = "Authenticate with a token before calling client().")]
    UnitializedToken,
}

impl From<url::ParseError> for AlgodBuildError {
    fn from(_err: url::ParseError) -> Self {
        AlgodBuildError::BadUrl
    }
}

impl From<TokenParsingError> for AlgodBuildError {
    fn from(_err: TokenParsingError) -> Self {
        AlgodBuildError::BadToken
    }
}

#[derive(Debug, Display)]
pub enum Error {
    #[display(fmt = "{}", _0)]
    Reqwest(reqwest::Error),
    #[display(fmt = "{}", _0)]
    Encode(rmp_serde::encode::Error),
    #[display(fmt = "{}", _0)]
    Json(serde_json::Error),
    #[display(fmt = "{}", _0)]
    Api(String),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Reqwest(e) => Some(e),
            Error::Encode(e) => Some(e),
            Error::Json(e) => Some(e),
            Error::Api(_) => None,
        }
    }
}

impl From<rmp_serde::encode::Error> for Error {
    fn from(err: rmp_serde::encode::Error) -> Self {
        Error::Encode(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Reqwest(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err)
    }
}
