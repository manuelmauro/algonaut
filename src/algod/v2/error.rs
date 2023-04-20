use algonaut_algod::apis;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum AlgodError {
    /// General text-only errors. Dedicated error variants can be created, if needed.
    #[error("Msg: {0}")]
    Msg(String),
}

impl From<apis::Error<apis::common_api::HealthCheckError>> for AlgodError {
    fn from(error: apis::Error<apis::common_api::HealthCheckError>) -> Self {
        AlgodError::Msg(error.to_string())
    }
}

impl From<apis::Error<apis::nonparticipating_api::GetStatusError>> for AlgodError {
    fn from(error: apis::Error<apis::nonparticipating_api::GetStatusError>) -> Self {
        AlgodError::Msg(error.to_string())
    }
}
