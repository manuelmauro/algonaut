use crate::error::ClientError;
use crate::extensions::reqwest::ResponseExt;
use crate::token::ApiToken;
use algonaut_core::{Address, Round};
use message::*;
use reqwest::header::HeaderMap;
use reqwest::Url;

/// API message structs for Algorand's daemon v2
pub mod message;

const AUTH_HEADER: &str = "X-Algo-API-Token";

/// Client for interacting with the Algorand protocol daemon
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
            .header(AUTH_HEADER, &self.token)
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
            .header(AUTH_HEADER, &self.token)
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
            .header(AUTH_HEADER, &self.token)
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

    pub async fn application_information(&self, id: usize) -> Result<Application, ClientError> {
        let response = self
            .http_client
            .get(&format!("{}v2/applications/{}", self.url, id))
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

    pub async fn asset_information(&self, id: usize) -> Result<Application, ClientError> {
        let response = self
            .http_client
            .get(&format!("{}v2/asset/{}", self.url, id))
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

    pub async fn block(&self, round: Round) -> Result<Block, ClientError> {
        let response = self
            .http_client
            .get(&format!("{}v2/blocks/{}", self.url, round))
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

    pub async fn start_catchup(&self, catchpoint: &str) -> Result<Catchup, ClientError> {
        let response = self
            .http_client
            .post(&format!("{}v2/catchup/{}", self.url, catchpoint))
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

    pub async fn abort_catchup(&self, catchpoint: &str) -> Result<Catchup, ClientError> {
        let response = self
            .http_client
            .delete(&format!("{}v2/catchup/{}", self.url, catchpoint))
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

    pub async fn ledger_supply(&self) -> Result<Supply, ClientError> {
        let response = self
            .http_client
            .get(&format!("{}v2/ledger/supply", self.url))
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

    pub async fn register_participation_keys(
        &self,
        address: &Address,
        params: &KeyRegistration,
    ) -> Result<String, ClientError> {
        let response = self
            .http_client
            .post(&format!(
                "{}v2/register-participation-keys/{}",
                self.url,
                address.to_string()
            ))
            .header(AUTH_HEADER, &self.token)
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
            .header(AUTH_HEADER, &self.token)
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

    pub async fn status_after_round(&self, round: Round) -> Result<NodeStatus, ClientError> {
        let response = self
            .http_client
            .get(&format!(
                "{}v2/status/wait-for-block-after/{}",
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

    pub async fn compile_teal(&self, teal: String) -> Result<ApiCompiledTealWithHash, ClientError> {
        let response = self
            .http_client
            .post(&format!("{}v2/teal/compile", self.url))
            .headers(self.headers.clone())
            .header(AUTH_HEADER, &self.token)
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

    pub async fn dryrun_teal(&self, req: &DryrunRequest) -> Result<DryrunResponse, ClientError> {
        let response = self
            .http_client
            .post(&format!("{}v2/teal/dryrun", self.url))
            .headers(self.headers.clone())
            .header(AUTH_HEADER, &self.token)
            .header("Content-Type", "application/json")
            .json(req)
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
            .header(AUTH_HEADER, &self.token)
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

    pub async fn pending_transactions(&self, max: u64) -> Result<PendingTransactions, ClientError> {
        let response = self
            .http_client
            .get(&format!("{}v2/transactions/pending", self.url))
            .header(AUTH_HEADER, &self.token)
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
}
