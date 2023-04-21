use crate::error::ServiceError;
use algonaut_algod::{
    apis::configuration::{ApiKey, Configuration},
    models::{
        self, Account, AccountApplicationInformation200Response, Application, Asset, DryrunRequest,
        GetApplicationBoxes200Response, GetBlock200Response, GetBlockHash200Response,
        GetPendingTransactionsByAddress200Response, GetStatus200Response, GetSupply200Response,
        GetSyncRound200Response, GetTransactionProof200Response, LightBlockHeaderProof,
        PendingTransactionResponse, RawTransaction200Response, SimulateRequest,
        SimulateTransaction200Response, StateProof, TealCompile200Response,
        TealDisassemble200Response, TealDryrun200Response, TransactionParams200Response, Version,
    },
};
use algonaut_core::ToMsgPack;
use algonaut_transaction::SignedTransaction;

use self::error::AlgodError;

/// Error class wrapping errors from algonaut_algod
pub(crate) mod error;

#[derive(Debug, Clone)]
pub struct Algod {
    pub(crate) configuration: Configuration,
}

impl Algod {
    /// Build a v2 client for Algorand protocol daemon.
    ///
    /// For third party providers / custom headers, use [with_headers](Self::with_headers).
    ///
    /// Returns an error if the url or token have an invalid format.
    pub fn new(url: &str, token: &str) -> Result<Algod, ServiceError> {
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
            configuration: conf,
        })
    }

    /// Given a specific account public key and application ID, this call returns the account's application local state and global state (AppLocalState and AppParams, if either exists). Global state will only be returned if the provided address is the application's creator.
    pub async fn account_application_information(
        self,
        address: &str,
        application_id: u64,
    ) -> Result<AccountApplicationInformation200Response, ServiceError> {
        Ok(
            algonaut_algod::apis::public_api::account_application_information(
                &self.configuration,
                address,
                application_id,
                None,
            )
            .await
            .map_err(|e| Into::<AlgodError>::into(e))?,
        )
    }

    /// Given a specific account public key, this call returns the accounts status, balance and spendable amounts
    pub async fn account_information(&self, address: &str) -> Result<Account, ServiceError> {
        Ok(algonaut_algod::apis::public_api::account_information(
            &self.configuration,
            address,
            None,
            None,
        )
        .await
        .map_err(|e| Into::<AlgodError>::into(e))?)
    }

    /// Returns wether the experimental API are enabled
    pub async fn experimental_check(&self) -> Result<(), ServiceError> {
        Ok(
            algonaut_algod::apis::public_api::experimental_check(&self.configuration)
                .await
                .map_err(|e| Into::<AlgodError>::into(e))?,
        )
    }

    /// Given an application ID and box name, it returns the box name and value (each base64 encoded). Box names must be in the goal app call arg encoding form 'encoding:value'. For ints, use the form 'int:1234'. For raw bytes, use the form 'b64:A=='. For printable strings, use the form 'str:hello'. For addresses, use the form 'addr:XYZ...'.
    pub async fn get_application_box_by_name(
        &self,
        application_id: u64,
        name: &str,
    ) -> Result<models::Box, ServiceError> {
        Ok(
            algonaut_algod::apis::public_api::get_application_box_by_name(
                &self.configuration,
                application_id,
                name,
            )
            .await
            .map_err(|e| Into::<AlgodError>::into(e))?,
        )
    }

    /// Given an application ID, return all Box names. No particular ordering is guaranteed. Request fails when client or server-side configured limits prevent returning all Box names.
    pub async fn get_application_boxes(
        &self,
        application_id: u64,
        max: Option<u64>,
    ) -> Result<GetApplicationBoxes200Response, ServiceError> {
        Ok(algonaut_algod::apis::public_api::get_application_boxes(
            &self.configuration,
            application_id,
            max,
        )
        .await
        .map_err(|e| Into::<AlgodError>::into(e))?)
    }

    /// Given a application ID, it returns application information including creator, approval and clear programs, global and local schemas, and global state.
    pub async fn get_application_by_id(
        &self,
        application_id: u64,
    ) -> Result<Application, ServiceError> {
        Ok(algonaut_algod::apis::public_api::get_application_by_id(
            &self.configuration,
            application_id,
        )
        .await
        .map_err(|e| Into::<AlgodError>::into(e))?)
    }

    /// Given a asset ID, it returns asset information including creator, name, total supply and special addresses.
    pub async fn get_asset_by_id(&self, asset_id: u64) -> Result<Asset, ServiceError> {
        Ok(
            algonaut_algod::apis::public_api::get_asset_by_id(&self.configuration, asset_id)
                .await
                .map_err(|e| Into::<AlgodError>::into(e))?,
        )
    }

    /// Get the block for the given round.
    pub async fn get_block(&self, round: u64) -> Result<GetBlock200Response, ServiceError> {
        Ok(
            algonaut_algod::apis::public_api::get_block(&self.configuration, round, None)
                .await
                .map_err(|e| Into::<AlgodError>::into(e))?,
        )
    }

    /// Get the block hash for the block on the given round.
    pub async fn get_block_hash(
        &self,
        round: u64,
    ) -> Result<GetBlockHash200Response, ServiceError> {
        Ok(
            algonaut_algod::apis::public_api::get_block_hash(&self.configuration, round)
                .await
                .map_err(|e| Into::<AlgodError>::into(e))?,
        )
    }

    /// Returns the entire genesis file in json.
    pub async fn get_genesis(&self) -> Result<String, ServiceError> {
        Ok(
            algonaut_algod::apis::public_api::get_genesis(&self.configuration)
                .await
                .map_err(|e| Into::<AlgodError>::into(e))?,
        )
    }

    /// Get ledger deltas for a round.
    pub async fn get_ledger_state_delta(
        &self,
        round: u64,
    ) -> Result<serde_json::Value, ServiceError> {
        Ok(algonaut_algod::apis::public_api::get_ledger_state_delta(
            &self.configuration,
            round,
            None,
        )
        .await
        .map_err(|e| Into::<AlgodError>::into(e))?)
    }

    /// Gets a proof for a given light block header inside a state proof commitment.
    pub async fn get_light_block_header_proof(
        &self,
        round: u64,
    ) -> Result<LightBlockHeaderProof, ServiceError> {
        Ok(
            algonaut_algod::apis::public_api::get_light_block_header_proof(
                &self.configuration,
                round,
            )
            .await
            .map_err(|e| Into::<AlgodError>::into(e))?,
        )
    }

    /// Get the list of pending transactions, sorted by priority, in decreasing order, truncated at the end at MAX. If MAX = 0, returns all pending transactions.
    pub async fn get_pending_transactions(
        &self,
        max: Option<u64>,
    ) -> Result<GetPendingTransactionsByAddress200Response, ServiceError> {
        Ok(algonaut_algod::apis::public_api::get_pending_transactions(
            &self.configuration,
            max,
            None,
        )
        .await
        .map_err(|e| Into::<AlgodError>::into(e))?)
    }

    /// Get the list of pending transactions by address, sorted by priority, in decreasing order, truncated at the end at MAX. If MAX = 0, returns all pending transactions.
    pub async fn get_pending_transactions_by_address(
        &self,
        address: &str,
        max: Option<u64>,
    ) -> Result<GetPendingTransactionsByAddress200Response, ServiceError> {
        Ok(
            algonaut_algod::apis::public_api::get_pending_transactions_by_address(
                &self.configuration,
                address,
                max,
                None,
            )
            .await
            .map_err(|e| Into::<AlgodError>::into(e))?,
        )
    }

    /// TODO
    pub async fn get_ready(&self) -> Result<(), ServiceError> {
        Ok(
            algonaut_algod::apis::public_api::get_ready(&self.configuration)
                .await
                .map_err(|e| Into::<AlgodError>::into(e))?,
        )
    }

    /// Get a state proof that covers a given round.
    pub async fn get_state_proof(&self, round: u64) -> Result<StateProof, ServiceError> {
        Ok(
            algonaut_algod::apis::public_api::get_state_proof(&self.configuration, round)
                .await
                .map_err(|e| Into::<AlgodError>::into(e))?,
        )
    }

    /// Gets the current node status.
    pub async fn get_status(&self) -> Result<GetStatus200Response, ServiceError> {
        Ok(
            algonaut_algod::apis::public_api::get_status(&self.configuration)
                .await
                .map_err(|e| Into::<AlgodError>::into(e))?,
        )
    }

    /// Get the current supply reported by the ledger.
    pub async fn get_supply(&self) -> Result<GetSupply200Response, ServiceError> {
        Ok(
            algonaut_algod::apis::public_api::get_supply(&self.configuration)
                .await
                .map_err(|e| Into::<AlgodError>::into(e))?,
        )
    }

    /// Gets the minimum sync round for the ledger.
    pub async fn get_sync_round(&self) -> Result<GetSyncRound200Response, ServiceError> {
        Ok(
            algonaut_algod::apis::public_api::get_sync_round(&self.configuration)
                .await
                .map_err(|e| Into::<AlgodError>::into(e))?,
        )
    }

    /// Get a proof for a transaction in a block.
    pub async fn get_transaction_proof(
        &self,
        round: u64,
        txid: &str,
    ) -> Result<GetTransactionProof200Response, ServiceError> {
        Ok(algonaut_algod::apis::public_api::get_transaction_proof(
            &self.configuration,
            round,
            txid,
            None,
            None,
        )
        .await
        .map_err(|e| Into::<AlgodError>::into(e))?)
    }

    /// Retrieves the supported API versions, binary build versions, and genesis information.
    pub async fn get_version(&self) -> Result<Version, ServiceError> {
        Ok(
            algonaut_algod::apis::public_api::get_version(&self.configuration)
                .await
                .map_err(|e| Into::<AlgodError>::into(e))?,
        )
    }

    /// Returns Ok if healthy
    pub async fn health_check(&self) -> Result<(), ServiceError> {
        Ok(
            algonaut_algod::apis::public_api::health_check(&self.configuration)
                .await
                .map_err(|e| Into::<AlgodError>::into(e))?,
        )
    }

    /// Return metrics about algod functioning.
    pub async fn metrics(&self) -> Result<(), ServiceError> {
        Ok(
            algonaut_algod::apis::public_api::metrics(&self.configuration)
                .await
                .map_err(|e| Into::<AlgodError>::into(e))?,
        )
    }

    /// Given a transaction ID of a recently submitted transaction, it returns information about it.  There are several cases when this might succeed: - transaction committed (committed round > 0) - transaction still in the pool (committed round = 0, pool error = \"\") - transaction removed from pool due to error (committed round = 0, pool error != \"\") Or the transaction may have happened sufficiently long ago that the node no longer remembers it, and this will return an error.
    pub async fn pending_transaction_information(
        &self,
        txid: &str,
    ) -> Result<PendingTransactionResponse, ServiceError> {
        Ok(
            algonaut_algod::apis::public_api::pending_transaction_information(
                &self.configuration,
                txid,
                None,
            )
            .await
            .map_err(|e| Into::<AlgodError>::into(e))?,
        )
    }

    /// Broadcasts a raw transaction or transaction group to the network.
    pub async fn raw_transaction(
        &self,
        rawtxn: &[u8],
    ) -> Result<RawTransaction200Response, ServiceError> {
        Ok(
            algonaut_algod::apis::public_api::raw_transaction(&self.configuration, rawtxn)
                .await
                .map_err(|e| Into::<AlgodError>::into(e))?,
        )
    }

    /// Broadcasts a transaction to the network.
    pub async fn signed_transaction(
        &self,
        txn: &SignedTransaction,
    ) -> Result<RawTransaction200Response, ServiceError> {
        self.raw_transaction(&txn.to_msg_pack()?).await
    }

    /// Broadcasts a transaction group to the network.
    ///
    /// Atomic if the transactions share a [group](algonaut_transaction::transaction::Transaction::group)
    pub async fn signed_transactions(
        &self,
        txns: &[SignedTransaction],
    ) -> Result<RawTransaction200Response, ServiceError> {
        let mut bytes = vec![];
        for t in txns {
            bytes.push(t.to_msg_pack()?);
        }
        self.raw_transaction(&bytes.concat()).await
    }

    /// Sets the minimum sync round on the ledger.
    pub async fn set_sync_round(&self, round: u64) -> Result<(), ServiceError> {
        Ok(
            algonaut_algod::apis::public_api::set_sync_round(&self.configuration, round)
                .await
                .map_err(|e| Into::<AlgodError>::into(e))?,
        )
    }

    /// Simulates a raw transaction or transaction group as it would be evaluated on the network. WARNING: This endpoint is experimental and under active development. There are no guarantees in terms of functionality or future support.
    pub async fn simulate_transaction(
        &self,
        request: SimulateRequest,
    ) -> Result<SimulateTransaction200Response, ServiceError> {
        Ok(algonaut_algod::apis::public_api::simulate_transaction(
            &self.configuration,
            request,
            None,
        )
        .await
        .map_err(|e| Into::<AlgodError>::into(e))?)
    }

    /// Returns the entire swagger spec in json.
    pub async fn swagger_json(&self) -> Result<String, ServiceError> {
        Ok(
            algonaut_algod::apis::public_api::swagger_json(&self.configuration)
                .await
                .map_err(|e| Into::<AlgodError>::into(e))?,
        )
    }

    /// Given TEAL source code in plain text, return base64 encoded program bytes and base32 SHA512_256 hash of program bytes (Address style). This endpoint is only enabled when a node's configuration file sets EnableDeveloperAPI to true.
    pub async fn teal_compile(
        &self,
        source: &str,
        sourcemap: Option<bool>,
    ) -> Result<TealCompile200Response, ServiceError> {
        Ok(
            algonaut_algod::apis::public_api::teal_compile(&self.configuration, source, sourcemap)
                .await
                .map_err(|e| Into::<AlgodError>::into(e))?,
        )
    }

    /// Given the program bytes, return the TEAL source code in plain text. This endpoint is only enabled when a node's configuration file sets EnableDeveloperAPI to true.
    pub async fn teal_disassemble(
        &self,
        source: String,
    ) -> Result<TealDisassemble200Response, ServiceError> {
        Ok(
            algonaut_algod::apis::public_api::teal_disassemble(&self.configuration, source)
                .await
                .map_err(|e| Into::<AlgodError>::into(e))?,
        )
    }

    /// Executes TEAL program(s) in context and returns debugging information about the execution. This endpoint is only enabled when a node's configuration file sets EnableDeveloperAPI to true.
    pub async fn teal_dryrun(
        &self,
        request: Option<DryrunRequest>,
    ) -> Result<TealDryrun200Response, ServiceError> {
        Ok(
            algonaut_algod::apis::public_api::teal_dryrun(&self.configuration, request)
                .await
                .map_err(|e| Into::<AlgodError>::into(e))?,
        )
    }

    /// Get parameters for constructing a new transaction.
    pub async fn transaction_params(&self) -> Result<TransactionParams200Response, ServiceError> {
        Ok(
            algonaut_algod::apis::public_api::transaction_params(&self.configuration)
                .await
                .map_err(|e| Into::<AlgodError>::into(e))?,
        )
    }

    /// Unset the ledger sync round.
    pub async fn unset_sync_round(&self) -> Result<(), ServiceError> {
        Ok(
            algonaut_algod::apis::public_api::unset_sync_round(&self.configuration)
                .await
                .map_err(|e| Into::<AlgodError>::into(e))?,
        )
    }
    /// Waits for a block to appear after round {round} and returns the node's status at the time.
    pub async fn wait_for_block(&self, round: u64) -> Result<GetStatus200Response, ServiceError> {
        Ok(
            algonaut_algod::apis::public_api::wait_for_block(&self.configuration, round)
                .await
                .map_err(|e| Into::<AlgodError>::into(e))?,
        )
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
