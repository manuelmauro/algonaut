use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum AlgodError {
    /// General text-only errors. Dedicated error variants can be created, if needed.
    #[error("Msg: {0}")]
    Msg(String),
}

impl From<algonaut_algod::apis::Error<algonaut_algod::apis::common_api::HealthCheckError>>
    for AlgodError
{
    fn from(
        _error: algonaut_algod::apis::Error<algonaut_algod::apis::common_api::HealthCheckError>,
    ) -> Self {
        AlgodError::Msg("HealthCheckError".to_owned())
    }
}
