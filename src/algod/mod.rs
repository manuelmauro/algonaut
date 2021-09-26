use algonaut_client::{token::ApiToken, Headers};

use crate::error::AlgonautError;

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
///         .header("HeaderName", "HeaderData")
///         .build_v2()?;
///
///     println!("Algod versions: {:?}", algod.versions().await?.versions);
///
///     Ok(())
/// }
/// ```
#[derive(Default)]
pub struct AlgodBuilder<'a> {
    url: Option<&'a str>,
    token: Option<&'a str>,
    additional_headers: Headers<'a>,
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

    /// Add an extra header to the client.
    pub fn header(mut self, header_name: &'a str, header_data: &'a str) -> Self {
        self.additional_headers.push((header_name, header_data));
        self
    }

    /// Build a v1 client for Algorand protocol daemon.
    ///
    /// Returns an error if url or token is not set or has an invalid format.
    pub fn build_v1(self) -> Result<v1::Algod, AlgonautError> {
        match (self.url, self.token) {
            (Some(url), Some(token)) => Ok(v1::Algod::new(
                algonaut_client::algod::v1::Client::new_with_header(
                    url,
                    &ApiToken::parse(token)?.to_string(),
                    self.additional_headers,
                )?,
            )),
            (None, Some(_)) => Err(AlgonautError::UnitializedUrl),
            (Some(_), None) => Err(AlgonautError::UnitializedToken),
            (None, None) => Err(AlgonautError::UnitializedUrl),
        }
    }

    /// Build a v2 client for Algorand protocol daemon.
    ///
    /// Returns an error if url or token is not set or has an invalid format.
    pub fn build_v2(self) -> Result<v2::Algod, AlgonautError> {
        match (self.url, self.token, self.additional_headers) {
            (Some(url), Some(token), mut headers) => {
                let token = ApiToken::parse(token)?.to_string();
                headers.push(("X-Algo-API-Token", token.as_str()));
                Ok(v2::Algod::new(algonaut_client::algod::v2::Client::new(
                    url,
                    headers,
                )?))
            }
            (None, Some(_), _) => Err(AlgonautError::UnitializedUrl),
            (Some(_), None, _) => Err(AlgonautError::UnitializedToken),
            (None, None, _) => Err(AlgonautError::UnitializedUrl),
        }
    }
}

#[derive(Default)]
pub struct AlgodCustomEndpointBuilder<'a> {
    url: Option<&'a str>,
    headers: Headers<'a>,
}

impl<'a> AlgodCustomEndpointBuilder<'a> {
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

    /// Build a v2 client for Algorand protocol daemon.
    ///
    /// Returns an error if url or token is not set or has an invalid format.
    pub fn build_v2(self) -> Result<v2::Algod, AlgonautError> {
        match self.url {
            Some(url) => Ok(v2::Algod::new(algonaut_client::algod::v2::Client::new(
                url,
                self.headers,
            )?)),
            None => Err(AlgonautError::UnitializedUrl),
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
            .build_v2();

        assert!(algod.ok().is_some());
    }

    #[test]
    fn test_client_builder_with_no_token() {
        let res = AlgodBuilder::new().bind("http://example.com").build_v2();
        assert!(res.is_err());
        assert!(res.err().unwrap() == AlgonautError::UnitializedToken);
    }

    #[test]
    fn test_client_builder_with_no_url() {
        let res = AlgodBuilder::new()
            .auth("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .build_v2();
        assert!(res.is_err());
        assert!(res.err().unwrap() == AlgonautError::UnitializedUrl);
    }

    #[test]
    fn test_client_builder_with_invalid_url() {
        let res = AlgodBuilder::new()
            .bind("asfdsdfs")
            .auth("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .build_v2();
        assert!(res.is_err());
        assert!(matches!(res.err().unwrap(), AlgonautError::BadUrl(_)));
    }

    #[test]
    fn test_client_builder_with_invalid_url_no_scheme() {
        let res = AlgodBuilder::new()
            .bind("example.com")
            .auth("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .build_v2();
        assert!(res.is_err());
        assert!(matches!(res.err().unwrap(), AlgonautError::BadUrl(_)));
    }

    #[test]
    fn test_client_builder_with_invalid_token() {
        let res = AlgodBuilder::new()
            .bind("http://example.com")
            .auth("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .build_v2();
        assert!(res.is_err());
        assert!(res.err().unwrap() == AlgonautError::BadToken);
    }

    #[test]
    fn test_client_builder_with_invalid_token_empty() {
        let res = AlgodBuilder::new()
            .bind("http://example.com")
            .auth("")
            .build_v2();
        assert!(res.is_err());
        assert!(res.err().unwrap() == AlgonautError::BadToken);
    }
}
