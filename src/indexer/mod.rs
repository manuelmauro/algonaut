use crate::error::AlgonautError;
use algonaut_client::{indexer::v2::Client, Headers};

pub mod v2;

/// Indexer is the entry point to the creation of a client for the Algorand's indexer
#[derive(Default)]
pub struct IndexerBuilder<'a> {
    url: Option<&'a str>,
}

impl<'a> IndexerBuilder<'a> {
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
    ///
    /// Returns an error if url is not set or has an invalid format.
    pub fn build_v2(self) -> Result<v2::Indexer, AlgonautError> {
        match self.url {
            Some(url) => Ok(v2::Indexer::new(Client::new(url, vec![])?)),
            None => Err(AlgonautError::UnitializedUrl),
        }
    }
}

#[derive(Default)]
pub struct IndexerCustomEndpointBuilder<'a> {
    url: Option<&'a str>,
    headers: Headers<'a>,
}

impl<'a> IndexerCustomEndpointBuilder<'a> {
    /// Start the creation of a client.
    pub fn new() -> Self {
        Self::default()
    }

    /// Bind to a URL.
    pub fn bind(mut self, url: &'a str) -> Self {
        self.url = Some(url);
        self
    }

    /// Add custom headers to requests.
    pub fn headers(mut self, headers: Headers<'a>) -> Self {
        self.headers = headers;
        self
    }

    /// Build a v2 client for Algorand's indexer.
    ///
    /// Returns an error if url is not set or has an invalid format.
    pub fn build_v2(self) -> Result<v2::Indexer, AlgonautError> {
        match self.url {
            Some(url) => Ok(v2::Indexer::new(Client::new(url, self.headers)?)),
            None => Err(AlgonautError::UnitializedUrl),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_client_builder() {
        let indexer = IndexerBuilder::new().bind("http://example.com").build_v2();

        assert!(indexer.ok().is_some());
    }

    #[test]
    #[should_panic(expected = "")]
    fn test_client_builder_with_no_url() {
        let _ = IndexerBuilder::new().build_v2().unwrap();
    }
}
