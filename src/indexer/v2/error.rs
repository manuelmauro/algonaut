use std::fmt::Debug;

use algonaut_indexer::apis;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum IndexerError {
    /// General text-only errors. Dedicated error variants can be created, if needed.
    #[error("Msg: {0}")]
    Msg(String),
}

impl<T: Debug> From<apis::Error<T>> for IndexerError {
    fn from(error: apis::Error<T>) -> Self {
        IndexerError::Msg(format!("{:?}", error))
    }
}
