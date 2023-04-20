use algonaut_algod::apis;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum AlgodError {
    /// General text-only errors. Dedicated error variants can be created, if needed.
    #[error("Msg: {0}")]
    Msg(String),
}

impl From<apis::Error<apis::common_api::HealthCheckError>> for AlgodError {
    fn from(_error: apis::Error<apis::common_api::HealthCheckError>) -> Self {
        AlgodError::Msg("HealthCheckError".to_owned())
    }
}

impl From<apis::Error<apis::nonparticipating_api::GetStatusError>> for AlgodError {
    fn from(_error: apis::Error<apis::nonparticipating_api::GetStatusError>) -> Self {
        AlgodError::Msg("GetStatusError".to_owned())
    }
}
