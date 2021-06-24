use crate::error::ClientError;
use crate::extensions::reqwest::ResponseExt;
use crate::token::ApiToken;
use algonaut_core::Round;
use message::{
    Account, Block, NodeStatus, PendingTransactions, QueryAccountTransactions, Supply, Transaction,
    TransactionFee, TransactionId, TransactionList, TransactionParams, Version,
};
use reqwest::header::HeaderMap;
use reqwest::Url;

/// API message structs for Algorand's daemon v1
pub mod message;

const AUTH_HEADER: &str = "X-Algo-API-Token";

/// Client for interacting with the Algorand protocol daemon.
pub struct Client {
    url: String,
    token: String,
    headers: HeaderMap,
    http_client: reqwest::Client,
}

impl Client {
    pub fn new(url: &str, token: &str) -> Result<Client, ClientError> {
        Ok(Client {
            url: Url::parse(url)?.as_ref().into(),
            token: ApiToken::parse(token)?.to_string(),
            headers: HeaderMap::new(),
            http_client: reqwest::Client::new(),
        })
    }

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

    pub async fn versions(&self) -> Result<Version, ClientError> {
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

    pub async fn status(&self) -> Result<NodeStatus, ClientError> {
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

    pub async fn status_after_block(&self, round: Round) -> Result<NodeStatus, ClientError> {
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

    pub async fn block(&self, round: Round) -> Result<Block, ClientError> {
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

    pub async fn ledger_supply(&self) -> Result<Supply, ClientError> {
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

    pub async fn account_information(&self, address: &str) -> Result<Account, ClientError> {
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

    pub async fn pending_transactions(
        &self,
        limit: u64,
    ) -> Result<PendingTransactions, ClientError> {
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

    pub async fn pending_transaction_information(
        &self,
        transaction_id: &str,
    ) -> Result<Transaction, ClientError> {
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

    pub async fn transactions(
        &self,
        address: &str,
        query: &QueryAccountTransactions,
    ) -> Result<TransactionList, ClientError> {
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

    pub async fn raw_transaction(&self, raw: &[u8]) -> Result<TransactionId, ClientError> {
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

    pub async fn transaction(&self, transaction_id: &str) -> Result<Transaction, ClientError> {
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

    pub async fn transaction_information(
        &self,
        address: &str,
        transaction_id: &str,
    ) -> Result<Transaction, ClientError> {
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

    pub async fn suggested_fee(&self) -> Result<TransactionFee, ClientError> {
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

    pub async fn transaction_params(&self) -> Result<TransactionParams, ClientError> {
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
