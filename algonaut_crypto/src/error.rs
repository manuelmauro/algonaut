extern crate derive_more;
use derive_more::{Display, From};
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Display, Error, From)]
pub enum CryptoError {
    #[display(fmt = "Key length is invalid.")]
    InvalidKeyLength,
    #[display(fmt = "Mnemonic length is invalid.")]
    InvalidMnemonicLength,
    #[display(fmt = "Mnemonic contains invalid words.")]
    InvalidWordsInMnemonic,
    #[display(fmt = "Invalid checksum.")]
    InvalidChecksum,
}
