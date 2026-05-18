extern crate derive_more;
use derive_more::{Display, From};
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Display, Error, From, Clone)]
pub enum CryptoError {
    #[display("Key length is invalid.")]
    InvalidKeyLength,
    #[display("Mnemonic length is invalid.")]
    InvalidMnemonicLength,
    #[display("Mnemonic contains invalid words.")]
    InvalidWordsInMnemonic,
    #[display("Invalid checksum.")]
    InvalidChecksum,
}
