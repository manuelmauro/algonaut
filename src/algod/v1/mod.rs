use algonaut_client::{
    algod::v1::{
        message::{
            Account, Block, NodeStatus, PendingTransactions, QueryAccountTransactions, Supply,
            Transaction, TransactionFee, TransactionId, TransactionList, TransactionParams,
            Version,
        },
        Client,
    },
    error::AlgorandError,
};
use algonaut_core::Round;

pub struct Algod {
    pub(crate) client: Client,
}

impl Algod {
    pub fn new(client: Client) -> Algod {
        Algod { client }
    }

    pub async fn health(&self) -> Result<(), AlgorandError> {
        self.client.health().await
    }

    /// Retrieves the current version
    pub async fn versions(&self) -> Result<Version, AlgorandError> {
        self.client.versions().await
    }

    /// Gets the current node status
    pub async fn status(&self) -> Result<NodeStatus, AlgorandError> {
        self.client.status().await
    }

    /// Waits for a block to appear after the specified round and returns the node status at the time
    pub async fn status_after_block(&self, round: Round) -> Result<NodeStatus, AlgorandError> {
        self.client.status_after_block(round).await
    }

    /// Get the block for the given round
    pub async fn block(&self, round: Round) -> Result<Block, AlgorandError> {
        self.client.block(round).await
    }

    /// Gets the current supply reported by the ledger
    pub async fn ledger_supply(&self) -> Result<Supply, AlgorandError> {
        self.client.ledger_supply().await
    }

    pub async fn account_information(&self, address: &str) -> Result<Account, AlgorandError> {
        self.client.account_information(address).await
    }

    /// Gets a list of unconfirmed transactions currently in the transaction pool
    ///
    /// Sorted by priority in decreasing order and truncated at the specified limit, or returns all if specified limit is 0
    pub async fn pending_transactions(
        &self,
        limit: u64,
    ) -> Result<PendingTransactions, AlgorandError> {
        self.client.pending_transactions(limit).await
    }

    /// Get a specified pending transaction
    ///
    /// Given a transaction id of a recently submitted transaction, it returns information
    /// about it. There are several cases when this might succeed: - transaction committed
    /// (committed round > 0) - transaction still in the pool (committed round = 0, pool
    /// error = "") - transaction removed from pool due to error (committed round = 0, pool
    /// error != "") Or the transaction may have happened sufficiently long ago that the
    /// node no longer remembers it, and this will return an error.
    pub async fn pending_transaction_information(
        &self,
        transaction_id: &str,
    ) -> Result<Transaction, AlgorandError> {
        self.client
            .pending_transaction_information(transaction_id)
            .await
    }

    /// Get a list of confirmed transactions, limited to filters if specified
    pub async fn transactions(
        &self,
        address: &str,
        query: &QueryAccountTransactions,
    ) -> Result<TransactionList, AlgorandError> {
        self.client.transactions(address, query).await
    }

    /// Broadcasts a raw transaction to the network
    pub async fn raw_transaction(&self, raw: &[u8]) -> Result<TransactionId, AlgorandError> {
        self.client.raw_transaction(raw).await
    }

    /// Gets the information of a single transaction
    pub async fn transaction(&self, transaction_id: &str) -> Result<Transaction, AlgorandError> {
        self.client.transaction(transaction_id).await
    }

    /// Gets a specific confirmed transaction
    pub async fn transaction_information(
        &self,
        address: &str,
        transaction_id: &str,
    ) -> Result<Transaction, AlgorandError> {
        self.client
            .transaction_information(address, transaction_id)
            .await
    }

    /// Gets suggested fee in units of micro-Algos per byte
    pub async fn suggested_fee(&self) -> Result<TransactionFee, AlgorandError> {
        self.client.suggested_fee().await
    }

    /// Gets parameters for constructing a new transaction
    pub async fn transaction_params(&self) -> Result<TransactionParams, AlgorandError> {
        self.client.transaction_params().await
    }
}
