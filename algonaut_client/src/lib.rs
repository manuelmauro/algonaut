///
pub mod error;
///
mod extensions;
/// Algorand's indexer
pub mod indexer;
/// Key management daemon
pub mod kmd;
/// Api token management utils
pub mod token;

pub type Headers<'a> = Vec<(&'a str, &'a str)>;
