use crate::error::AlgorandError;
use algonaut_core::Round;
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
    pub fn pending_transactions_for(
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

    /// Starts a catchpoint catchup.
    pub fn start_catchup(&self, catchpoint: &str) -> Result<Catchup, AlgorandError> {
        let response = self
            .http_client
            .post(&format!("{}v2/catchup/{}", self.url, catchpoint))
            .headers(self.headers.clone())
            .header(AUTH_HEADER, &self.token)
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }

    /// Aborts a catchpoint catchup.
    pub fn abort_catchup(&self, catchpoint: &str) -> Result<Catchup, AlgorandError> {
        let response = self
            .http_client
            .delete(&format!("{}v2/catchup/{}", self.url, catchpoint))
            .headers(self.headers.clone())
            .header(AUTH_HEADER, &self.token)
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }

    /// Get the current supply reported by the ledger.
    pub fn ledger_supply(&self) -> Result<Supply, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v2/ledger/supply", self.url))
            .headers(self.headers.clone())
            .header(AUTH_HEADER, &self.token)
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }

    /// Generate (or renew) and register participation keys on the node for a given account address.
    ///
    /// address: The account-id to update, or all to update all accounts.
    /// fee: The fee to use when submitting key registration transactions. Defaults to the suggested
    /// fee. (default = 1000)
    /// key-dilution: value to use for two-level participation key.
    /// no-wait: Don't wait for transaction to commit before returning response.
    /// round-last-valid: The last round for which the generated participation keys will be valid.
    pub fn register_participation_keys(
        &self,
        address: &str,
        fee: Option<usize>,
        key_dilution: Option<usize>,
        no_wait: Option<bool>,
        round_last_valid: Option<String>,
    ) -> Result<String, AlgorandError> {
        let mut query = Vec::new();
        if let Some(fee) = fee {
            query.push(("fee", fee.to_string()))
        }
        if let Some(key_dilution) = key_dilution {
            query.push(("key-dilution", key_dilution.to_string()))
        }
        if let Some(no_wait) = no_wait {
            query.push(("no-wait", no_wait.to_string()))
        }
        if let Some(round_last_valid) = round_last_valid {
            query.push(("round-last-valid", round_last_valid))
        }
        let response = self
            .http_client
            .post(&format!(
                "{}v2/register-participation-keys/{}",
                self.url, address
            ))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .query(&query)
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }

    /// Special management endpoint to shutdown the node. Optionally provide a timeout parameter
    /// to indicate that the node should begin shutting down after a number of seconds.
    pub fn shutdown(&self, timeout: usize) -> Result<(), AlgorandError> {
        self.http_client
            .post(&format!("{}v2/shutdown", self.url))
            .headers(self.headers.clone())
            .header(AUTH_HEADER, &self.token)
            .query(&[("timeout", timeout.to_string())])
            .send()?
            .error_for_status()?
            .json()?;

        Ok(())
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

    /// Gets the node status after waiting for the given round.
    pub fn status_after_round(&self, round: Round) -> Result<NodeStatus, AlgorandError> {
        let response = self
            .http_client
            .get(&format!(
                "{}v2/status/wait-for-block-after/{}",
                self.url, round.0
            ))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }

    /// Compile TEAL source code to binary, produce its hash.
    ///
    /// Given TEAL source code in plain text, return base64 encoded program bytes and base32
    /// SHA512_256 hash of program bytes (Address style). This endpoint is only enabled when
    /// a node's configuration file sets EnableDeveloperAPI to true.
    pub fn compile_teal(&self, teal: String) -> Result<CompiledTeal, AlgorandError> {
        let response = self
            .http_client
            .post(&format!("{}v2/teal/compile", self.url))
            .headers(self.headers.clone())
            .header(AUTH_HEADER, &self.token)
            .header("Content-Type", "application/x-binary")
            .body(teal)
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }

    /// Provide debugging information for a transaction (or group).
    ///
    /// Executes TEAL program(s) in context and returns debugging information about the execution.
    /// This endpoint is only enabled when a node's configureation file sets EnableDeveloperAPI
    /// to true.
    pub fn dryrun_teal(&self, req: &DryrunRequest) -> Result<DryrunResponse, AlgorandError> {
        let response = self
            .http_client
            .post(&format!("{}v2/teal/dryrun", self.url))
            .headers(self.headers.clone())
            .header(AUTH_HEADER, &self.token)
            .header("Content-Type", "application/json")
            .json(req)
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }

    /// Broadcasts a raw transaction to the network.
    pub fn broadcast_raw_transaction(&self, rawtxn: String) -> Result<String, AlgorandError> {
        let response = self
            .http_client
            .post(&format!("{}v2/transactions", self.url))
            .headers(self.headers.clone())
            .header(AUTH_HEADER, &self.token)
            .header("Content-Type", "application/x-binary")
            .body(rawtxn)
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }

    /// Get parameters for constructing a new transaction.
    pub fn transaction_params(&self) -> Result<TransactionParams, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v2/transactions/params", self.url))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }

    /// Get a list of unconfirmed transactions currently in the transaction pool.
    ///
    /// Get the list of pending transactions, sorted by priority, in decreasing order,
    /// truncated at the end at MAX. If MAX = 0, returns all pending transactions.
    pub fn pending_transactions(&self, max: u64) -> Result<PendingTransactions, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v2/transactions/pending", self.url))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .query(&[("max", max.to_string())])
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response)
    }

    /// Get a specific pending transaction.
    ///
    /// Given a transaction id of a recently submitted transaction, it returns information about
    /// it. There are several cases when this might succeed:
    /// - transaction committed (committed round > 0)
    /// - transaction still in the pool (committed round = 0, pool error = "")
    /// - transaction removed from pool due to error (committed round = 0, pool error != "")
    ///
    /// Or the transaction may have happened sufficiently long ago that the node no longer remembers
    /// it, and this will return an error.
    pub fn pending_transaction_with_id(
        &self,
        txid: &str,
    ) -> Result<PendingTransaction, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v2/transactions/pending/{}", self.url, txid))
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
