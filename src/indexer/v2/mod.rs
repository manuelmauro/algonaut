use algonaut_client::indexer::v2::Client;
use algonaut_core::Round;
use algonaut_model::indexer::v2::{
    AccountInfoResponse, AccountResponse, AccountTransactionResponse, ApplicationInfoResponse,
    ApplicationResponse, AssetResponse, AssetTransactionResponse, AssetsInfoResponse,
    BalancesResponse, Block, QueryAccount, QueryAccountInfo, QueryAccountTransaction,
    QueryApplicationInfo, QueryApplications, QueryAssetTransaction, QueryAssets, QueryAssetsInfo,
    QueryBalances, QueryTransaction, TransactionResponse,
};

use crate::error::AlgonautError;

pub struct Indexer {
    pub(super) client: Client,
}

impl Indexer {
    pub fn new(client: Client) -> Indexer {
        Indexer { client }
    }

    /// Returns Ok if healthy
    pub async fn health(&self) -> Result<(), AlgonautError> {
        Ok(self.client.health().await?)
    }

    /// Search for accounts.
    pub async fn accounts(&self, query: &QueryAccount) -> Result<AccountResponse, AlgonautError> {
        Ok(self.client.accounts(query).await?)
    }

    /// Lookup account information.
    pub async fn account_info(
        &self,
        id: &str,
        query: &QueryAccountInfo,
    ) -> Result<AccountInfoResponse, AlgonautError> {
        Ok(self.client.account_info(id, query).await?)
    }

    /// Lookup account transactions.
    pub async fn account_transactions(
        &self,
        id: &str,
        query: &QueryAccountTransaction,
    ) -> Result<AccountTransactionResponse, AlgonautError> {
        Ok(self.client.account_transactions(id, query).await?)
    }

    /// Search for applications
    pub async fn applications(
        &self,
        query: &QueryApplications,
    ) -> Result<ApplicationResponse, AlgonautError> {
        Ok(self.client.applications(query).await?)
    }

    /// Lookup application.
    pub async fn application_info(
        &self,
        id: &str,
        query: &QueryApplicationInfo,
    ) -> Result<ApplicationInfoResponse, AlgonautError> {
        Ok(self.client.application_info(id, query).await?)
    }

    /// Search for assets.
    pub async fn assets(&self, query: &QueryAssets) -> Result<AssetResponse, AlgonautError> {
        Ok(self.client.assets(query).await?)
    }

    /// Lookup asset information.
    pub async fn assets_info(
        &self,
        id: &str,
        query: &QueryAssetsInfo,
    ) -> Result<AssetsInfoResponse, AlgonautError> {
        Ok(self.client.assets_info(id, query).await?)
    }

    /// Lookup the list of accounts who hold this asset.
    pub async fn asset_balances(
        &self,
        id: &str,
        query: &QueryBalances,
    ) -> Result<BalancesResponse, AlgonautError> {
        Ok(self.client.asset_balances(id, query).await?)
    }

    /// Lookup transactions for an asset.
    pub async fn asset_transactions(
        &self,
        id: &str,
        query: &QueryAssetTransaction,
    ) -> Result<AssetTransactionResponse, AlgonautError> {
        Ok(self.client.asset_transactions(id, query).await?)
    }

    /// Lookup block.
    pub async fn block(&self, round: Round) -> Result<Block, AlgonautError> {
        Ok(self.client.block(round).await?)
    }

    /// Search for transactions.
    pub async fn transactions(
        &self,
        query: &QueryTransaction,
    ) -> Result<TransactionResponse, AlgonautError> {
        Ok(self.client.transactions(query).await?)
    }

    /// Search for transactions.
    pub async fn transaction_info(&self, id: &str) -> Result<TransactionResponse, AlgonautError> {
        Ok(self.client.transaction_info(id).await?)
    }
}
