use crate::error::AlgorandError;
use reqwest::header::HeaderMap;

/// API message structs for Algorand's indexer v2
pub mod message;

/// Client interacting with the Algorand's indexer
pub struct Client {
    pub(super) url: String,
    pub(super) headers: HeaderMap,
    pub(super) http_client: reqwest::blocking::Client,
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
}
