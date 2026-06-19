//! ARC-74 — the NFT indexer API for ARC-72 tokens.
//!
//! A versioned REST surface an indexer implements to serve current ownership and
//! transfer history for [`arc72`](crate::arc72) NFTs. The request/response types
//! are always available (they are plain serde structs); the async HTTP client is
//! gated behind the `fetch` feature.
//!
//! Note: the ARC-74 text is internally inconsistent about the base path — its
//! intro says `/nft-index/v1` while the endpoint headers say `/nft-indexer/v1`.
//! This client uses the `/nft-indexer/v1` form and lets the base URL be
//! configured, so a server using the other prefix can still be targeted.

use serde::{Deserialize, Serialize};

/// Query parameters for `GET /nft-indexer/v1/tokens`.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct TokensQuery {
    /// Results as of this round.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub round: Option<u64>,
    /// Pagination token from a previous `next-token`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    /// Maximum number of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
    /// Restrict to a contract (application) id.
    #[serde(rename = "contractId", skip_serializing_if = "Option::is_none")]
    pub contract_id: Option<u64>,
    /// Restrict to a token id.
    #[serde(rename = "tokenId", skip_serializing_if = "Option::is_none")]
    pub token_id: Option<u64>,
    /// Restrict to a current owner.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// Minimum mint round.
    #[serde(rename = "mint-min-round", skip_serializing_if = "Option::is_none")]
    pub mint_min_round: Option<u64>,
    /// Maximum mint round.
    #[serde(rename = "mint-max-round", skip_serializing_if = "Option::is_none")]
    pub mint_max_round: Option<u64>,
}

/// A single ARC-72 token as reported by the indexer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Token {
    /// Current owner address.
    pub owner: String,
    /// Contract (application) id.
    #[serde(rename = "contractId")]
    pub contract_id: u64,
    /// Token id within the contract.
    #[serde(rename = "tokenId")]
    pub token_id: u64,
    /// Round in which the token was minted (transferred from the zero address).
    #[serde(
        rename = "mint-round",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mint_round: Option<u64>,
    /// The token's metadata URI.
    #[serde(
        rename = "metadataURI",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub metadata_uri: Option<String>,
    /// The resolved metadata object, if the indexer fetched it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Response body of `GET /nft-indexer/v1/tokens`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TokensResponse {
    /// The matching tokens.
    pub tokens: Vec<Token>,
    /// The round the response is current as of.
    #[serde(rename = "current-round")]
    pub current_round: u64,
    /// Pagination token for the next page, if any.
    #[serde(
        rename = "next-token",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub next_token: Option<String>,
}

/// Query parameters for `GET /nft-indexer/v1/transfers`.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct TransfersQuery {
    /// Results as of this round.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub round: Option<u64>,
    /// Pagination token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    /// Maximum number of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
    /// Restrict to a contract id.
    #[serde(rename = "contractId", skip_serializing_if = "Option::is_none")]
    pub contract_id: Option<u64>,
    /// Restrict to a token id.
    #[serde(rename = "tokenId", skip_serializing_if = "Option::is_none")]
    pub token_id: Option<u64>,
    /// Restrict to a sender or receiver.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Restrict to a sender.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// Restrict to a receiver.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    /// Minimum round.
    #[serde(rename = "min-round", skip_serializing_if = "Option::is_none")]
    pub min_round: Option<u64>,
    /// Maximum round.
    #[serde(rename = "max-round", skip_serializing_if = "Option::is_none")]
    pub max_round: Option<u64>,
}

/// A single ARC-72 transfer event.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transfer {
    /// Contract (application) id.
    #[serde(rename = "contractId")]
    pub contract_id: u64,
    /// Token id.
    #[serde(rename = "tokenId")]
    pub token_id: u64,
    /// Sender address.
    pub from: String,
    /// Receiver address.
    pub to: String,
    /// Round of the transfer.
    pub round: u64,
}

/// Response body of `GET /nft-indexer/v1/transfers`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransfersResponse {
    /// The matching transfers.
    pub transfers: Vec<Transfer>,
    /// The round the response is current as of.
    #[serde(rename = "current-round")]
    pub current_round: u64,
    /// Pagination token for the next page, if any.
    #[serde(
        rename = "next-token",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub next_token: Option<String>,
}

/// An async client for an ARC-74 NFT indexer (requires the `fetch` feature).
#[cfg(feature = "fetch")]
pub struct Arc74Client {
    base_url: String,
    http: reqwest::Client,
}

#[cfg(feature = "fetch")]
impl Arc74Client {
    /// Create a client targeting `base_url` (e.g. `https://arc72-idx.example`)
    /// with a 30-second request timeout.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self::with_timeout(base_url, std::time::Duration::from_secs(30))
    }

    /// Create a client with an explicit request timeout.
    pub fn with_timeout(base_url: impl Into<String>, timeout: std::time::Duration) -> Self {
        Arc74Client {
            base_url: base_url.into(),
            http: reqwest::Client::builder()
                .timeout(timeout)
                .build()
                .expect("reqwest client with default config"),
        }
    }

    /// `GET /nft-indexer/v1/tokens`.
    pub async fn tokens(&self, query: &TokensQuery) -> Result<TokensResponse, crate::NftError> {
        let url = format!(
            "{}/nft-indexer/v1/tokens",
            self.base_url.trim_end_matches('/')
        );
        Ok(self
            .http
            .get(url)
            .query(query)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// `GET /nft-indexer/v1/transfers`.
    pub async fn transfers(
        &self,
        query: &TransfersQuery,
    ) -> Result<TransfersResponse, crate::NftError> {
        let url = format!(
            "{}/nft-indexer/v1/transfers",
            self.base_url.trim_end_matches('/')
        );
        Ok(self
            .http
            .get(url)
            .query(query)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_serialises_to_nothing() {
        let q = TokensQuery::default();
        assert_eq!(serde_json::to_value(&q).unwrap(), serde_json::json!({}));
    }

    #[test]
    fn query_uses_api_field_names() {
        let q = TokensQuery {
            contract_id: Some(7),
            mint_min_round: Some(100),
            ..Default::default()
        };
        let v = serde_json::to_value(&q).unwrap();
        assert_eq!(v["contractId"], 7);
        assert_eq!(v["mint-min-round"], 100);
    }

    #[test]
    fn token_response_deserialises() {
        let body = serde_json::json!({
            "tokens": [{
                "owner": "AAAA",
                "contractId": 1,
                "tokenId": 2,
                "mint-round": 30,
                "metadataURI": "ipfs://x"
            }],
            "current-round": 100
        });
        let r: TokensResponse = serde_json::from_value(body).unwrap();
        assert_eq!(r.current_round, 100);
        assert_eq!(r.tokens[0].token_id, 2);
        assert_eq!(r.tokens[0].metadata_uri.as_deref(), Some("ipfs://x"));
        assert_eq!(r.next_token, None);
    }
}
