use crate::error::ClientError;
use crate::extensions::reqwest::ResponseExt;
use algonaut_core::Round;
use algonaut_model::indexer::v2::{
    AccountInfoResponse, AccountResponse, AccountTransactionResponse, ApplicationInfoResponse,
    ApplicationResponse, AssetResponse, AssetTransactionResponse, AssetsInfoResponse,
    BalancesResponse, Block, QueryAccount, QueryAccountInfo, QueryAccountTransaction,
    QueryApplicationInfo, QueryApplications, QueryAssetTransaction, QueryAssets, QueryAssetsInfo,
    QueryBalances, QueryTransaction, TransactionResponse,
};
use reqwest::header::HeaderMap;
use reqwest::Url;

/// Client interacting with the Algorand's indexer
pub struct Client {
    pub(super) url: String,
    pub(super) headers: HeaderMap,
    pub(super) http_client: reqwest::Client,
}

impl Client {
    pub fn new(url: &str) -> Result<Client, ClientError> {
        Ok(Client {
            url: Url::parse(url)?.as_ref().into(),
            headers: HeaderMap::new(),
            http_client: reqwest::Client::new(),
        })
    }

    /// Returns Ok if healthy
    pub async fn health(&self) -> Result<(), ClientError> {
        let _ = self
            .http_client
            .get(&format!("{}health", self.url))
            .headers(self.headers.clone())
            .send()
            .await?
            .http_error_for_status()
            .await?;
        Ok(())
    }

    /// Search for accounts.
    pub async fn accounts(&self, query: &QueryAccount) -> Result<AccountResponse, ClientError> {
        let response = self
            .http_client
            .get(&format!("{}v2/accounts", self.url))
            .headers(self.headers.clone())
            .query(query)
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    /// Lookup account information.
    pub async fn account_info(
        &self,
        id: &str,
        query: &QueryAccountInfo,
    ) -> Result<AccountInfoResponse, ClientError> {
        let response = self
            .http_client
            .get(&format!("{}v2/accounts/{}", self.url, id))
            .headers(self.headers.clone())
            .query(query)
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    /// Lookup account transactions.
    pub async fn account_transactions(
        &self,
        id: &str,
        query: &QueryAccountTransaction,
    ) -> Result<AccountTransactionResponse, ClientError> {
        let response = self
            .http_client
            .get(&format!("{}v2/accounts/{}/transactions", self.url, id))
            .headers(self.headers.clone())
            .query(query)
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    /// Search for applications
    pub async fn applications(
        &self,
        query: &QueryApplications,
    ) -> Result<ApplicationResponse, ClientError> {
        let response = self
            .http_client
            .get(&format!("{}v2/applications", self.url))
            .headers(self.headers.clone())
            .query(query)
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    /// Lookup application.
    pub async fn application_info(
        &self,
        id: &str,
        query: &QueryApplicationInfo,
    ) -> Result<ApplicationInfoResponse, ClientError> {
        let response = self
            .http_client
            .get(&format!("{}v2/applications/{}", self.url, id))
            .headers(self.headers.clone())
            .query(query)
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    /// Search for assets.
    pub async fn assets(&self, query: &QueryAssets) -> Result<AssetResponse, ClientError> {
        let response = self
            .http_client
            .get(&format!("{}v2/assets", self.url))
            .headers(self.headers.clone())
            .query(query)
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    /// Lookup asset information.
    pub async fn assets_info(
        &self,
        id: &str,
        query: &QueryAssetsInfo,
    ) -> Result<AssetsInfoResponse, ClientError> {
        let response = self
            .http_client
            .get(&format!("{}v2/assets/{}", self.url, id))
            .headers(self.headers.clone())
            .query(query)
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    /// Lookup the list of accounts who hold this asset.
    pub async fn asset_balances(
        &self,
        id: &str,
        query: &QueryBalances,
    ) -> Result<BalancesResponse, ClientError> {
        let response = self
            .http_client
            .get(&format!("{}v2/assets/{}/balances", self.url, id))
            .headers(self.headers.clone())
            .query(query)
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    /// Lookup transactions for an asset.
    pub async fn asset_transactions(
        &self,
        id: &str,
        query: &QueryAssetTransaction,
    ) -> Result<AssetTransactionResponse, ClientError> {
        let response = self
            .http_client
            .get(&format!("{}v2/assets/{}/transactions", self.url, id))
            .headers(self.headers.clone())
            .query(query)
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    /// Lookup block.
    pub async fn block(&self, round: Round) -> Result<Block, ClientError> {
        let response = self
            .http_client
            .get(&format!("{}v2/blocks/{}", self.url, round))
            .headers(self.headers.clone())
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    /// Search for transactions.
    pub async fn transactions(
        &self,
        query: &QueryTransaction,
    ) -> Result<TransactionResponse, ClientError> {
        let response = self
            .http_client
            .get(&format!("{}v2/transactions", self.url))
            .headers(self.headers.clone())
            .query(query)
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    /// Search for transactions.
    pub async fn transaction_info(&self, id: &str) -> Result<TransactionResponse, ClientError> {
        let response = self
            .http_client
            .get(&format!("{}v2/transactions/{}", self.url, id))
            .headers(self.headers.clone())
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;

        Ok(response)
    }
}
