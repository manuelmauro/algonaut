use algonaut_client::{
    error::AlgorandError,
    indexer::v2::{
        message::{
            AccountInfoResponse, AccountResponse, AccountTransactionResponse,
            ApplicationInfoResponse, ApplicationResponse, AssetResponse, AssetTransactionResponse,
            AssetsInfoResponse, BalancesResponse, Block, QueryAccount, QueryAccountInfo,
            QueryAccountTransaction, QueryApplicationInfo, QueryApplications,
            QueryAssetTransaction, QueryAssets, QueryAssetsInfo, QueryBalances, QueryTransaction,
            TransactionResponse,
        },
        Client,
    },
};
use algonaut_core::Round;

pub struct Indexer {
    pub(super) client: Client,
}

impl Indexer {
    pub fn new(client: Client) -> Indexer {
        Indexer { client }
    }

    /// Returns Ok if healthy
    pub async fn health(&self) -> Result<(), AlgorandError> {
        self.client.health().await
    }

    /// Search for accounts.
    pub async fn accounts(&self, query: &QueryAccount) -> Result<AccountResponse, AlgorandError> {
        self.client.accounts(query).await
    }

    /// Lookup account information.
    pub async fn account_info(
        &self,
        id: &str,
        query: &QueryAccountInfo,
    ) -> Result<AccountInfoResponse, AlgorandError> {
        self.client.account_info(id, query).await
    }

    /// Lookup account transactions.
    pub async fn account_transactions(
        &self,
        id: &str,
        query: &QueryAccountTransaction,
    ) -> Result<AccountTransactionResponse, AlgorandError> {
        self.client.account_transactions(id, query).await
    }

    /// Search for applications
    pub async fn applications(
        &self,
        query: &QueryApplications,
    ) -> Result<ApplicationResponse, AlgorandError> {
        self.client.applications(query).await
    }

    /// Lookup application.
    pub async fn application_info(
        &self,
        id: &str,
        query: &QueryApplicationInfo,
    ) -> Result<ApplicationInfoResponse, AlgorandError> {
        self.client.application_info(id, query).await
    }

    /// Search for assets.
    pub async fn assets(&self, query: &QueryAssets) -> Result<AssetResponse, AlgorandError> {
        self.client.assets(query).await
    }

    /// Lookup asset information.
    pub async fn assets_info(
        &self,
        id: &str,
        query: &QueryAssetsInfo,
    ) -> Result<AssetsInfoResponse, AlgorandError> {
        self.client.assets_info(id, query).await
    }

    /// Lookup the list of accounts who hold this asset.
    pub async fn asset_balances(
        &self,
        id: &str,
        query: &QueryBalances,
    ) -> Result<BalancesResponse, AlgorandError> {
        self.client.asset_balances(id, query).await
    }

    /// Lookup transactions for an asset.
    pub async fn asset_transactions(
        &self,
        id: &str,
        query: &QueryAssetTransaction,
    ) -> Result<AssetTransactionResponse, AlgorandError> {
        self.client.asset_transactions(id, query).await
    }

    /// Lookup block.
    pub async fn block(&self, round: Round) -> Result<Block, AlgorandError> {
        self.client.block(round).await
    }

    /// Search for transactions.
    pub async fn transactions(
        &self,
        query: &QueryTransaction,
    ) -> Result<TransactionResponse, AlgorandError> {
        self.client.transactions(query).await
    }

    /// Search for transactions.
    pub async fn transaction_info(&self, id: &str) -> Result<TransactionResponse, AlgorandError> {
        self.client.transaction_info(id).await
    }
}
