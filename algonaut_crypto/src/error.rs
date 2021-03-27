extern crate derive_more;
use derive_more::{Display, From};
use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AlgorandError {
    /// An api error.
    #[error("api error {0}")]
    ApiError(#[from] ApiError),
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
