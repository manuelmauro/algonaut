extern crate derive_more;
use derive_more::{Display, From};
use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AlgorandError {
    /// A builder error.
    #[error("builder error {0}")]
    BuilderError(#[from] BuilderError),
    /// An api error.
    #[error("api error {0}")]
    ApiError(#[from] ApiError),
    /// A url parsing error
    #[error("parse error {0}")]
    BadUrl(#[from] url::ParseError),
    /// Http error
    #[error("http error {0}")]
    HttpError(#[from] reqwest::Error),
    /// Serialization error
    #[error("serde encode error {0}")]
    RmpSerdeError(#[from] rmp_serde::encode::Error),
    /// Serialization error
    #[error("serde encode error {0}")]
    SerdeJsonError(#[from] serde_json::Error),
}

#[derive(Debug, Display, Error, From)]
pub enum BuilderError {
    /// URL parse error.
    #[display(fmt = "Url parsing error.")]
    BadUrl(url::ParseError),
    /// Token parse error.
    #[display(fmt = "Token parsing error.")]
    BadToken,
    /// Missing the base URL of the REST API server.
    #[display(fmt = "Bind the client to URL before calling client().")]
    UnitializedUrl,
    /// Missing the authentication token for the REST API server.
    #[display(fmt = "Authenticate with a token before calling client().")]
    UnitializedToken,
}

#[derive(Debug, Display, Error, From)]
pub enum ApiError {
    #[display(fmt = "Key length is invalid.")]
    InvalidKeyLength,
    #[display(fmt = "Mnemonic length is invalid.")]
    InvalidMnemonicLength,
    #[display(fmt = "Mnemonic contains invalid words.")]
    InvalidWordsInMnemonic,
    #[display(fmt = "Invalid checksum.")]
    InvalidChecksum,
    #[display(fmt = "Transaction sender does not match multisig identity.")]
    InvalidSenderInMultisig,
    #[display(fmt = "Multisig identity does not contain this secret key.")]
    InvalidSecretKeyInMultisig,
    #[display(fmt = "Can't merge only one transaction.")]
    InsufficientTransactions,
    #[display(fmt = "Multisig signatures to merge must have the same number of subsignatures.")]
    InvalidNumberOfSubsignatures,
    #[display(fmt = "Transaction msig public keys do not match.")]
    InvalidPublicKeyInMultisig,
    #[display(fmt = "Transaction msig has mismatched signatures.")]
    MismatchingSignatures,
    #[display(fmt = "Response error: {}", response)]
    ResponseError { response: String },
}
