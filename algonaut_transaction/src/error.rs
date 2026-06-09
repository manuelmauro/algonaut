extern crate derive_more;
use std::error::Error as StdError;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransactionError {
    /// An error from a [`Signer`] implementation.
    ///
    /// This variant allows typed errors from external signers (like WalletConnect)
    /// to flow through the signing pipeline without requiring a reverse dependency.
    #[error("signer error: {0}")]
    Signer(#[source] Box<dyn StdError + Send + Sync>),
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
    #[error("No accounts to sign the transaction.")]
    NoAccountsToSign,
}
