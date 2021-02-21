use crate::error::{AlgorandError, BuilderError};
use reqwest::header::HeaderMap;
use url::Url;

mod v2;

/// Indexer is the entry point to the creation of a client for the Algorand's indexer
pub struct Indexer<'a> {
    url: Option<&'a str>,
    headers: HeaderMap,
}

impl<'a> Indexer<'a> {
    /// Start the creation of a client.
    pub fn new() -> Self {
        Self::default()
    }

    /// Bind to a URL.
    pub fn bind(mut self, url: &'a str) -> Self {
        self.url = Some(url);
        self
    }

    /// Build a v2 client for Algorand's indexer.
    pub fn client_v2(self) -> Result<v2::Client, AlgorandError> {
        match self.url {
            Some(url) => Ok(v2::Client {
                url: Url::parse(url)?.into_string(),
                headers: self.headers,
                http_client: reqwest::Client::new(),
            }),
            None => Err(BuilderError::UnitializedUrl.into()),
        }
    }
}

impl<'a> Default for Indexer<'a> {
    fn default() -> Self {
        Indexer {
            url: None,
            headers: HeaderMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_client_builder() {
        let indexer = Indexer::new().bind("http://example.com").client_v2();

        assert!(indexer.ok().is_some());
    }

    #[test]
    #[should_panic(expected = "")]
    fn test_client_builder_with_no_url() {
        let _ = Indexer::new().client_v2().unwrap();
    }
}
