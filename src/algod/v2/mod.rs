use algonaut_client::algod::v2::message::Account;
use algonaut_client::algod::v2::message::ApiCompiledTeal;
use algonaut_client::algod::v2::message::Application;
use algonaut_client::algod::v2::message::Block;
use algonaut_client::algod::v2::message::Catchup;
use algonaut_client::algod::v2::message::DryrunRequest;
use algonaut_client::algod::v2::message::DryrunResponse;
use algonaut_client::algod::v2::message::GenesisBlock;
use algonaut_client::algod::v2::message::KeyRegistration;
use algonaut_client::algod::v2::message::NodeStatus;
use algonaut_client::algod::v2::message::PendingTransaction;
use algonaut_client::algod::v2::message::PendingTransactions;
use algonaut_client::algod::v2::message::Supply;
use algonaut_client::algod::v2::message::TransactionParams;
use algonaut_client::algod::v2::message::TransactionResponse;
use algonaut_client::algod::v2::message::Version;
use algonaut_client::algod::v2::Client;
use algonaut_client::error::AlgorandError;
use algonaut_core::Address;
use algonaut_core::Round;
use algonaut_core::ToMsgPack;
use algonaut_transaction::SignedTransaction;

pub struct Algod {
    pub(crate) client: Client,
}

impl Algod {
    pub fn new(client: Client) -> Algod {
        Algod { client }
    }

    /// Returns the entire genesis file in json.
    pub async fn genesis(&self) -> Result<GenesisBlock, AlgorandError> {
        self.client.genesis().await
    }

    /// Returns Ok if healthy
    pub async fn health(&self) -> Result<(), AlgorandError> {
        self.client.health().await
    }

    /// Return metrics about algod functioning.
    pub async fn metrics(&self) -> Result<String, AlgorandError> {
        self.client.metrics().await
    }

    /// Get account information.
    /// Description Given a specific account public key, this call returns the accounts status,
    /// balance and spendable amounts
    pub async fn account_information(&self, address: &Address) -> Result<Account, AlgorandError> {
        self.client.account_information(&address.to_string()).await
    }

    /// Get a list of unconfirmed transactions currently in the transaction pool by address.
    /// Description: Get the list of pending transactions by address, sorted by priority,
    /// in decreasing order, truncated at the end at MAX. If MAX = 0, returns all pending transactions.
    pub async fn pending_transactions_for(
        &self,
        address: &Address,
        max: u64,
    ) -> Result<PendingTransactions, AlgorandError> {
        self.client
            .pending_transactions_for(&address.to_string(), max)
            .await
    }

    /// Get application information.
    ///
    /// Given a application id, it returns application information including creator,
    /// approval and clear programs, global and local schemas, and global state.
    pub async fn application_information(&self, id: usize) -> Result<Application, AlgorandError> {
        self.client.application_information(id).await
    }

    /// Get asset information.
    ///
    /// Given a asset id, it returns asset information including creator, name,
    /// total supply and special addresses.
    pub async fn asset_information(&self, id: usize) -> Result<Application, AlgorandError> {
        self.client.asset_information(id).await
    }

    /// Get the block for the given round.
    pub async fn block(&self, round: Round) -> Result<Block, AlgorandError> {
        self.client.block(round).await
    }

    /// Starts a catchpoint catchup.
    pub async fn start_catchup(&self, catchpoint: &str) -> Result<Catchup, AlgorandError> {
        self.client.start_catchup(catchpoint).await
    }

    /// Aborts a catchpoint catchup.
    pub async fn abort_catchup(&self, catchpoint: &str) -> Result<Catchup, AlgorandError> {
        self.client.abort_catchup(catchpoint).await
    }

    /// Get the current supply reported by the ledger.
    pub async fn ledger_supply(&self) -> Result<Supply, AlgorandError> {
        self.client.ledger_supply().await
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
    ) -> Result<String, AlgorandError> {
        self.client
            .register_participation_keys(address, params)
            .await
    }

    /// Special management endpoint to shutdown the node. Optionally provide a timeout parameter
    /// to indicate that the node should begin shutting down after a number of seconds.
    pub async fn shutdown(&self, timeout: usize) -> Result<(), AlgorandError> {
        self.client.shutdown(timeout).await
    }

    /// Gets the current node status.
    pub async fn status(&self) -> Result<NodeStatus, AlgorandError> {
        self.client.status().await
    }

    /// Gets the node status after waiting for the given round.
    pub async fn status_after_round(&self, round: Round) -> Result<NodeStatus, AlgorandError> {
        self.client.status_after_round(round).await
    }

    /// Compile TEAL source code to binary, produce its hash.
    ///
    /// Given TEAL source code in plain text, return base64 encoded program bytes and base32
    /// SHA512_256 hash of program bytes (Address style). This endpoint is only enabled when
    /// a node's configuration file sets EnableDeveloperAPI to true.
    pub async fn compile_teal(&self, teal: String) -> Result<ApiCompiledTeal, AlgorandError> {
        self.client.compile_teal(teal).await
    }

    /// Provide debugging information for a transaction (or group).
    ///
    /// Executes TEAL program(s) in context and returns debugging information about the execution.
    /// This endpoint is only enabled when a node's configureation file sets EnableDeveloperAPI
    /// to true.
    pub async fn dryrun_teal(&self, req: &DryrunRequest) -> Result<DryrunResponse, AlgorandError> {
        self.client.dryrun_teal(req).await
    }

    /// Broadcasts a transaction to the network.
    pub async fn broadcast_signed_transaction(
        &self,
        txn: &SignedTransaction,
    ) -> Result<TransactionResponse, AlgorandError> {
        self.broadcast_raw_transaction(&txn.to_msg_pack()?).await
    }

    /// Broadcasts a transaction group to the network.
    pub async fn broadcast_signed_transactions(
        &self,
        txns: &[SignedTransaction],
    ) -> Result<TransactionResponse, AlgorandError> {
        let mut bytes = vec![];
        for t in txns {
            bytes.push(t.to_msg_pack()?);
        }
        self.broadcast_raw_transaction(&bytes.concat()).await
    }

    /// Broadcasts a raw transaction or transaction group to the network.
    pub async fn broadcast_raw_transaction(
        &self,
        rawtxn: &[u8],
    ) -> Result<TransactionResponse, AlgorandError> {
        self.client.broadcast_raw_transaction(rawtxn).await
    }

    /// Get parameters for constructing a new transaction.
    pub async fn transaction_params(&self) -> Result<TransactionParams, AlgorandError> {
        self.client.transaction_params().await
    }

    /// Get a list of unconfirmed transactions currently in the transaction pool.
    ///
    /// Get the list of pending transactions, sorted by priority, in decreasing order,
    /// truncated at the end at MAX. If MAX = 0, returns all pending transactions.
    pub async fn pending_transactions(
        &self,
        max: u64,
    ) -> Result<PendingTransactions, AlgorandError> {
        self.client.pending_transactions(max).await
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
    ) -> Result<PendingTransaction, AlgorandError> {
        self.client.pending_transaction_with_id(txid).await
    }

    /// Retrieves the current version
    pub async fn versions(&self) -> Result<Version, AlgorandError> {
        self.client.versions().await
    }
}
