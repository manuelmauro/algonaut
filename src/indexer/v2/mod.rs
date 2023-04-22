use algonaut_client::{indexer::v2::Client, Headers};
use algonaut_core::{Address, Round};
use algonaut_model::indexer::v2::{
    AccountInfoResponse, AccountResponse, AccountTransactionResponse, ApplicationInfoResponse,
    ApplicationResponse, AssetResponse, AssetTransactionResponse, AssetsInfoResponse,
    BalancesResponse, Block, QueryAccount, QueryAccountInfo, QueryAccountTransaction,
    QueryApplicationInfo, QueryApplications, QueryAssetTransaction, QueryAssets, QueryAssetsInfo,
    QueryBalances, QueryTransaction, TransactionInfoResponse, TransactionResponse,
};

use crate::Error;

#[derive(Debug, Clone)]
pub struct Indexer {
    pub(super) client: Client,
}

impl Indexer {
    /// Build a v2 client for Algorand's indexer.
    ///
    /// For third party providers / custom headers, use [with_headers](Self::with_headers).
    ///
    /// Returns an error if the url has an invalid format.
    pub fn new(url: &str) -> Result<Indexer, Error> {
        Self::with_headers(url, vec![])
    }

    /// Build a v2 client for Algorand's indexer.
    ///
    /// Use this initializer when interfacing with third party services, that require custom headers.
    ///
    /// Returns an error if the url or the headers have an invalid format.
    pub fn with_headers(url: &str, headers: Headers) -> Result<Indexer, Error> {
        Ok(Indexer {
            client: Client::new(url, headers)?,
        })
    }

    /// Returns Ok if healthy
    pub async fn health(&self) -> Result<(), Error> {
        Ok(self.client.health().await?)
    }

    /// Search for accounts.
    pub async fn accounts(&self, query: &QueryAccount) -> Result<AccountResponse, Error> {
        Ok(self.client.accounts(query).await?)
    }

    /// Lookup account information.
    pub async fn account_info(
        &self,
        address: &Address,
        query: &QueryAccountInfo,
    ) -> Result<AccountInfoResponse, Error> {
        Ok(self.client.account_info(address, query).await?)
    }

    /// Lookup account transactions.
    pub async fn account_transactions(
        &self,
        address: &Address,
        query: &QueryAccountTransaction,
    ) -> Result<AccountTransactionResponse, Error> {
        Ok(self.client.account_transactions(address, query).await?)
    }

    /// Search for applications
    pub async fn applications(
        &self,
        query: &QueryApplications,
    ) -> Result<ApplicationResponse, Error> {
        Ok(self.client.applications(query).await?)
    }

    /// Lookup application.
    pub async fn application_info(
        &self,
        id: u64,
        query: &QueryApplicationInfo,
    ) -> Result<ApplicationInfoResponse, Error> {
        Ok(self.client.application_info(id, query).await?)
    }

    /// Search for assets.
    pub async fn assets(&self, query: &QueryAssets) -> Result<AssetResponse, Error> {
        Ok(self.client.assets(query).await?)
    }

    /// Lookup asset information.
    pub async fn assets_info(
        &self,
        id: u64,
        query: &QueryAssetsInfo,
    ) -> Result<AssetsInfoResponse, Error> {
        Ok(self.client.assets_info(id, query).await?)
    }

    /// Lookup the list of accounts who hold this asset.
    pub async fn asset_balances(
        &self,
        id: u64,
        query: &QueryBalances,
    ) -> Result<BalancesResponse, Error> {
        Ok(self.client.asset_balances(id, query).await?)
    }

    /// Lookup transactions for an asset.
    pub async fn asset_transactions(
        &self,
        id: u64,
        query: &QueryAssetTransaction,
    ) -> Result<AssetTransactionResponse, Error> {
        Ok(self.client.asset_transactions(id, query).await?)
    }

    /// Lookup block.
    pub async fn block(&self, round: Round) -> Result<Block, Error> {
        Ok(self.client.block(round).await?)
    }

    /// Search for transactions.
    pub async fn transactions(
        &self,
        query: &QueryTransaction,
    ) -> Result<TransactionResponse, Error> {
        Ok(self.client.transactions(query).await?)
    }

    /// Search for transactions.
    pub async fn transaction_info(&self, id: &str) -> Result<TransactionInfoResponse, Error> {
        Ok(self.client.transaction_info(id).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_with_valid_url() {
        let indexer = Indexer::new("http://example.com");
        assert!(indexer.ok().is_some());
    }

    #[test]
    #[should_panic(expected = "")]
    fn test_create_with_empty_url() {
        Indexer::new("").unwrap();
    }
}
