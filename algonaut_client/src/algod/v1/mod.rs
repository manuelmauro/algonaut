use crate::error::AlgorandError;
use crate::extensions::reqwest::ResponseExt;
use algonaut_core::Round;
use message::{
    Account, Block, NodeStatus, PendingTransactions, QueryAccountTransactions, Supply, Transaction,
    TransactionFee, TransactionId, TransactionList, TransactionParams, Version,
};
use reqwest::header::HeaderMap;

/// API message structs for Algorand's daemon v1
pub mod message;

const AUTH_HEADER: &str = "X-Algo-API-Token";

/// Client for interacting with the Algorand protocol daemon.
pub struct Client {
    pub(super) url: String,
    pub(super) token: String,
    pub(super) headers: HeaderMap,
    pub(super) http_client: reqwest::Client,
}

impl Client {
    /// Returns Ok if healthy
    pub async fn health(&self) -> Result<(), AlgorandError> {
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

    /// Retrieves the current version
    pub async fn versions(&self) -> Result<Version, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}versions", self.url))
            .headers(self.headers.clone())
            .header(AUTH_HEADER, &self.token)
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    /// Gets the current node status
    pub async fn status(&self) -> Result<NodeStatus, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v1/status", self.url))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    /// Waits for a block to appear after the specified round and returns the node status at the time
    pub async fn status_after_block(&self, round: Round) -> Result<NodeStatus, AlgorandError> {
        let response = self
            .http_client
            .get(&format!(
                "{}v1/status/wait-for-block-after/{}",
                self.url, round.0
            ))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    /// Get the block for the given round
    pub async fn block(&self, round: Round) -> Result<Block, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v1/block/{}", self.url, round.0))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    /// Gets the current supply reported by the ledger
    pub async fn ledger_supply(&self) -> Result<Supply, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v1/ledger/supply", self.url))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn account_information(&self, address: &str) -> Result<Account, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v1/account/{}", self.url, address))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    /// Gets a list of unconfirmed transactions currently in the transaction pool
    ///
    /// Sorted by priority in decreasing order and truncated at the specified limit, or returns all if specified limit is 0
    pub async fn pending_transactions(
        &self,
        limit: u64,
    ) -> Result<PendingTransactions, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v1/transactions/pending", self.url))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .query(&[("max", limit.to_string())])
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;
        Ok(response)
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
        let response = self
            .http_client
            .get(&format!(
                "{}v1/transactions/pending/{}",
                self.url, transaction_id
            ))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    /// Get a list of confirmed transactions, limited to filters if specified
    pub async fn transactions(
        &self,
        address: &str,
        query: &QueryAccountTransactions,
    ) -> Result<TransactionList, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v1/account/{}/transactions", self.url, address))
            .header(AUTH_HEADER, &self.token)
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

    /// Broadcasts a raw transaction to the network
    pub async fn raw_transaction(&self, raw: &[u8]) -> Result<TransactionId, AlgorandError> {
        let response = self
            .http_client
            .post(&format!("{}v1/transactions", self.url))
            .header(AUTH_HEADER, &self.token)
            .header("Content-Type", "application/x-binary")
            .headers(self.headers.clone())
            .body(raw.to_vec())
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    /// Gets the information of a single transaction
    pub async fn transaction(&self, transaction_id: &str) -> Result<Transaction, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v1/transaction/{}", self.url, transaction_id))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    /// Gets a specific confirmed transaction
    pub async fn transaction_information(
        &self,
        address: &str,
        transaction_id: &str,
    ) -> Result<Transaction, AlgorandError> {
        let response = self
            .http_client
            .get(&format!(
                "{}v1/account/{}/transaction/{}",
                self.url, address, transaction_id
            ))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    /// Gets suggested fee in units of micro-Algos per byte
    pub async fn suggested_fee(&self) -> Result<TransactionFee, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v1/transactions/fee", self.url))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    /// Gets parameters for constructing a new transaction
    pub async fn transaction_params(&self) -> Result<TransactionParams, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v1/transactions/params", self.url))
            .header(AUTH_HEADER, &self.token)
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
