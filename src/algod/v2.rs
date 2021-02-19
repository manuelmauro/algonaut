use crate::error::AlgorandError;
use crate::models::Version;
use reqwest::header::HeaderMap;

const AUTH_HEADER: &str = "X-Algo-API-Token";

/// Client for interacting with the Algorand protocol daemon
pub struct Client {
    pub(super) url: String,
    pub(super) token: String,
    pub(super) headers: HeaderMap,
    pub(super) http_client: reqwest::Client,
}

impl Client {
    /// Returns Ok if healthy
    pub fn health(&self) -> Result<(), AlgorandError> {
        let _ = self
            .http_client
            .get(&format!("{}health", self.url))
            .headers(self.headers.clone())
            .send()?
            .error_for_status()?;
        Ok(())
    }

    /// Retrieves the current version
    pub fn versions(&self) -> Result<Version, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}versions", self.url))
            .headers(self.headers.clone())
            .header(AUTH_HEADER, &self.token)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }
}
