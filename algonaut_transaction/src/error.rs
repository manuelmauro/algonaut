extern crate derive_more;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransactionError {
    #[error("Transaction sender does not match multisig identity.")]
    InvalidSenderInMultisig,
    #[error("Multisig identity does not contain this secret key.")]
    InvalidSecretKeyInMultisig,
    #[error("Can't merge only one transaction.")]
    InsufficientTransactions,
    #[error("Multisig signatures to merge must have the same number of subsignatures.")]
    InvalidNumberOfSubsignatures,
    #[error("Transaction msig public keys do not match.")]
    InvalidPublicKeyInMultisig,
    #[error("Transaction msig has mismatched signatures.")]
    MismatchingSignatures,
    #[error("Empty transaction list.")]
    EmptyTransactionListError,
    #[error("Max group size is {}.", size)]
    MaxTransactionGroupSizeError { size: usize },
    #[error("serde encode error {0}")]
    RmpSerdeError(#[from] rmp_serde::encode::Error),
    #[error("crypto error {0}")]
    MnemonicError(#[from] algonaut_crypto::error::CryptoError),
    #[error("Deserialization error: {0}")]
    Deserialization(String),
}
