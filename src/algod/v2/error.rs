use algonaut_algod::apis;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum AlgodError {
    /// General text-only errors. Dedicated error variants can be created, if needed.
    #[error("Msg: {0}")]
    Msg(String),
}

impl<T> From<apis::Error<T>> for AlgodError {
    fn from(_error: apis::Error<T>) -> Self {
        AlgodError::Msg("".to_owned())
    }
}
