extern crate derive_more;
use derive_more::{Display, Error, From};
use std::fmt::Debug;
#[derive(Clone, Debug, Display, Error, From)]
pub enum TokenParsingError {
    /// Token has an invalid length.
    #[display(fmt = "Token too short or too long.")]
    InvalidLength,
}

#[derive(Clone, Debug, Display, Error, From)]
pub enum AlgodBuildError {
    /// URL parse error.
    #[display(fmt = "Url parsing error.")]
    BadUrl(url::ParseError),
    /// Token parse error.
    #[display(fmt = "Token parsing error.")]
    BadToken(TokenParsingError),
    /// Missing the base URL of the REST API server.
    #[display(fmt = "Bind the client to URL before calling client().")]
    UnitializedUrl,
    /// Missing the authentication token for the REST API server.
    #[display(fmt = "Authenticate with a token before calling client().")]
    UnitializedToken,
}

#[derive(Debug, Display, Error, From)]
pub struct ReqwestError(reqwest::Error);

#[derive(Debug, Display, Error, From)]
pub struct EncodeError(rmp_serde::encode::Error);

#[derive(Debug, Display, Error, From)]
pub struct JsonError(serde_json::Error);

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
