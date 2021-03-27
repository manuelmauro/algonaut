use super::token::ApiToken;
use crate::error::{AlgorandError, BuilderError};
use url::Url;

pub mod v1;

/// Kmd is the entry point to the creation of a client for the Algorand key management daemon.
/// ```
/// use algonaut_client::Kmd;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let algod = Kmd::new()
///         .bind("http://localhost:4001")
///         .auth("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
///         .client_v1()?;
///
///     println!("Algod versions: {:?}", algod.versions()?.versions);
///
///     Ok(())
/// }
/// ```
pub struct Kmd<'a> {
    url: Option<&'a str>,
    token: Option<&'a str>,
}

impl<'a> Kmd<'a> {
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
            (Some(url), Some(token)) => Ok(v1::Client::new(
                Url::parse(url)?.as_str(),
                ApiToken::parse(token)?.to_string().as_ref(),
            )),
            (None, Some(_)) => Err(BuilderError::UnitializedUrl.into()),
            (Some(_), None) => Err(BuilderError::UnitializedToken.into()),
            (None, None) => Err(BuilderError::UnitializedUrl.into()),
        }
    }
}

impl<'a> Default for Kmd<'a> {
    fn default() -> Self {
        Kmd {
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
        let algod = Kmd::new()
            .bind("http://example.com")
            .auth("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .client_v1();

        assert!(algod.ok().is_some());
    }

    #[test]
    #[should_panic(expected = "")]
    fn test_client_builder_with_no_token() {
        let _ = Kmd::new().bind("http://example.com").client_v1().unwrap();
    }

    #[test]
    #[should_panic(expected = "")]
    fn test_client_builder_with_no_url() {
        let _ = Kmd::new()
            .auth("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .client_v1()
            .unwrap();
    }
}
