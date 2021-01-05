use derive_more::Display;
use std::fmt::Debug;

#[derive(Debug, Display)]
pub enum Error {
    #[display(fmt = "Parsing error.")]
    Url,
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
            Error::Url => None,
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
//
// impl Display for Error {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Error::Reqwest(e) => Display::fmt(e, f),
//             Error::Url => Display::fmt(e, f),
//             Error::Encode(e) => Display::fmt(e, f),
//             Error::Json(e) => Display::fmt(e, f),
//             Error::Api(e) => Display::fmt(e, f),
//         }
//     }
// }
//
