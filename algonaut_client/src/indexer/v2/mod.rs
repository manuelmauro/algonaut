use self::message::*;
use crate::error::AlgorandError;
use algonaut_core::Round;
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

    /// Search for accounts.
    pub fn accounts(&self, query: &QueryAccount) -> Result<AccountResponse, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v2/accounts", self.url))
            .headers(self.headers.clone())
            .header("Content-Type", "application/json")
            .json(query)
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }

    /// Lookup account information.
    pub fn account_info(
        &self,
        id: &str,
        round: Option<Round>,
    ) -> Result<AccountIdResponse, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v2/accounts/{}", self.url, id))
            .headers(self.headers.clone())
            .header("Content-Type", "application/json")
            .json(&(round.map(|r| QueryRound { round: r })))
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }

    /// Lookup account transactions.
    pub fn account_transactions(
        &self,
        id: &str,
        query: &QueryAccountTransaction,
    ) -> Result<AccountTransactionResponse, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v2/accounts/{}/transactions", self.url, id))
            .headers(self.headers.clone())
            .header("Content-Type", "application/json")
            .json(query)
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }

    /// Search for applications
    pub fn applications(
        &self,
        query: &QueryApplications,
    ) -> Result<ApplicationResponse, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v2/applications", self.url))
            .headers(self.headers.clone())
            .header("Content-Type", "application/json")
            .json(query)
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }

    /// Lookup application.
    pub fn application_info(&self, id: &str) -> Result<ApplicationInfoResponse, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v2/applications/{}", self.url, id))
            .headers(self.headers.clone())
            .header("Content-Type", "application/json")
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }

    /// Search for assets.
    pub fn assets(&self, query: &QueryAssets) -> Result<AssetResponse, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v2/assets", self.url))
            .headers(self.headers.clone())
            .header("Content-Type", "application/json")
            .json(query)
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }

    /// Lookup asset information.
    pub fn assets_info(&self, id: &str) -> Result<AssetsInfoResponse, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v2/assets/{}", self.url, id))
            .headers(self.headers.clone())
            .header("Content-Type", "application/json")
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }

    /// Lookup the list of accounts who hold this asset.
    pub fn asset_balances(
        &self,
        id: &str,
        query: &QueryBalances,
    ) -> Result<BalancesResponse, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v2/assets/{}/balances", self.url, id))
            .headers(self.headers.clone())
            .header("Content-Type", "application/json")
            .json(query)
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }

    /// Lookup transactions for an asset.
    pub fn asset_transactions(
        &self,
        id: &str,
        query: &QueryAssetTransaction,
    ) -> Result<AssetTransactionResponse, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v2/assets/{}/balances", self.url, id))
            .headers(self.headers.clone())
            .header("Content-Type", "application/json")
            .json(query)
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }

    /// Lookup block.
    pub fn block(&self, round: Round) -> Result<Block, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v2/blocks/{}", self.url, round))
            .headers(self.headers.clone())
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }

    /// Search for transactions.
    pub fn transactions(
        &self,
        query: &QueryTransaction,
    ) -> Result<TransactionResponse, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v2/transactions", self.url))
            .headers(self.headers.clone())
            .header("Content-Type", "application/json")
            .json(query)
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }

    /// Search for transactions.
    pub fn transaction_info(&self, id: &str) -> Result<TransactionResponse, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v2/transactions/{}", self.url, id))
            .headers(self.headers.clone())
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }
}
