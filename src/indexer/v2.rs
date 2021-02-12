use crate::error::AlgorandError;
use reqwest::header::HeaderMap;

/// Client interacting with the Algorand's indexer
pub struct Client {
    pub(super) url: String,
    pub(super) headers: HeaderMap,
}

impl Client {
    pub fn new(address: &str) -> Client {
        Client::new_with_headers(address, HeaderMap::new())
    }

    pub fn new_with_headers(address: &str, headers: HeaderMap) -> Client {
        Client {
            url: address.to_string(),
            headers,
        }
    }

    /// Returns Ok if healthy
    pub fn health(&self) -> Result<(), AlgorandError> {
        let _ = reqwest::Client::new()
            .get(&format!("{}health", self.url))
            .headers(self.headers.clone())
            .send()?
            .error_for_status()?;
        Ok(())
    }
}
