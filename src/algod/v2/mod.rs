use self::error::AlgodError;
use crate::Error;
use algonaut_algod::{
    apis::configuration::{ApiKey, Configuration},
    ext::block::BlockResponse,
    models::{
        self, Account, AccountApplicationInformation200Response,
        AccountApplicationsInformation200Response, AccountAssetsInformation200Response,
        Application, Asset, DebugSettingsProf, DryrunRequest, GetApplicationBoxes200Response,
        GetBlockHash200Response, GetBlockLogs200Response, GetBlockTimeStampOffset200Response,
        GetBlockTxids200Response, GetPendingTransactionsByAddress200Response,
        GetSyncRound200Response, GetTransactionGroupLedgerStateDeltasForRound200Response,
        GetTransactionProof200Response, LightBlockHeaderProof, PendingTransactionResponse,
        RawTransaction200Response, SimulateRequest, SimulateTransaction200Response, StateProof,
        TealDisassemble200Response, TealDryrun200Response, Version,
    },
};
use algonaut_core::{Address, AppId, AssetId, CompiledTeal, MicroAlgos, Round, ToMsgPack, TxId};
use algonaut_encoding::decode_base64;
use algonaut_model::client_responses::{NodeStatus, SuggestedParams, Supply};
use algonaut_transaction::SignedTransaction;

/// Whether `teal_compile` should ask algod to include a source-map alongside
/// the compiled bytes. Distinct from [`algonaut_abi::sourcemap::SourceMap`],
/// which is the *parsed* source-map type returned by
/// [`Algod::teal_compile_with_sourcemap`].
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SourceMap {
    /// Request a source-map. The map travels back on the JSON response but
    /// [`Algod::teal_compile`] still returns only the compiled bytes — call
    /// [`Algod::teal_compile_with_sourcemap`] if you need the parsed map.
    Emit,
    /// Do not request a source-map.
    Skip,
}

impl SourceMap {
    fn as_option_bool(self) -> Option<bool> {
        match self {
            SourceMap::Emit => Some(true),
            SourceMap::Skip => None,
        }
    }
}

/// Error class wrapping errors from algonaut_algod
pub(crate) mod error;
mod pending_submission;
pub use pending_submission::PendingSubmission;

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
        address: &Address,
        app_id: AppId,
    ) -> Result<AccountApplicationInformation200Response, Error> {
        Ok(
            algonaut_algod::apis::public_api::account_application_information(
                &self.configuration,
                &address.to_string(),
                app_id.0,
                None,
            )
            .await
            .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Lookup an account's application holdings (local state and params if the
    /// account is the creator), paginated by application ID.
    ///
    /// `include` controls which sub-objects are returned (e.g. `["params"]`).
    pub async fn account_apps(
        &self,
        address: &Address,
        limit: Option<u64>,
        next: Option<&str>,
        include: Option<&[String]>,
    ) -> Result<AccountApplicationsInformation200Response, Error> {
        Ok(
            algonaut_algod::apis::public_api::account_applications_information(
                &self.configuration,
                &address.to_string(),
                limit,
                next,
                include.map(<[String]>::to_vec),
            )
            .await
            .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Lookup an account's asset holdings, paginated by asset ID.
    pub async fn account_assets(
        &self,
        address: &Address,
        limit: Option<u64>,
        next: Option<&str>,
    ) -> Result<AccountAssetsInformation200Response, Error> {
        Ok(
            algonaut_algod::apis::public_api::account_assets_information(
                &self.configuration,
                &address.to_string(),
                limit,
                next,
            )
            .await
            .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Given a specific account public key, this call returns the accounts status, balance and spendable amounts
    pub async fn account(&self, address: &Address) -> Result<Account, Error> {
        Ok(algonaut_algod::apis::public_api::account_information(
            &self.configuration,
            &address.to_string(),
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
    pub async fn app_box(&self, app_id: AppId, name: &str) -> Result<models::Box, Error> {
        Ok(
            algonaut_algod::apis::public_api::get_application_box_by_name(
                &self.configuration,
                app_id.0,
                name,
            )
            .await
            .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Given an application ID, return all Box names. No particular ordering is guaranteed. Request fails when client or server-side configured limits prevent returning all Box names.
    pub async fn app_boxes(
        &self,
        app_id: AppId,
        max: Option<u64>,
    ) -> Result<GetApplicationBoxes200Response, Error> {
        Ok(algonaut_algod::apis::public_api::get_application_boxes(
            &self.configuration,
            app_id.0,
            max,
        )
        .await
        .map_err(Into::<AlgodError>::into)?)
    }

    /// Given a application ID, it returns application information including creator, approval and clear programs, global and local schemas, and global state.
    pub async fn app(&self, app_id: AppId) -> Result<Application, Error> {
        Ok(
            algonaut_algod::apis::public_api::get_application_by_id(&self.configuration, app_id.0)
                .await
                .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Given a asset ID, it returns asset information including creator, name, total supply and special addresses.
    pub async fn asset(&self, asset_id: AssetId) -> Result<Asset, Error> {
        Ok(
            algonaut_algod::apis::public_api::get_asset_by_id(&self.configuration, asset_id.0)
                .await
                .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Get the block for the given round.
    pub async fn block(&self, round: u64) -> Result<BlockResponse, Error> {
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

    /// Get all of the logs from outer and inner app calls in the given round.
    pub async fn block_logs(&self, round: u64) -> Result<GetBlockLogs200Response, Error> {
        Ok(
            algonaut_algod::apis::public_api::get_block_logs(&self.configuration, round)
                .await
                .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Get the top level transaction IDs for the block on the given round.
    pub async fn block_txids(&self, round: u64) -> Result<GetBlockTxids200Response, Error> {
        Ok(
            algonaut_algod::apis::public_api::get_block_txids(&self.configuration, round)
                .await
                .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Get the current timestamp offset.
    pub async fn block_timestamp_offset(
        &self,
    ) -> Result<GetBlockTimeStampOffset200Response, Error> {
        Ok(
            algonaut_algod::apis::public_api::get_block_time_stamp_offset(&self.configuration)
                .await
                .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Set the timestamp offset (seconds) for blocks in dev mode. Providing an
    /// offset of 0 will unset this value and try to use the real clock for the
    /// timestamp.
    pub async fn set_block_timestamp_offset(&self, offset: u64) -> Result<(), Error> {
        Ok(
            algonaut_algod::apis::public_api::set_block_time_stamp_offset(
                &self.configuration,
                offset,
            )
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

    /// Get a ledger delta for a given transaction group, identified by the ID
    /// of the first transaction in the group.
    pub async fn txn_group_state_delta(&self, id: &TxId) -> Result<serde_json::Value, Error> {
        Ok(
            algonaut_algod::apis::public_api::get_ledger_state_delta_for_transaction_group(
                &self.configuration,
                id.as_str(),
                None,
            )
            .await
            .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Get ledger deltas for every transaction group in a given round.
    pub async fn txn_group_state_deltas_for_round(
        &self,
        round: u64,
    ) -> Result<GetTransactionGroupLedgerStateDeltasForRound200Response, Error> {
        Ok(
            algonaut_algod::apis::public_api::get_transaction_group_ledger_state_deltas_for_round(
                &self.configuration,
                round,
                None,
            )
            .await
            .map_err(Into::<AlgodError>::into)?,
        )
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
    ///
    /// `format` selects the response encoding (`"json"` or `"msgpack"`).
    pub async fn pending_txns(
        &self,
        max: Option<u64>,
        format: Option<&str>,
    ) -> Result<GetPendingTransactionsByAddress200Response, Error> {
        Ok(algonaut_algod::apis::public_api::get_pending_transactions(
            &self.configuration,
            max,
            format,
        )
        .await
        .map_err(Into::<AlgodError>::into)?)
    }

    /// Get the list of pending transactions by address, sorted by priority, in decreasing order, truncated at the end at MAX. If MAX = 0, returns all pending transactions.
    ///
    /// `format` selects the response encoding (`"json"` or `"msgpack"`).
    pub async fn address_pending_txns(
        &self,
        address: &Address,
        max: Option<u64>,
        format: Option<&str>,
    ) -> Result<GetPendingTransactionsByAddress200Response, Error> {
        Ok(
            algonaut_algod::apis::public_api::get_pending_transactions_by_address(
                &self.configuration,
                &address.to_string(),
                max,
                format,
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
    pub async fn status(&self) -> Result<NodeStatus, Error> {
        let raw = algonaut_algod::apis::public_api::get_status(&self.configuration)
            .await
            .map_err(Into::<AlgodError>::into)?;
        Ok(NodeStatus {
            catchup_time: raw.catchup_time,
            last_round: Round(raw.last_round),
            last_version: raw.last_version,
            next_version: raw.next_version,
            next_version_round: Round(raw.next_version_round),
            next_version_supported: raw.next_version_supported,
            stopped_at_unsupported_round: raw.stopped_at_unsupported_round,
            time_since_last_round: raw.time_since_last_round,
        })
    }

    /// Get the current supply reported by the ledger.
    pub async fn supply(&self) -> Result<Supply, Error> {
        let raw = algonaut_algod::apis::public_api::get_supply(&self.configuration)
            .await
            .map_err(Into::<AlgodError>::into)?;
        Ok(Supply {
            current_round: Round(raw.current_round),
            total_money: MicroAlgos(raw.total_money),
            online_money: MicroAlgos(raw.online_money),
        })
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
        txid: &TxId,
    ) -> Result<GetTransactionProof200Response, Error> {
        Ok(algonaut_algod::apis::public_api::get_transaction_proof(
            &self.configuration,
            round,
            txid.as_str(),
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
    pub async fn pending_txn(&self, txid: &TxId) -> Result<PendingTransactionResponse, Error> {
        Ok(
            algonaut_algod::apis::public_api::pending_transaction_information(
                &self.configuration,
                txid.as_str(),
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

    /// Wrap an existing transaction id in a [`PendingSubmission`] so callers
    /// that already have the id (e.g. they submitted earlier and stashed it)
    /// can still poll for finality with the shared
    /// [`PendingSubmission::confirm`] implementation.
    pub fn pending_submission(&self, tx_id: &TxId) -> PendingSubmission {
        PendingSubmission::new(self.clone(), tx_id.clone())
    }

    /// Broadcasts a single signed transaction and returns a
    /// [`PendingSubmission`] handle that polls algod for finality.
    pub async fn submit(&self, txn: &SignedTransaction) -> Result<PendingSubmission, Error> {
        let resp = self.send_txn(txn).await?;
        Ok(PendingSubmission::new(self.clone(), TxId::from(resp.tx_id)))
    }

    /// Broadcasts a transaction group and returns a [`PendingSubmission`]
    /// handle for the group's representative transaction id.
    ///
    /// Atomic if the transactions share a
    /// [group](algonaut_transaction::transaction::Transaction::group).
    pub async fn submit_txns(
        &self,
        txns: &[SignedTransaction],
    ) -> Result<PendingSubmission, Error> {
        let resp = self.send_txns(txns).await?;
        Ok(PendingSubmission::new(self.clone(), TxId::from(resp.tx_id)))
    }

    /// Broadcasts already-encoded msgpack transaction bytes and returns a
    /// [`PendingSubmission`] handle for the returned transaction id.
    pub async fn submit_raw(&self, rawtxn: &[u8]) -> Result<PendingSubmission, Error> {
        let resp = self.send_raw_txn(rawtxn).await?;
        Ok(PendingSubmission::new(self.clone(), TxId::from(resp.tx_id)))
    }

    /// Broadcasts a raw transaction or transaction group to the network without
    /// performing ahead-of-time checks. Returns as soon as the request is
    /// accepted, without a transaction ID.
    pub async fn send_raw_txn_async(&self, rawtxn: &[u8]) -> Result<(), Error> {
        Ok(
            algonaut_algod::apis::public_api::raw_transaction_async(&self.configuration, rawtxn)
                .await
                .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Broadcasts a transaction to the network without ahead-of-time checks.
    pub async fn send_txn_async(&self, txn: &SignedTransaction) -> Result<(), Error> {
        self.send_raw_txn_async(&txn.to_msg_pack()?).await
    }

    /// Broadcasts a transaction group to the network without ahead-of-time checks.
    ///
    /// Atomic if the transactions share a [group](algonaut_transaction::transaction::Transaction::group)
    pub async fn send_txns_async(&self, txns: &[SignedTransaction]) -> Result<(), Error> {
        let mut bytes = vec![];
        for t in txns {
            bytes.push(t.to_msg_pack()?);
        }
        self.send_raw_txn_async(&bytes.concat()).await
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
        sourcemap: SourceMap,
    ) -> Result<CompiledTeal, Error> {
        let api_compiled_teal = algonaut_algod::apis::public_api::teal_compile(
            &self.configuration,
            source,
            sourcemap.as_option_bool(),
        )
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

    /// Compile TEAL with `sourcemap=true` and return the compiled bytes
    /// plus the parsed [`SourceMap`].
    pub async fn teal_compile_with_sourcemap(
        &self,
        source: &[u8],
    ) -> Result<(CompiledTeal, algonaut_abi::sourcemap::SourceMap), Error> {
        let api_compiled_teal =
            algonaut_algod::apis::public_api::teal_compile(&self.configuration, source, Some(true))
                .await
                .map_err(Into::<AlgodError>::into)?;
        let bytes = decode_base64(api_compiled_teal.result.as_bytes())?;
        let raw = api_compiled_teal.sourcemap.ok_or(Error::MissingSourcemap)?;
        let json = serde_json::to_string(&raw).map_err(|e| Error::Msg(e.to_string()))?;
        let map = algonaut_abi::sourcemap::SourceMap::from_json(&json)
            .map_err(|e| Error::Msg(e.to_string()))?;
        Ok((CompiledTeal(bytes), map))
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
    pub async fn txn_params(&self) -> Result<SuggestedParams, Error> {
        let raw = algonaut_algod::apis::public_api::transaction_params(&self.configuration)
            .await
            .map_err(Into::<AlgodError>::into)?;
        Ok(SuggestedParams {
            consensus_version: raw.consensus_version,
            fee_per_byte: MicroAlgos(raw.fee),
            genesis_hash: raw.genesis_hash,
            genesis_id: raw.genesis_id,
            last_round: Round(raw.last_round),
            min_fee: MicroAlgos(raw.min_fee),
        })
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
    pub async fn status_after_block(&self, round: Round) -> Result<NodeStatus, Error> {
        let raw = algonaut_algod::apis::public_api::wait_for_block(&self.configuration, round.0)
            .await
            .map_err(Into::<AlgodError>::into)?;
        Ok(NodeStatus {
            catchup_time: raw.catchup_time,
            last_round: Round(raw.last_round),
            last_version: raw.last_version,
            next_version: raw.next_version,
            next_version_round: Round(raw.next_version_round),
            next_version_supported: raw.next_version_supported,
            stopped_at_unsupported_round: raw.stopped_at_unsupported_round,
            time_since_last_round: raw.time_since_last_round,
        })
    }

    /// Generate and install participation keys to the node, valid for rounds
    /// `first` through `last`. Returns the participation ID of the new keys.
    pub async fn generate_participation_keys(
        &self,
        address: &Address,
        first: u64,
        last: u64,
        dilution: Option<u64>,
    ) -> Result<String, Error> {
        Ok(
            algonaut_algod::apis::private_api::generate_participation_keys(
                &self.configuration,
                &address.to_string(),
                first,
                last,
                dilution,
            )
            .await
            .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Returns the merged (defaults + overrides) node config file as a JSON string.
    pub async fn config(&self) -> Result<String, Error> {
        Ok(
            algonaut_algod::apis::private_api::get_config(&self.configuration)
                .await
                .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Retrieves the current settings for blocking and mutex profiles.
    pub async fn debug_settings_prof(&self) -> Result<DebugSettingsProf, Error> {
        Ok(
            algonaut_algod::apis::private_api::get_debug_settings_prof(&self.configuration)
                .await
                .map_err(Into::<AlgodError>::into)?,
        )
    }

    /// Enables blocking and mutex profiles, and returns the old settings.
    pub async fn set_debug_settings_prof(&self) -> Result<DebugSettingsProf, Error> {
        Ok(
            algonaut_algod::apis::private_api::put_debug_settings_prof(&self.configuration)
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
