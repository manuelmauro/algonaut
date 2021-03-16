use crate::error::AlgorandError;
use message::*;
use reqwest::header::HeaderMap;

/// API message structs for Algorand's daemon v2
pub mod message;

const AUTH_HEADER: &str = "X-Algo-API-Token";

/// Client for interacting with the Algorand protocol daemon
pub struct Client {
    pub(super) url: String,
    pub(super) token: String,
    pub(super) headers: HeaderMap,
    pub(super) http_client: reqwest::blocking::Client,
}

impl Client {
    /// Returns the entire genesis file in json.
    pub fn genesis(&self) -> Result<GenesisBlock, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}genesis", self.url))
            .headers(self.headers.clone())
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }

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

    /// Return metrics about algod functioning.
    pub fn metrics(&self) -> Result<String, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}metrics", self.url))
            .headers(self.headers.clone())
            .header(AUTH_HEADER, &self.token)
            .send()?
            .error_for_status()?
            .text()?;

        Ok(response)
    }

    /// Get account information.
    /// Description Given a specific account public key, this call returns the accounts status,
    /// balance and spendable amounts
    pub fn account_information(&self, address: &str) -> Result<Account, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v2/accounts/{}", self.url, address))
            .headers(self.headers.clone())
            .header(AUTH_HEADER, &self.token)
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }

    /// Get a list of unconfirmed transactions currently in the transaction pool by address.
    /// Description: Get the list of pending transactions by address, sorted by priority,
    /// in decreasing order, truncated at the end at MAX. If MAX = 0, returns all pending transactions.
    pub fn pending_transactions(
        &self,
        address: &str,
        max: u64,
    ) -> Result<PendingTransactions, AlgorandError> {
        let response = self
            .http_client
            .get(&format!(
                "{}v2/accounts/{}/transactions/pending",
                self.url, address,
            ))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .query(&[("max", max.to_string())])
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Get application information.
    ///
    /// Given a application id, it returns application information including creator,
    /// approval and clear programs, global and local schemas, and global state.
    pub fn application_information(&self, id: usize) -> Result<Application, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v2/applications/{}", self.url, id))
            .headers(self.headers.clone())
            .header(AUTH_HEADER, &self.token)
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }

    /// Get asset information.
    ///
    /// Given a asset id, it returns asset information including creator, name,
    /// total supply and special addresses.
    pub fn asset_information(&self, id: usize) -> Result<Application, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v2/asset/{}", self.url, id))
            .headers(self.headers.clone())
            .header(AUTH_HEADER, &self.token)
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }

    /// Get the block for the given round.
    pub fn block(&self, round: usize) -> Result<Block, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v2/blocks/{}", self.url, round))
            .headers(self.headers.clone())
            .header(AUTH_HEADER, &self.token)
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }

    /// Gets the current node status.
    pub fn status(&self) -> Result<NodeStatus, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v2/status", self.url))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }

    /// Retrieves the current version
    pub fn versions(&self) -> Result<Version, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}versions", self.url))
            .headers(self.headers.clone())
            .header(AUTH_HEADER, &self.token)
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }
}
