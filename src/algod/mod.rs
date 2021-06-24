use algonaut_client::error::ClientError;

pub mod v1;
pub mod v2;

/// AlgodBuilder is the entry point to the creation of a client for the Algorand protocol daemon.
/// ```
/// use algonaut::algod::AlgodBuilder;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let algod = AlgodBuilder::new()
///         .bind("http://localhost:4001")
///         .auth("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
///         .build_v2()?;
///
///     println!("Algod versions: {:?}", algod.versions().await?.versions);
///
///     Ok(())
/// }
/// ```
pub struct AlgodBuilder<'a> {
    url: Option<&'a str>,
    token: Option<&'a str>,
}

impl<'a> AlgodBuilder<'a> {
    /// Start the creation of a client.
    pub fn new() -> Self {
        Self::default()
    }

    /// Bind to a URL.
    pub fn bind(mut self, url: &'a str) -> Self {
        self.url = Some(url);
        self
    }

    /// Use a token to authenticate.
    pub fn auth(mut self, token: &'a str) -> Self {
        self.token = Some(token);
        self
    }

    /// Build a v1 client for Algorand protocol daemon.
    pub fn build_v1(self) -> Result<v1::Algod, ClientError> {
        match (self.url, self.token) {
            (Some(url), Some(token)) => Ok(v1::Algod::new(
                algonaut_client::algod::v1::Client::new(url, token)?,
            )),
            (None, Some(_)) => Err(ClientError::UnitializedUrl),
            (Some(_), None) => Err(ClientError::UnitializedToken),
            (None, None) => Err(ClientError::UnitializedUrl),
        }
    }

    /// Build a v2 client for Algorand protocol daemon.
    pub fn build_v2(self) -> Result<v2::Algod, ClientError> {
        match (self.url, self.token) {
            (Some(url), Some(token)) => Ok(v2::Algod::new(
                algonaut_client::algod::v2::Client::new(url, token)?,
            )),
            (None, Some(_)) => Err(ClientError::UnitializedUrl),
            (Some(_), None) => Err(ClientError::UnitializedToken),
            (None, None) => Err(ClientError::UnitializedUrl),
        }
    }
}

impl<'a> Default for AlgodBuilder<'a> {
    fn default() -> Self {
        AlgodBuilder {
            url: None,
            token: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_client_builder() {
        let algod = AlgodBuilder::new()
            .bind("http://example.com")
            .auth("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .build_v1();

        assert!(algod.ok().is_some());
    }

    #[test]
    #[should_panic(expected = "")]
    fn test_client_builder_with_no_token() {
        let _ = AlgodBuilder::new()
            .bind("http://example.com")
            .build_v1()
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "")]
    fn test_client_builder_with_no_url() {
        let _ = AlgodBuilder::new()
            .auth("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .build_v1()
            .unwrap();
    }
}
