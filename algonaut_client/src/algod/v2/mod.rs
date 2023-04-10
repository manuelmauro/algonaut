use crate::error::ClientError;
use crate::extensions::reqwest::{to_header_map, ResponseExt};
use crate::Headers;
use algonaut_core::{Address, Round};
use algonaut_model::algod::v2::{
    Account, ApiCompiledTeal, Application, Asset, Block, BlockWithCertificate, Catchup,
    DryrunResponse, GenesisBlock, KeyRegistration, NodeStatus, PendingTransaction,
    PendingTransactions, Supply, TransactionParams, TransactionResponse, Version,
};
use reqwest::header::HeaderMap;
use reqwest::Url;

#[derive(Debug, Clone)]
/// Client for interacting with the Algorand protocol daemon
pub struct Client {
    url: String,
    headers: HeaderMap,
    http_client: reqwest::Client,
}

impl Client {
    pub fn new(url: &str, headers: Headers) -> Result<Client, ClientError> {
        Ok(Client {
            url: Url::parse(url)?.as_ref().into(),
            headers: to_header_map(headers)?,
            http_client: reqwest::Client::new(),
        })
    }

    pub async fn genesis(&self) -> Result<GenesisBlock, ClientError> {
        let response = self
            .http_client
            .get(&format!("{}genesis", self.url))
            .headers(self.headers.clone())
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;

        Ok(response)
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

    pub async fn metrics(&self) -> Result<String, ClientError> {
        let response = self
            .http_client
            .get(&format!("{}metrics", self.url))
            .headers(self.headers.clone())
            .send()
            .await?
            .http_error_for_status()
            .await?
            .text()
            .await?;

        Ok(response)
    }

    pub async fn account_information(&self, address: &str) -> Result<Account, ClientError> {
        let response = self
            .http_client
            .get(&format!("{}v2/accounts/{}", self.url, address))
            .headers(self.headers.clone())
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn pending_transactions_for(
        &self,
        address: &str,
        max: u64,
    ) -> Result<PendingTransactions, ClientError> {
        let response = self
            .http_client
            .get(&format!(
                "{}v2/accounts/{}/transactions/pending",
                self.url, address,
            ))
            .headers(self.headers.clone())
            .query(&[("max", max.to_string())])
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn application_information(&self, id: u64) -> Result<Application, ClientError> {
        let response = self
            .http_client
            .get(&format!("{}v2/applications/{}", self.url, id))
            .headers(self.headers.clone())
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn asset_information(&self, id: u64) -> Result<Asset, ClientError> {
        let response = self
            .http_client
            .get(&format!("{}v2/assets/{}", self.url, id))
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

    pub async fn block_with_certificate(
        &self,
        round: Round,
    ) -> Result<BlockWithCertificate, ClientError> {
        let response_bytes = self
            .http_client
            .get(&format!("{}v2/blocks/{}?format=msgpack", self.url, round))
            .headers(self.headers.clone())
            .send()
            .await?
            .http_error_for_status()
            .await?
            .bytes()
            .await?;

        Ok(rmp_serde::from_slice(&response_bytes)?)
    }

    pub async fn start_catchup(&self, catchpoint: &str) -> Result<Catchup, ClientError> {
        let response = self
            .http_client
            .post(&format!("{}v2/catchup/{}", self.url, catchpoint))
            .headers(self.headers.clone())
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn abort_catchup(&self, catchpoint: &str) -> Result<Catchup, ClientError> {
        let response = self
            .http_client
            .delete(&format!("{}v2/catchup/{}", self.url, catchpoint))
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
            .get(&format!("{}v2/ledger/supply", self.url))
            .headers(self.headers.clone())
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn register_participation_keys(
        &self,
        address: &Address,
        params: &KeyRegistration,
    ) -> Result<String, ClientError> {
        let response = self
            .http_client
            .post(&format!(
                "{}v2/register-participation-keys/{}",
                self.url, address
            ))
            .headers(self.headers.clone())
            .query(&params)
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn shutdown(&self, timeout: usize) -> Result<(), ClientError> {
        self.http_client
            .post(&format!("{}v2/shutdown", self.url))
            .headers(self.headers.clone())
            .query(&[("timeout", timeout.to_string())])
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;

        Ok(())
    }

    pub async fn status(&self) -> Result<NodeStatus, ClientError> {
        let response = self
            .http_client
            .get(&format!("{}v2/status", self.url))
            .headers(self.headers.clone())
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn status_after_round(&self, round: Round) -> Result<NodeStatus, ClientError> {
        let response = self
            .http_client
            .get(&format!(
                "{}v2/status/wait-for-block-after/{}",
                self.url, round.0
            ))
            .headers(self.headers.clone())
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn compile_teal(&self, teal: Vec<u8>) -> Result<ApiCompiledTeal, ClientError> {
        let response = self
            .http_client
            .post(&format!("{}v2/teal/compile", self.url))
            .headers(self.headers.clone())
            .header("Content-Type", "application/x-binary")
            .body(teal)
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn dryrun_teal(&self, req: Vec<u8>) -> Result<DryrunResponse, ClientError> {
        let response = self
            .http_client
            .post(&format!("{}v2/teal/dryrun", self.url))
            .headers(self.headers.clone())
            .header("Content-Type", "application/x-binary")
            .body(req)
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn broadcast_raw_transaction(
        &self,
        rawtxn: &[u8],
    ) -> Result<TransactionResponse, ClientError> {
        let response = self
            .http_client
            .post(&format!("{}v2/transactions", self.url))
            .headers(self.headers.clone())
            .header("Content-Type", "application/x-binary")
            .body(rawtxn.to_vec())
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
            .get(&format!("{}v2/transactions/params", self.url))
            .headers(self.headers.clone())
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn pending_transactions(&self, max: u64) -> Result<PendingTransactions, ClientError> {
        let response = self
            .http_client
            .get(&format!("{}v2/transactions/pending", self.url))
            .headers(self.headers.clone())
            .query(&[("max", max.to_string())])
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn pending_transaction_with_id(
        &self,
        txid: &str,
    ) -> Result<PendingTransaction, ClientError> {
        let response = self
            .http_client
            .get(&format!("{}v2/transactions/pending/{}", self.url, txid))
            .headers(self.headers.clone())
            .send()
            .await?
            .http_error_for_status()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn versions(&self) -> Result<Version, ClientError> {
        let response = self
            .http_client
            .get(&format!("{}versions", self.url))
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
