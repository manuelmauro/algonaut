use self::error::AlgodError;
use crate::Error;
use algonaut_algod::{
    apis::configuration::{ApiKey, Configuration},
    models::{
        self, Account, AccountApplicationInformation200Response, Application, Asset, DryrunRequest,
        GetApplicationBoxes200Response, GetBlockHash200Response,
        GetPendingTransactionsByAddress200Response, GetStatus200Response, GetSupply200Response,
        GetSyncRound200Response, GetTransactionProof200Response, LightBlockHeaderProof,
        PendingTransactionResponse, RawTransaction200Response, SimulateRequest,
        SimulateTransaction200Response, StateProof, TealDisassemble200Response,
        TealDryrun200Response, TransactionParams200Response, Version,
    },
};
use algonaut_core::{CompiledTeal, ToMsgPack};
use algonaut_encoding::decode_base64;
use algonaut_model::algod::v2::block::Block;
use algonaut_transaction::SignedTransaction;

/// Error class wrapping errors from algonaut_algod
pub(crate) mod error;

#[derive(Debug, Clone)]
pub struct Algod {
    pub(crate) configuration: Configuration,
}

impl Algod {
    /// Build a v2 client for Algorand protocol daemon.
    pub fn new(url: &str, token: &str) -> Result<Self, Error> {
        let conf = Configuration {
            base_path: url.to_owned(),
            user_agent: Some("algonaut".to_owned()),
            client: reqwest::Client::new(),
            basic_auth: None,
            oauth_access_token: None,
            bearer_access_token: None,
            api_key: Some(ApiKey {
                prefix: None,
                key: token.to_owned(),
            }),
        };

        Ok(Self {
            configuration: conf,
        })
    }

    /// Given a specific account public key and application ID, this call returns the account's application local state and global state (AppLocalState and AppParams, if either exists). Global state will only be returned if the provided address is the application's creator.
    pub async fn account_app(
        self,
        address: &str,
        application_id: u64,
    ) -> Result<AccountApplicationInformation200Response, Error> {
        Ok(
            algonaut_algod::apis::public_api::account_application_information(
                &self.configuration,
                address,
                application_id,
                None,
            )
            .await
            .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Given a specific account public key, this call returns the accounts status, balance and spendable amounts
    pub async fn account(&self, address: &str) -> Result<Account, Error> {
        Ok(algonaut_algod::apis::public_api::account_information(
            &self.configuration,
            address,
            None,
            None,
        )
        .await
        .map_err(Into::<AlgodError>::into)?)
    }

    /// Returns wether the experimental API are enabled
    pub async fn experimental(&self) -> Result<(), Error> {
        Ok(
            algonaut_algod::apis::public_api::experimental_check(&self.configuration)
                .await
                .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Given an application ID and box name, it returns the box name and value (each base64 encoded). Box names must be in the goal app call arg encoding form 'encoding:value'. For ints, use the form 'int:1234'. For raw bytes, use the form 'b64:A=='. For printable strings, use the form 'str:hello'. For addresses, use the form 'addr:XYZ...'.
    pub async fn app_box(&self, application_id: u64, name: &str) -> Result<models::Box, Error> {
        Ok(
            algonaut_algod::apis::public_api::get_application_box_by_name(
                &self.configuration,
                application_id,
                name,
            )
            .await
            .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Given an application ID, return all Box names. No particular ordering is guaranteed. Request fails when client or server-side configured limits prevent returning all Box names.
    pub async fn app_boxes(
        &self,
        application_id: u64,
        max: Option<u64>,
    ) -> Result<GetApplicationBoxes200Response, Error> {
        Ok(algonaut_algod::apis::public_api::get_application_boxes(
            &self.configuration,
            application_id,
            max,
        )
        .await
        .map_err(Into::<AlgodError>::into)?)
    }

    /// Given a application ID, it returns application information including creator, approval and clear programs, global and local schemas, and global state.
    pub async fn app(&self, application_id: u64) -> Result<Application, Error> {
        Ok(algonaut_algod::apis::public_api::get_application_by_id(
            &self.configuration,
            application_id,
        )
        .await
        .map_err(Into::<AlgodError>::into)?)
    }

    /// Given a asset ID, it returns asset information including creator, name, total supply and special addresses.
    pub async fn asset(&self, asset_id: u64) -> Result<Asset, Error> {
        Ok(
            algonaut_algod::apis::public_api::get_asset_by_id(&self.configuration, asset_id)
                .await
                .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Get the block for the given round.
    pub async fn block(&self, round: u64) -> Result<Block, Error> {
        Ok(
            algonaut_algod::apis::public_api::get_block(&self.configuration, round, None)
                .await
                .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Get the block hash for the block on the given round.
    pub async fn block_hash(&self, round: u64) -> Result<GetBlockHash200Response, Error> {
        Ok(
            algonaut_algod::apis::public_api::get_block_hash(&self.configuration, round)
                .await
                .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Returns the entire genesis file in json.
    pub async fn genesis(&self) -> Result<String, Error> {
        Ok(
            algonaut_algod::apis::public_api::get_genesis(&self.configuration)
                .await
                .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Get ledger deltas for a round.
    pub async fn state_delta(&self, round: u64) -> Result<serde_json::Value, Error> {
        Ok(algonaut_algod::apis::public_api::get_ledger_state_delta(
            &self.configuration,
            round,
            None,
        )
        .await
        .map_err(Into::<AlgodError>::into)?)
    }

    /// Gets a proof for a given light block header inside a state proof commitment.
    pub async fn light_block_header_proof(
        &self,
        round: u64,
    ) -> Result<LightBlockHeaderProof, Error> {
        Ok(
            algonaut_algod::apis::public_api::get_light_block_header_proof(
                &self.configuration,
                round,
            )
            .await
            .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Get the list of pending transactions, sorted by priority, in decreasing order, truncated at the end at MAX. If MAX = 0, returns all pending transactions.
    pub async fn pending_txns(
        &self,
        max: Option<u64>,
    ) -> Result<GetPendingTransactionsByAddress200Response, Error> {
        Ok(algonaut_algod::apis::public_api::get_pending_transactions(
            &self.configuration,
            max,
            None,
        )
        .await
        .map_err(Into::<AlgodError>::into)?)
    }

    /// Get the list of pending transactions by address, sorted by priority, in decreasing order, truncated at the end at MAX. If MAX = 0, returns all pending transactions.
    pub async fn address_pending_txns(
        &self,
        address: &str,
        max: Option<u64>,
    ) -> Result<GetPendingTransactionsByAddress200Response, Error> {
        Ok(
            algonaut_algod::apis::public_api::get_pending_transactions_by_address(
                &self.configuration,
                address,
                max,
                None,
            )
            .await
            .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// TODO
    pub async fn ready(&self) -> Result<(), Error> {
        Ok(
            algonaut_algod::apis::public_api::get_ready(&self.configuration)
                .await
                .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Get a state proof that covers a given round.
    pub async fn state_proof(&self, round: u64) -> Result<StateProof, Error> {
        Ok(
            algonaut_algod::apis::public_api::get_state_proof(&self.configuration, round)
                .await
                .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Gets the current node status.
    pub async fn status(&self) -> Result<GetStatus200Response, Error> {
        Ok(
            algonaut_algod::apis::public_api::get_status(&self.configuration)
                .await
                .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Get the current supply reported by the ledger.
    pub async fn supply(&self) -> Result<GetSupply200Response, Error> {
        Ok(
            algonaut_algod::apis::public_api::get_supply(&self.configuration)
                .await
                .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Gets the minimum sync round for the ledger.
    pub async fn sync_round(&self) -> Result<GetSyncRound200Response, Error> {
        Ok(
            algonaut_algod::apis::public_api::get_sync_round(&self.configuration)
                .await
                .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Get a proof for a transaction in a block.
    pub async fn txn_proof(
        &self,
        round: u64,
        txid: &str,
    ) -> Result<GetTransactionProof200Response, Error> {
        Ok(algonaut_algod::apis::public_api::get_transaction_proof(
            &self.configuration,
            round,
            txid,
            None,
            None,
        )
        .await
        .map_err(Into::<AlgodError>::into)?)
    }

    /// Retrieves the supported API versions, binary build versions, and genesis information.
    pub async fn version(&self) -> Result<Version, Error> {
        Ok(
            algonaut_algod::apis::public_api::get_version(&self.configuration)
                .await
                .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Returns Ok if healthy
    pub async fn health(&self) -> Result<(), Error> {
        Ok(
            algonaut_algod::apis::public_api::health_check(&self.configuration)
                .await
                .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Return metrics about algod functioning.
    pub async fn metrics(&self) -> Result<(), Error> {
        Ok(
            algonaut_algod::apis::public_api::metrics(&self.configuration)
                .await
                .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Given a transaction ID of a recently submitted transaction, it returns information about it.  There are several cases when this might succeed: - transaction committed (committed round > 0) - transaction still in the pool (committed round = 0, pool error = \"\") - transaction removed from pool due to error (committed round = 0, pool error != \"\") Or the transaction may have happened sufficiently long ago that the node no longer remembers it, and this will return an error.
    pub async fn pending_txn(&self, txid: &str) -> Result<PendingTransactionResponse, Error> {
        Ok(
            algonaut_algod::apis::public_api::pending_transaction_information(
                &self.configuration,
                txid,
                None,
            )
            .await
            .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Broadcasts a raw transaction or transaction group to the network.
    pub async fn send_raw_txn(&self, rawtxn: &[u8]) -> Result<RawTransaction200Response, Error> {
        Ok(
            algonaut_algod::apis::public_api::raw_transaction(&self.configuration, rawtxn)
                .await
                .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Broadcasts a transaction to the network.
    pub async fn send_txn(
        &self,
        txn: &SignedTransaction,
    ) -> Result<RawTransaction200Response, Error> {
        self.send_raw_txn(&txn.to_msg_pack()?).await
    }

    /// Broadcasts a transaction group to the network.
    ///
    /// Atomic if the transactions share a [group](algonaut_transaction::transaction::Transaction::group)
    pub async fn send_txns(
        &self,
        txns: &[SignedTransaction],
    ) -> Result<RawTransaction200Response, Error> {
        let mut bytes = vec![];
        for t in txns {
            bytes.push(t.to_msg_pack()?);
        }
        self.send_raw_txn(&bytes.concat()).await
    }

    /// Sets the minimum sync round on the ledger.
    pub async fn sync(&self, round: u64) -> Result<(), Error> {
        Ok(
            algonaut_algod::apis::public_api::set_sync_round(&self.configuration, round)
                .await
                .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Simulates a raw transaction or transaction group as it would be evaluated on the network. WARNING: This endpoint is experimental and under active development. There are no guarantees in terms of functionality or future support.
    pub async fn simulate_txns(
        &self,
        request: SimulateRequest,
    ) -> Result<SimulateTransaction200Response, Error> {
        Ok(algonaut_algod::apis::public_api::simulate_transaction(
            &self.configuration,
            request,
            None,
        )
        .await
        .map_err(Into::<AlgodError>::into)?)
    }

    /// Returns the entire swagger spec in json.
    pub async fn swagger_json(&self) -> Result<String, Error> {
        Ok(
            algonaut_algod::apis::public_api::swagger_json(&self.configuration)
                .await
                .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Given TEAL source code in plain text, return base64 encoded program bytes and base32 SHA512_256 hash of program bytes (Address style). This endpoint is only enabled when a node's configuration file sets EnableDeveloperAPI to true.
    pub async fn teal_compile(
        &self,
        source: &[u8],
        sourcemap: Option<bool>,
    ) -> Result<CompiledTeal, Error> {
        let api_compiled_teal =
            algonaut_algod::apis::public_api::teal_compile(&self.configuration, source, sourcemap)
                .await
                .map_err(Into::<AlgodError>::into)?;
        // The api result (program + hash) is mapped to the domain program struct, which computes the hash on demand.
        // The hash here is redundant and we want to allow to generate it with the SDK too (e.g. for when loading programs from a DB).
        // At the moment it seems not warranted to add a cache (so it's initialized with the API hash or lazily), but this can be re-evaluated.
        // Note that for contract accounts, there's [ContractAccount](algonaut_transaction::account::ContractAccount), which caches it (as address).
        Ok(CompiledTeal(decode_base64(
            api_compiled_teal.result.as_bytes(),
        )?))
    }

    /// Given the program bytes, return the TEAL source code in plain text. This endpoint is only enabled when a node's configuration file sets EnableDeveloperAPI to true.
    pub async fn teal_disassemble(
        &self,
        source: &[u8],
    ) -> Result<TealDisassemble200Response, Error> {
        Ok(
            algonaut_algod::apis::public_api::teal_disassemble(&self.configuration, source)
                .await
                .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Executes TEAL program(s) in context and returns debugging information about the execution. This endpoint is only enabled when a node's configuration file sets EnableDeveloperAPI to true.
    pub async fn teal_dryrun(
        &self,
        request: Option<DryrunRequest>,
    ) -> Result<TealDryrun200Response, Error> {
        Ok(
            algonaut_algod::apis::public_api::teal_dryrun(&self.configuration, request)
                .await
                .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Get parameters for constructing a new transaction.
    pub async fn txn_params(&self) -> Result<TransactionParams200Response, Error> {
        Ok(
            algonaut_algod::apis::public_api::transaction_params(&self.configuration)
                .await
                .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Unset the ledger sync round.
    pub async fn unsync(&self) -> Result<(), Error> {
        Ok(
            algonaut_algod::apis::public_api::unset_sync_round(&self.configuration)
                .await
                .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Waits for a block to appear after round {round} and returns the node's status at the time.
    pub async fn status_after_block(&self, round: u64) -> Result<GetStatus200Response, Error> {
        Ok(
            algonaut_algod::apis::public_api::wait_for_block(&self.configuration, round)
                .await
                .map_err(Into::<AlgodError>::into)?,
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
}
