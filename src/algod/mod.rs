use crate::error::{AlgorandError, BuilderError};
use crate::util::ApiToken;
use reqwest::header::HeaderMap;
use url::Url;

mod v1;
mod v2;

/// Algod is the entry point to the creation of a client for the Algorand protocol daemon.
/// ```
/// use algorand_rs::Algod;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let algod = Algod::new()
///         .bind("http://localhost:4001")
///         .auth("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
///         .client_v1()?;
///
///     println!("Algod versions: {:?}", algod.versions()?.versions);
///
///     Ok(())
/// }
/// ```
pub struct Algod<'a> {
    url: Option<&'a str>,
    token: Option<&'a str>,
    headers: HeaderMap,
}

impl<'a> Algod<'a> {
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
    pub fn client_v1(self) -> Result<v1::Client, AlgorandError> {
        match (self.url, self.token) {
            (Some(url), Some(token)) => Ok(v1::Client {
                url: Url::parse(url)?.into_string(),
                token: ApiToken::parse(token)?.to_string(),
                headers: self.headers,
                http_client: reqwest::Client::new(),
            }),
            (None, Some(_)) => Err(BuilderError::UnitializedUrl.into()),
            (Some(_), None) => Err(BuilderError::UnitializedToken.into()),
            (None, None) => Err(BuilderError::UnitializedUrl.into()),
        }
    }

    /// Build a v2 client for Algorand protocol daemon.
    pub fn client_v2(self) -> Result<v2::Client, AlgorandError> {
        match (self.url, self.token) {
            (Some(url), Some(token)) => Ok(v2::Client {
                url: Url::parse(url)?.into_string(),
                token: token.to_string(),
                headers: self.headers,
                http_client: reqwest::Client::new(),
            }),
            (None, Some(_)) => Err(BuilderError::UnitializedUrl.into()),
            (Some(_), None) => Err(BuilderError::UnitializedToken.into()),
            (None, None) => Err(BuilderError::UnitializedUrl.into()),
        }
    }
}

impl<'a> Default for Algod<'a> {
    fn default() -> Self {
        Algod {
            url: None,
            token: None,
            headers: HeaderMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_client_builder() {
        let algod = Algod::new()
            .bind("http://example.com")
            .auth("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .client_v1();

        assert!(algod.ok().is_some());
    }

    #[test]
    #[should_panic(expected = "")]
    fn test_client_builder_with_no_token() {
        let _ = Algod::new().bind("http://example.com").client_v1().unwrap();
    }

    #[test]
    #[should_panic(expected = "")]
    fn test_client_builder_with_no_url() {
        let _ = Algod::new()
            .auth("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .client_v1()
            .unwrap();
    }
}
