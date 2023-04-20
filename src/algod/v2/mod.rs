use crate::error::ServiceError;
use algonaut_algod::apis::configuration::{ApiKey, Configuration};
use algonaut_client::{algod::v2::Client, token::ApiToken, Headers};
use algonaut_core::{Address, CompiledTeal, Round, SuggestedTransactionParams, ToMsgPack};
use algonaut_encoding::decode_base64;
use algonaut_model::algod::v2::{
    Account, Application, Asset, Block, BlockWithCertificate, Catchup, DryrunRequest,
    DryrunResponse, GenesisBlock, KeyRegistration, NodeStatus, PendingTransaction,
    PendingTransactions, Supply, TransactionParams, TransactionResponse, Version,
};
use algonaut_transaction::SignedTransaction;

use self::error::AlgodError;

/// Error class wrapping errors from algonaut_algod
pub(crate) mod error;

#[derive(Debug, Clone)]
pub struct Algod {
    pub(crate) client: Client,
    pub(crate) configuration: Configuration,
}

impl Algod {
    /// Build a v2 client for Algorand protocol daemon.
    ///
    /// For third party providers / custom headers, use [with_headers](Self::with_headers).
    ///
    /// Returns an error if the url or token have an invalid format.
    pub fn new(url: &str, token: &str) -> Result<Algod, ServiceError> {
        Self::with_headers(
            url,
            token,
            vec![("X-Algo-API-Token", &ApiToken::parse(token)?.to_string())],
        )
    }

    /// Build a v2 client for Algorand protocol daemon.
    ///
    /// Use this initializer when interfacing with third party services, that require custom headers.
    ///
    /// Returns an error if the url or headers have an invalid format.
    pub fn with_headers(url: &str, token: &str, headers: Headers) -> Result<Algod, ServiceError> {
        let conf = Configuration {
            base_path: url.to_owned(),
            user_agent: Some("OpenAPI-Generator/0.0.1/rust".to_owned()),
            client: reqwest::Client::new(),
            basic_auth: None,
            oauth_access_token: None,
            bearer_access_token: None,
            api_key: Some(ApiKey {
                prefix: None,
                key: token.to_owned(),
            }),
        };

        Ok(Algod {
            client: Client::new(url, headers)?,
            configuration: conf,
        })
    }

    /// Returns the entire genesis file in json.
    pub async fn genesis(&self) -> Result<GenesisBlock, ServiceError> {
        Ok(self.client.genesis().await?)
    }

    /// Returns Ok if healthy
    pub async fn health(&self) -> Result<(), ServiceError> {
        Ok(
            algonaut_algod::apis::common_api::health_check(&self.configuration)
                .await
                .map_err(|e| Into::<AlgodError>::into(e))?,
        )
    }

    /// Return metrics about algod functioning.
    pub async fn metrics(&self) -> Result<String, ServiceError> {
        Ok(self.client.metrics().await?)
    }

    /// Get account information.
    /// Description Given a specific account public key, this call returns the accounts status,
    /// balance and spendable amounts
    pub async fn account_information(&self, address: &Address) -> Result<Account, ServiceError> {
        Ok(self
            .client
            .account_information(&address.to_string())
            .await?)
    }

    /// Get a list of unconfirmed transactions currently in the transaction pool by address.
    /// Description: Get the list of pending transactions by address, sorted by priority,
    /// in decreasing order, truncated at the end at MAX. If MAX = 0, returns all pending transactions.
    pub async fn pending_transactions_for(
        &self,
        address: &Address,
        max: u64,
    ) -> Result<PendingTransactions, ServiceError> {
        Ok(self
            .client
            .pending_transactions_for(&address.to_string(), max)
            .await?)
    }

    /// Get application information.
    ///
    /// Given a application id, it returns application information including creator,
    /// approval and clear programs, global and local schemas, and global state.
    pub async fn application_information(&self, id: u64) -> Result<Application, ServiceError> {
        Ok(self.client.application_information(id).await?)
    }

    /// Get asset information.
    ///
    /// Given a asset id, it returns asset information including creator, name,
    /// total supply and special addresses.
    pub async fn asset_information(&self, id: u64) -> Result<Asset, ServiceError> {
        Ok(self.client.asset_information(id).await?)
    }

    /// Get the block for the given round.
    pub async fn block(&self, round: Round) -> Result<Block, ServiceError> {
        Ok(self.client.block(round).await?)
    }

    pub async fn block_with_certificate(
        &self,
        round: Round,
    ) -> Result<BlockWithCertificate, ServiceError> {
        Ok(self.client.block_with_certificate(round).await?)
    }

    /// Starts a catchpoint catchup.
    pub async fn start_catchup(&self, catchpoint: &str) -> Result<Catchup, ServiceError> {
        Ok(self.client.start_catchup(catchpoint).await?)
    }

    /// Aborts a catchpoint catchup.
    pub async fn abort_catchup(&self, catchpoint: &str) -> Result<Catchup, ServiceError> {
        Ok(self.client.abort_catchup(catchpoint).await?)
    }

    /// Get the current supply reported by the ledger.
    pub async fn ledger_supply(&self) -> Result<Supply, ServiceError> {
        Ok(self.client.ledger_supply().await?)
    }

    /// Generate (or renew) and register participation keys on the node for a given account address.
    ///
    /// address: The account-id to update, or all to update all accounts.
    /// fee: The fee to use when submitting key registration transactions. Defaults to the suggested
    /// fee. (default = 1000)
    /// key-dilution: value to use for two-level participation key.
    /// no-wait: Don't wait for transaction to commit before returning response.
    /// round-last-valid: The last round for which the generated participation keys will be valid.
    pub async fn register_participation_keys(
        &self,
        address: &Address,
        params: &KeyRegistration,
    ) -> Result<String, ServiceError> {
        Ok(self
            .client
            .register_participation_keys(address, params)
            .await?)
    }

    /// Special management endpoint to shutdown the node. Optionally provide a timeout parameter
    /// to indicate that the node should begin shutting down after a number of seconds.
    pub async fn shutdown(&self, timeout: usize) -> Result<(), ServiceError> {
        Ok(self.client.shutdown(timeout).await?)
    }

    /// Gets the current node status.
    pub async fn status(&self) -> Result<NodeStatus, ServiceError> {
        Ok(self.client.status().await?)
    }

    /// Gets the node status after waiting for the given round.
    pub async fn status_after_round(&self, round: Round) -> Result<NodeStatus, ServiceError> {
        Ok(self.client.status_after_round(round).await?)
    }

    /// Compile TEAL source code to binary, produce its hash.
    ///
    /// Given TEAL source code in plain text, return compiled program bytes.
    /// This endpoint is only enabled when a node's configuration file sets EnableDeveloperAPI to true.
    pub async fn compile_teal(&self, teal: &[u8]) -> Result<CompiledTeal, ServiceError> {
        let api_compiled_teal = self.client.compile_teal(teal.to_vec()).await?;
        // The api result (program + hash) is mapped to the domain program struct, which computes the hash on demand.
        // The hash here is redundant and we want to allow to generate it with the SDK too (e.g. for when loading programs from a DB).
        // At the moment it seems not warranted to add a cache (so it's initialized with the API hash or lazily), but this can be re-evaluated.
        // Note that for contract accounts, there's [ContractAccount](algonaut_transaction::account::ContractAccount), which caches it (as address).
        Ok(CompiledTeal(decode_base64(
            api_compiled_teal.result.as_bytes(),
        )?))
    }

    /// Provide debugging information for a transaction (or group).
    ///
    /// Executes TEAL program(s) in context and returns debugging information about the execution.
    /// This endpoint is only enabled when a node's configureation file sets EnableDeveloperAPI
    /// to true.
    pub async fn dryrun_teal(&self, req: &DryrunRequest) -> Result<DryrunResponse, ServiceError> {
        let bytes = req.to_msg_pack()?;
        Ok(self.client.dryrun_teal(bytes).await?)
    }

    /// Broadcasts a transaction to the network.
    pub async fn broadcast_signed_transaction(
        &self,
        txn: &SignedTransaction,
    ) -> Result<TransactionResponse, ServiceError> {
        self.broadcast_raw_transaction(&txn.to_msg_pack()?).await
    }

    /// Broadcasts a transaction group to the network.
    ///
    /// Atomic if the transactions share a [group](algonaut_transaction::transaction::Transaction::group)
    pub async fn broadcast_signed_transactions(
        &self,
        txns: &[SignedTransaction],
    ) -> Result<TransactionResponse, ServiceError> {
        let mut bytes = vec![];
        for t in txns {
            bytes.push(t.to_msg_pack()?);
        }
        self.broadcast_raw_transaction(&bytes.concat()).await
    }

    /// Broadcasts raw transactions to the network.
    ///
    /// When passing multiple transactions, the transactions are atomic if they share a [group](algonaut_transaction::transaction::Transaction::group)
    ///
    /// Use this when using a third party (e.g. KMD) that delivers directly the serialized signed transaction.
    ///
    /// Otherwise, prefer [broadcast_signed_transaction](Self::broadcast_signed_transaction) or [broadcast_signed_transactions][Self::broadcast_signed_transactions]

    pub async fn broadcast_raw_transaction(
        &self,
        rawtxn: &[u8],
    ) -> Result<TransactionResponse, ServiceError> {
        Ok(self.client.broadcast_raw_transaction(rawtxn).await?)
    }

    /// Get parameters for constructing a new transaction.
    pub async fn transaction_params(&self) -> Result<TransactionParams, ServiceError> {
        Ok(self.client.transaction_params().await?)
    }

    /// Get suggested parameters for constructing a new transaction.
    pub async fn suggested_transaction_params(
        &self,
    ) -> Result<SuggestedTransactionParams, ServiceError> {
        let params = self.client.transaction_params().await?;
        Ok(SuggestedTransactionParams {
            genesis_id: params.genesis_id,
            genesis_hash: params.genesis_hash,
            consensus_version: params.consensus_version,
            fee_per_byte: params.fee_per_byte,
            min_fee: params.min_fee,
            first_valid: params.last_round,
            last_valid: params.last_round + 1000,
        })
    }

    /// Get a list of unconfirmed transactions currently in the transaction pool.
    ///
    /// Get the list of pending transactions, sorted by priority, in decreasing order,
    /// truncated at the end at MAX. If MAX = 0, returns all pending transactions.
    pub async fn pending_transactions(
        &self,
        max: u64,
    ) -> Result<PendingTransactions, ServiceError> {
        Ok(self.client.pending_transactions(max).await?)
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
    pub async fn pending_transaction_with_id(
        &self,
        txid: &str,
    ) -> Result<PendingTransaction, ServiceError> {
        Ok(self.client.pending_transaction_with_id(txid).await?)
    }

    /// Retrieves the current version
    pub async fn versions(&self) -> Result<Version, ServiceError> {
        Ok(self.client.versions().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_client_builder() {
        let res = Algod::new(
            "http://example.com",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        assert!(res.ok().is_some());
    }

    #[test]
    fn test_client_builder_with_invalid_url() {
        let res = Algod::new(
            "asfdsdfs",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        assert!(res.is_err());
        assert!(matches!(res.err().unwrap(), ServiceError::BadUrl(_)));
    }

    #[test]
    fn test_client_builder_with_invalid_url_no_scheme() {
        let res = Algod::new(
            "example.com",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        assert!(res.is_err());
        assert!(matches!(res.err().unwrap(), ServiceError::BadUrl(_)));
    }

    #[test]
    fn test_client_builder_with_invalid_token() {
        let res = Algod::new(
            "http://example.com",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        assert!(res.is_err());
        assert!(res.err().unwrap() == ServiceError::BadToken);
    }

    #[test]
    fn test_client_builder_with_empty_token() {
        let res = Algod::new("http://example.com", "");
        assert!(res.is_err());
        assert!(res.err().unwrap() == ServiceError::BadToken);
    }
}
