use algonaut_abi::{
    abi_error::AbiError,
    abi_interactions::{
        AbiArgType, AbiMethod, AbiReturnType, ReferenceArgType, TransactionArgType,
    },
    abi_type::{AbiType, AbiValue},
    make_tuple_type,
};
use algonaut_algod::models::{
    PendingTransactionResponse, SimulateRequest, SimulateRequestTransactionGroup,
    SimulateTransaction200Response,
};
use algonaut_core::{Address, AppId, AssetId, CompiledTeal, MicroAlgos, Round, TxId};
use algonaut_crypto::HashDigest;
use algonaut_transaction::{
    SignedTransaction, Signer, Transaction, TransactionType,
    builder::TransactionParams,
    error::TransactionError,
    signed_transaction,
    transaction::{
        ApplicationCallOnComplete, ApplicationCallTransaction, BoxReference, StateSchema,
        to_tx_type_enum,
    },
    tx_group,
};
use data_encoding::BASE64;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::{Error, algod::v2::Algod};

use instant::Instant;

/// Default timeout matching [`crate::algod::v2::PendingSubmission::confirm`].
const COMPOSER_CONFIRM_TIMEOUT: Duration = Duration::from_secs(60);

/// Poll algod for finality of the given transaction id. The composer
/// already has the tx ids it wants to wait on (post-`send_txns`), so this
/// internal helper is the equivalent of `PendingSubmission::confirm`
/// against an arbitrary id.
async fn poll_until_confirmed(
    algod: &Algod,
    tx_id: &TxId,
) -> Result<PendingTransactionResponse, Error> {
    let start = Instant::now();
    let mut last_round = algod.status().await?.last_round;
    loop {
        let pending = algod.pending_txn(tx_id).await?;
        if pending.confirmed_round.is_some() {
            return Ok(pending);
        }
        if !pending.pool_error.is_empty() {
            return Err(Error::PendingTransactionPoolError {
                reason: pending.pool_error,
            });
        }
        if start.elapsed() >= COMPOSER_CONFIRM_TIMEOUT {
            return Err(Error::PendingTransactionTimeout {
                timeout: COMPOSER_CONFIRM_TIMEOUT,
            });
        }
        last_round = algod.status_after_block(last_round).await?.last_round;
    }
}

/// 4-byte prefix for logged return values, from https://github.com/algorandfoundation/ARCs/blob/main/ARCs/arc-0004.md#standard-format
const ABI_RETURN_HASH: [u8; 4] = [0x15, 0x1f, 0x7c, 0x75];

/// The maximum size of an atomic transaction group.
const MAX_ATOMIC_GROUP_SIZE: usize = 16;

// if the abi type argument number > 15, then the abi types after 14th should be wrapped in a tuple
const MAX_ABI_ARG_TYPE_LEN: usize = 15;

const FOREIGN_OBJ_ABI_UINT_SIZE: usize = 8;

/// Represents an unsigned transactions and a signer that can authorize that transaction.
///
/// `signer` is optional: when `None`, the composer will produce a
/// placeholder `SignedTransaction` whose signature is the all-zero
/// 64-byte sentinel. That mirrors the old `TransactionSigner::Empty`
/// enum variant and is useful for the `/v2/transactions/simulate`
/// "allow-empty-signatures = false" scenarios; never use it against
/// the live submit endpoint.
#[derive(Debug, Clone)]
pub struct TransactionWithSigner {
    /// An unsigned transaction
    pub tx: Transaction,
    /// A transaction signer that can authorize the transaction, or
    /// `None` for an unsigned simulate slot.
    pub signer: Option<Arc<dyn Signer>>,
}

impl TransactionWithSigner {
    /// Build a `TransactionWithSigner` from a transaction and a signer
    /// that will authorize it.
    pub fn new(tx: Transaction, signer: Arc<dyn Signer>) -> Self {
        Self {
            tx,
            signer: Some(signer),
        }
    }

    /// Build a `TransactionWithSigner` that has no real signer
    /// attached. The composer will fill the corresponding slot with an
    /// all-zero placeholder signature — only safe for simulate.
    pub fn unsigned(tx: Transaction) -> Self {
        Self { tx, signer: None }
    }
}

/// Represents the output from a successful ABI method call.
#[derive(Debug, Clone)]
pub struct AbiMethodResult {
    /// The TxID of the transaction that invoked the ABI method call.
    pub tx_id: TxId,
    /// Information about the confirmed transaction that invoked the ABI method call.
    pub tx_info: PendingTransactionResponse,
    /// The method's return value
    pub return_value: Result<AbiMethodReturnValue, AbiReturnDecodeError>,
}

#[derive(Debug, Clone)]
pub struct AbiReturnDecodeError(pub String);

#[derive(Debug, Clone)]
pub enum AbiMethodReturnValue {
    Some(AbiValue),
    Void,
}

/// Contains the parameters for the method AtomicTransactionComposer.AddMethodCall
pub struct AddMethodCallParams {
    /// The ID of the smart contract to call. Set this to 0 to indicate an application creation call.
    pub app_id: AppId,
    /// The method to call on the smart contract
    pub method: AbiMethod,
    /// The arguments to include in the method call. If omitted, no arguments will be passed to the method.
    pub method_args: Vec<AbiArgValue>,
    /// Fee
    pub fee: MicroAlgos,
    /// The address of the sender of this application call
    pub sender: Address,
    /// Transactions params to use for this application call
    pub suggested_params: algonaut_model::client_types::SuggestedParams,
    /// The OnComplete action to take for this application call
    pub on_complete: ApplicationCallOnComplete,
    /// The approval program for this application call. Only set this if this is an application
    /// creation call, or if onComplete is UpdateApplicationOC.
    pub approval_program: Option<CompiledTeal>,
    /// The clear program for this application call. Only set this if this is an application creation
    /// call, or if onComplete is UpdateApplicationOC.
    pub clear_program: Option<CompiledTeal>,
    /// The global schema sizes. Only set this if this is an application creation call.
    pub global_schema: Option<StateSchema>,
    /// The local schema sizes. Only set this if this is an application creation call.
    pub local_schema: Option<StateSchema>,
    /// The number of extra pages to allocate for the application's programs. Only set this if this
    /// is an application creation call.
    pub extra_pages: u32,
    /// The note value for this application call
    pub note: Option<Vec<u8>>,
    /// The lease value for this application call
    pub lease: Option<HashDigest>,
    /// If provided, the address that the sender will be rekeyed to at the conclusion of this application call
    pub rekey_to: Option<Address>,
    /// A transaction Signer that can authorize this application call from sender
    pub signer: Arc<dyn Signer>,
    /// A list of boxes that the app call has access to
    pub boxes: Option<Vec<BoxReference>>,
}

#[derive(Debug, Clone)]
/// ExecuteResult contains the results of successfully calling the Execute method on an
/// AtomicTransactionComposer object.
pub struct ExecuteResult {
    /// The round in which the executed transaction group was confirmed on chain
    /// (optional, because the transaction's confirmed round is optional).
    pub confirmed_round: Option<u64>,
    /// A list of the TxIDs for each transaction in the executed group
    pub tx_ids: Vec<String>,
    /// Return values for all the ABI method calls in the executed group
    pub method_results: Vec<AbiMethodResult>,
}

/// AtcSimulateResult is the result of calling `simulate` on an
/// AtomicTransactionComposer. Mirrors `ExecuteResult` with the raw
/// simulate response attached, but the composer is not transitioned to
/// `Committed` — a subsequent `execute()` is still legal.
#[derive(Debug, Clone)]
pub struct AtcSimulateResult {
    /// TxIDs for each transaction in the simulated group.
    pub tx_ids: Vec<String>,
    /// ABI return values per method call. Errors are surfaced
    /// per-result (the same way `ExecuteResult` does it) so callers can
    /// inspect partial successes.
    pub method_results: Vec<AbiMethodResult>,
    /// Raw simulate response from algod, including failure messages,
    /// budget consumed, eval-overrides, and exec-trace when requested.
    pub simulate_response: algonaut_algod::models::SimulateTransaction200Response,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AtomicTransactionComposerStatus {
    /// The atomic group is still under construction.
    Building,
    /// The atomic group has been finalized, but not yet signed.
    Built,
    /// The atomic group has been finalized and signed, but not yet submitted to the network.
    Signed,
    /// The atomic group has been finalized and signed, then simulated against the simulate
    /// endpoint. Simulate is non-destructive: the composer can still be submitted/executed
    /// afterwards, which is why this variant sits between Signed and Submitted in the
    /// monotonic chain.
    Simulated,
    /// The atomic group has been finalized, signed, and submitted to the network.
    Submitted,
    /// The atomic group has been finalized, signed, submitted, and successfully committed to a block.
    Committed,
}

/// Helper used to construct and execute atomic transaction groups
#[derive(Debug)]
pub struct AtomicTransactionComposer {
    /// The current status of the composer. The status increases monotonically.
    status: AtomicTransactionComposerStatus,

    /// The transaction contexts in the group with their respective signers.
    /// If status is greater than BUILDING then this slice cannot change.
    method_map: HashMap<usize, AbiMethod>,

    txs: Vec<TransactionWithSigner>,

    signed_txs: Vec<SignedTransaction>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum AbiArgValue {
    TxWithSigner(TransactionWithSigner),
    AbiValue(AbiValue),
}

impl AbiArgValue {
    fn address(&self) -> Option<Address> {
        match self {
            AbiArgValue::AbiValue(AbiValue::Address(address)) => Some(*address),
            _ => None,
        }
    }

    fn int(&self) -> Option<BigUint> {
        match self {
            AbiArgValue::AbiValue(AbiValue::Int(int)) => Some(int.clone()),
            _ => None,
        }
    }
}

impl Default for AtomicTransactionComposer {
    fn default() -> Self {
        AtomicTransactionComposer {
            status: AtomicTransactionComposerStatus::Building,
            method_map: HashMap::new(),
            txs: vec![],
            signed_txs: vec![],
        }
    }
}

impl AtomicTransactionComposer {
    /// Returns the number of transactions currently in this atomic group.
    pub fn len(&self) -> usize {
        self.txs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn status(&self) -> AtomicTransactionComposerStatus {
        self.status
    }

    /// Creates a new composer with the same underlying transactions.
    /// The new composer's status will be BUILDING, so additional transactions may be added to it.
    /// This probably can be named better, as it's not strictly a clone - for now keeping it like in the official SDKs.
    pub fn clone_composer(&self) -> AtomicTransactionComposer {
        let mut cloned = AtomicTransactionComposer {
            status: AtomicTransactionComposerStatus::Building,
            method_map: self.method_map.clone(),
            txs: vec![],
            signed_txs: vec![],
        };

        for tx_with_signer in &self.txs {
            let mut tx = tx_with_signer.tx.clone();
            tx.group = None;
            let new_tx_with_signer = TransactionWithSigner {
                tx,
                signer: tx_with_signer.signer.clone(),
            };
            cloned.txs.push(new_tx_with_signer);
        }

        cloned
    }

    /// Adds a transaction to this atomic group.
    ///
    /// An error will be thrown if the composer's status is not Building,
    /// or if adding this transaction causes the current group to exceed MaxAtomicGroupSize.
    pub fn add_transaction(&mut self, txn_with_signer: TransactionWithSigner) -> Result<(), Error> {
        if self.status != AtomicTransactionComposerStatus::Building {
            return Err(Error::ComposerStatusInvalid(
                "add_transaction requires status=Building".to_owned(),
            ));
        }

        if self.len() == MAX_ATOMIC_GROUP_SIZE {
            return Err(Error::ComposerGroupFull {
                max: MAX_ATOMIC_GROUP_SIZE,
            });
        }

        validate_tx(&txn_with_signer.tx, TransactionArgType::Any)?;

        self.txs.push(txn_with_signer);

        Ok(())
    }

    pub fn add_method_call(&mut self, params: &mut AddMethodCallParams) -> Result<(), Error> {
        if self.status != AtomicTransactionComposerStatus::Building {
            return Err(Error::ComposerStatusInvalid(
                "add_method_call requires status=Building".to_owned(),
            ));
        }
        if params.method_args.len() != params.method.args.len() {
            return Err(Error::Msg(format!(
                "incorrect number of arguments were provided: {} != {}",
                params.method_args.len(),
                params.method.args.len()
            )));
        }
        if self.len() + params.method.get_tx_count() > MAX_ATOMIC_GROUP_SIZE {
            return Err(Error::ComposerGroupFull {
                max: MAX_ATOMIC_GROUP_SIZE,
            });
        }

        let mut method_types = vec![];
        let mut method_args: Vec<AbiValue> = vec![];
        let mut txs_with_signer = vec![];
        let mut foreign_accounts = vec![];
        let mut foreign_assets = vec![];
        let mut foreign_apps = vec![];

        for i in 0..params.method.args.len() {
            let mut arg_type = params.method.args[i].clone();
            let arg_value = &params.method_args[i];

            match arg_type.type_()? {
                AbiArgType::Tx(type_) => {
                    add_tx_arg_type_to_method_call(arg_value, type_, &mut txs_with_signer)?
                }
                AbiArgType::Ref(type_) => add_ref_arg_to_method_call(
                    &type_,
                    arg_value,
                    &mut foreign_accounts,
                    &mut foreign_assets,
                    &mut foreign_apps,
                    &mut method_types,
                    &mut method_args,
                    params.sender,
                    params.app_id,
                )?,
                AbiArgType::AbiObj(type_) => {
                    add_abi_obj_arg_to_method_call(
                        &type_,
                        arg_value,
                        &mut method_types,
                        &mut method_args,
                    )?;
                }
            };
        }

        if method_args.len() > MAX_ABI_ARG_TYPE_LEN {
            let (type_, value) = wrap_overflowing_values(&method_types, &method_args)?;
            method_types.push(type_);
            method_args.push(value);
        }

        let mut encoded_abi_args = vec![params.method.get_selector()?.into()];
        for i in 0..method_args.len() {
            encoded_abi_args.push(method_types[i].encode(method_args[i].clone())?);
        }

        let app_call = TransactionType::ApplicationCallTransaction(ApplicationCallTransaction {
            sender: params.sender,
            app_id: Some(params.app_id),
            on_complete: params.on_complete.clone(),
            accounts: Some(foreign_accounts),
            approval_program: params.approval_program.clone(),
            app_arguments: Some(encoded_abi_args),
            clear_state_program: params.clear_program.clone(),
            foreign_apps: Some(foreign_apps),
            foreign_assets: Some(foreign_assets),
            global_state_schema: params.global_schema.clone(),
            local_state_schema: params.local_schema.clone(),
            extra_pages: params.extra_pages,
            boxes: params.boxes.clone(),
        });

        let sp = &params.suggested_params;
        let tx = Transaction {
            fee: params.fee,
            first_valid: Round(sp.last_round()),
            genesis_hash: sp.genesis_hash(),
            last_valid: Round(sp.last_round() + 1000),
            txn_type: app_call,
            genesis_id: Some(sp.genesis_id().clone()),
            group: None,
            lease: params.lease,
            note: params.note.clone(),
            rekey_to: params.rekey_to,
        };

        self.txs.append(&mut txs_with_signer);
        self.txs.push(TransactionWithSigner {
            tx,
            signer: Some(params.signer.clone()),
        });
        self.method_map
            .insert(self.txs.len() - 1, params.method.clone());

        Ok(())
    }

    /// Finalize the transaction group and returned the finalized transactions.
    /// The composer's status will be at least BUILT after executing this method.
    pub fn build_group(&mut self) -> Result<Vec<TransactionWithSigner>, Error> {
        if self.status >= AtomicTransactionComposerStatus::Built {
            return Ok(self.txs.clone());
        }

        if self.txs.is_empty() {
            return Err(Error::EmptyTransactionGroup);
        } else if self.txs.len() > 1 {
            let mut group_txs = vec![];
            for tx in self.txs.iter_mut() {
                group_txs.push(&mut tx.tx);
            }
            tx_group::assign_in_place(&mut group_txs)?;
        }

        self.status = AtomicTransactionComposerStatus::Built;
        Ok(self.txs.clone())
    }

    pub fn gather_signatures(&mut self) -> Result<Vec<SignedTransaction>, Error> {
        if self.status >= AtomicTransactionComposerStatus::Signed {
            return Ok(self.signed_txs.clone());
        }

        let tx_and_signers = self.build_group()?;

        let mut signed_txs = Vec::with_capacity(tx_and_signers.len());
        for tx_with_signer in tx_and_signers.iter() {
            let signed = match &tx_with_signer.signer {
                Some(signer) => {
                    let one = [tx_with_signer.tx.clone()];
                    let mut out = signer.sign_transactions(&one)?;
                    out.pop().ok_or_else(|| {
                        Error::Msg("signer returned no signed transactions".to_owned())
                    })?
                }
                None => {
                    // Mirrors the old `TransactionSigner::Empty` variant:
                    // produce a placeholder `SignedTransaction` whose
                    // 64-byte signature is all zeros. Algod's simulator
                    // detects this as a missing signature; never submit
                    // it to the live endpoint.
                    signed_transaction::placeholder(tx_with_signer.tx.clone())?
                }
            };
            signed_txs.push(signed);
        }

        self.signed_txs = signed_txs.clone();

        self.status = AtomicTransactionComposerStatus::Signed;

        Ok(signed_txs)
    }

    fn get_txs_ids(&self) -> Vec<String> {
        self.signed_txs
            .iter()
            .map(|t| t.transaction_id().0.clone())
            .collect()
    }

    pub async fn submit(&mut self, algod: &Algod) -> Result<Vec<String>, Error> {
        if self.status >= AtomicTransactionComposerStatus::Submitted {
            return Err(Error::ComposerStatusInvalid(
                "submit cannot be called after a previous submit/execute".to_owned(),
            ));
        }

        self.gather_signatures()?;

        algod.send_txns(&self.signed_txs).await?;

        self.status = AtomicTransactionComposerStatus::Submitted;

        Ok(self.get_txs_ids())
    }

    pub async fn execute(&mut self, algod: &Algod) -> Result<ExecuteResult, Error> {
        if self.status >= AtomicTransactionComposerStatus::Committed {
            return Err(Error::ComposerStatusInvalid(
                "execute cannot be called after the composer is committed".to_owned(),
            ));
        }

        self.submit(algod).await?;

        let mut index_to_wait = 0;
        for i in 0..self.signed_txs.len() {
            if self.method_map.contains_key(&i) {
                index_to_wait = i;
                break;
            }
        }

        let tx_id = self.signed_txs[index_to_wait].transaction_id().clone();
        let pending_tx = poll_until_confirmed(algod, &tx_id).await?;

        let mut return_list: Vec<AbiMethodResult> = vec![];

        self.status = AtomicTransactionComposerStatus::Committed;

        for i in 0..self.txs.len() {
            if !self.method_map.contains_key(&i) {
                continue;
            }

            let mut current_tx_id = tx_id.clone(); // this variable wouldn't be needed if our txn in PendingTransaction was complete / able to generate an id
            let mut current_pending_tx = pending_tx.clone();

            if i != index_to_wait {
                let tx_id = self.signed_txs[i].transaction_id().clone();

                match algod.pending_txn(&tx_id).await {
                    Ok(p) => {
                        current_tx_id = tx_id;
                        current_pending_tx = p;
                    }
                    Err(e) => {
                        return_list.push(AbiMethodResult {
                            tx_id,
                            tx_info: pending_tx.clone(),
                            return_value: Err(AbiReturnDecodeError(format!("{e:?}"))),
                        });
                        continue;
                    }
                };
            }

            let return_type = self.method_map[&i].returns.clone().type_()?;
            return_list.push(get_return_value_with_return_type(
                &current_pending_tx,
                &current_tx_id,
                return_type,
            )?);
        }

        Ok(ExecuteResult {
            confirmed_round: pending_tx.confirmed_round,
            tx_ids: self.get_txs_ids(),
            method_results: return_list,
        })
    }

    /// Simulate the composed group through algod's `/v2/transactions/simulate`
    /// endpoint with no power-pack overrides. Equivalent to:
    /// `simulate_with(algod, SimulateRequest::new(vec![default_group]))`.
    pub async fn simulate(&mut self, algod: &Algod) -> Result<AtcSimulateResult, Error> {
        let default = SimulateRequest::new(vec![]);
        self.simulate_with(algod, default).await
    }

    /// Simulate the composed group with a caller-supplied
    /// [`SimulateRequest`] — use this when you need to toggle the
    /// power-pack fields (extra opcode budget, allow-more-logging,
    /// exec-trace-config, etc.).
    ///
    /// The request's `txn_groups` is ignored; this method always
    /// substitutes the composer's own signed group. Any other field on
    /// the request (toggles, exec-trace config, round) is forwarded as
    /// given.
    pub async fn simulate_with(
        &mut self,
        algod: &Algod,
        mut request: SimulateRequest,
    ) -> Result<AtcSimulateResult, Error> {
        if self.status >= AtomicTransactionComposerStatus::Submitted {
            return Err(Error::Msg(
                "Atomic Transaction Composer cannot simulate after submit/execute".to_owned(),
            ));
        }

        self.gather_signatures()?;

        request.txn_groups = vec![SimulateRequestTransactionGroup::new(
            self.signed_txs.clone(),
        )];

        let response: SimulateTransaction200Response = algod.simulate_txns(request).await?;

        // Build per-method ABI return values from the pending-txn
        // payloads embedded in the simulate response (mirrors execute()).
        let mut return_list: Vec<AbiMethodResult> = Vec::new();
        if let Some(group) = response.txn_groups.first() {
            for (i, txn_result) in group.txn_results.iter().enumerate() {
                if !self.method_map.contains_key(&i) {
                    continue;
                }
                let tx_id = self.signed_txs[i].transaction_id().clone();
                let pending_tx = (*txn_result.txn_result).clone();
                let return_type = self.method_map[&i].returns.clone().type_()?;
                return_list.push(get_return_value_with_return_type(
                    &pending_tx,
                    &tx_id,
                    return_type,
                )?);
            }
        }

        self.status = AtomicTransactionComposerStatus::Simulated;

        Ok(AtcSimulateResult {
            tx_ids: self.get_txs_ids(),
            method_results: return_list,
            simulate_response: response,
        })
    }
}

fn get_return_value_with_return_type(
    pending_tx: &PendingTransactionResponse,
    tx_id: &TxId, // our txn in PendingTransaction currently has no fields, so the tx id is passed separately
    return_type: AbiReturnType,
) -> Result<AbiMethodResult, Error> {
    let return_value = match return_type {
        AbiReturnType::Some(return_type) => {
            get_return_value_with_abi_type(pending_tx, &return_type)?
        }
        AbiReturnType::Void => Ok(AbiMethodReturnValue::Void),
    };

    Ok(AbiMethodResult {
        tx_id: tx_id.to_owned(),
        tx_info: pending_tx.clone(),
        return_value,
    })
}

impl From<TransactionError> for Error {
    fn from(e: TransactionError) -> Self {
        Self::Msg(format!("{e:?}"))
    }
}

impl From<AbiError> for Error {
    fn from(e: AbiError) -> Self {
        match e {
            AbiError::Msg(msg) => Self::Msg(msg),
        }
    }
}

fn validate_tx(tx: &Transaction, expected_type: TransactionArgType) -> Result<(), Error> {
    if tx.group.is_some() {
        return Err(Error::Msg("Expected empty group id".to_owned()));
    }

    if expected_type != TransactionArgType::Any
        && expected_type != TransactionArgType::One(to_tx_type_enum(&tx.txn_type))
    {
        return Err(Error::Msg(format!(
            "expected transaction with type {expected_type:?}, but got type {:?}",
            tx.txn_type
        )));
    }

    Ok(())
}

fn add_tx_arg_type_to_method_call(
    arg_value: &AbiArgValue,
    expected_type: TransactionArgType,
    txs_with_signer: &mut Vec<TransactionWithSigner>,
) -> Result<(), Error> {
    let txn_and_signer = match arg_value {
        AbiArgValue::TxWithSigner(tx_with_signer) => tx_with_signer,
        _ => {
            return Err(Error::Msg(
                "invalid arg value, expected transaction".to_owned(),
            ));
        }
    };

    validate_tx(&txn_and_signer.tx, expected_type)?;
    txs_with_signer.push(txn_and_signer.to_owned());

    Ok(())
}

fn add_abi_obj_arg_to_method_call(
    abi_type: &AbiType,
    arg_value: &AbiArgValue,
    method_types: &mut Vec<AbiType>,
    method_args: &mut Vec<AbiValue>,
) -> Result<(), Error> {
    match arg_value {
        AbiArgValue::AbiValue(value) => {
            method_types.push(abi_type.clone());
            method_args.push(value.clone());
        }
        AbiArgValue::TxWithSigner(_) => {
            return Err(Error::Msg(
                "Invalid state: shouldn't be here with a tx with signer value type".to_owned(),
            ));
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn add_ref_arg_to_method_call(
    arg_type: &ReferenceArgType,
    arg_value: &AbiArgValue,

    foreign_accounts: &mut Vec<Address>,
    foreign_assets: &mut Vec<AssetId>,
    foreign_apps: &mut Vec<AppId>,

    method_types: &mut Vec<AbiType>,
    method_args: &mut Vec<AbiValue>,

    sender: Address,
    app_id: AppId,
) -> Result<(), Error> {
    let index = add_to_foreign_array(
        arg_type,
        arg_value,
        foreign_accounts,
        foreign_assets,
        foreign_apps,
        sender,
        app_id,
    )?;

    method_types.push(AbiType::uint(FOREIGN_OBJ_ABI_UINT_SIZE)?);
    method_args.push(AbiValue::Int(index.into()));

    Ok(())
}

/// Adds arg value to its respective foreign array
/// Returns index that can be used to reference `arg_value` in its foreign array (in TEAL).
fn add_to_foreign_array(
    arg_type: &ReferenceArgType,
    arg_value: &AbiArgValue,
    foreign_accounts: &mut Vec<Address>,
    foreign_assets: &mut Vec<AssetId>,
    foreign_apps: &mut Vec<AppId>,
    sender: Address,
    app_id: AppId,
) -> Result<usize, Error> {
    match arg_type {
        ReferenceArgType::Account => match arg_value.address() {
            Some(address) => Ok(populate_foreign_array(
                address,
                foreign_accounts,
                Some(sender),
            )),
            _ => Err(Error::Msg(format!(
                "Invalid value type: {arg_value:?} for arg type: {arg_type:?}"
            ))),
        },
        ReferenceArgType::Asset => match arg_value.int() {
            Some(int) => {
                let intu64 = int.to_u64().ok_or_else(|| {
                    AbiError::Msg(format!("big int: {int} couldn't be converted to u64"))
                })?;

                Ok(populate_foreign_array(
                    AssetId(intu64),
                    foreign_assets,
                    None,
                ))
            }
            _ => Err(Error::Msg(format!(
                "Invalid value type: {arg_value:?} for arg type: {arg_type:?}"
            ))),
        },
        ReferenceArgType::Application => match arg_value.int() {
            Some(int) => {
                let intu64 = int.to_u64().ok_or_else(|| {
                    AbiError::Msg(format!("big int: {int} couldn't be converted to u64"))
                })?;

                Ok(populate_foreign_array(
                    AppId(intu64),
                    foreign_apps,
                    Some(app_id),
                ))
            }
            _ => Err(Error::Msg(format!(
                "Invalid value type: {arg_value:?} for arg type: {arg_type:?}"
            ))),
        },
    }
}

fn wrap_overflowing_values(
    method_types: &[AbiType],
    method_args: &[AbiValue],
) -> Result<(AbiType, AbiValue), Error> {
    let mut wrapped_abi_types = vec![];
    let mut wrapped_value_list = vec![];

    for i in (MAX_ABI_ARG_TYPE_LEN - 1)..method_args.len() {
        wrapped_abi_types.push(method_types[i].clone());
        wrapped_value_list.push(method_args[i].clone());
    }

    let tuple_type = make_tuple_type(&wrapped_abi_types)?;

    Ok((tuple_type, AbiValue::Array(wrapped_value_list)))
}

/// Add a value to an application call's foreign array. The addition will be as compact as possible,
/// and this function will return an index that can be used to reference `object_to_add` in `obj_array`.
///
/// # Arguments
///
/// * `obj_to_add` - The value to add to the array. If this value is already present in the array,
///   it will not be added again. Instead, the existing index will be returned.
/// * `obj_array` - The existing foreign array. This input may be modified to append `obj_to_add`.
/// * `zeroth_obj` - If provided, this value indicated two things: the 0 value is special for this
///   array, so all indexes into `obj_array` must start at 1; additionally, if `obj_to_add` equals
///   `zeroth_obj`, then `obj_to_add` will not be added to the array, and instead the 0 indexes will be returned.
///
/// Returns an index that can be used to reference `obj_to_add` in `obj_array`.
fn populate_foreign_array<T: Eq>(
    obj_to_add: T,
    obj_array: &mut Vec<T>,
    zeroth_obj: Option<T>,
) -> usize {
    if let Some(o) = &zeroth_obj
        && &obj_to_add == o
    {
        return 0;
    }

    let start_from: usize = zeroth_obj.map(|_| 1).unwrap_or(0);
    let search_in_vec_index = obj_array.iter().position(|o| o == &obj_to_add);
    if let Some(index) = search_in_vec_index {
        start_from + index
    } else {
        obj_array.push(obj_to_add);
        obj_array.len() - 1 + start_from
    }
}

fn get_return_value_with_abi_type(
    pending_tx: &PendingTransactionResponse,
    abi_type: &AbiType,
) -> Result<Result<AbiMethodReturnValue, AbiReturnDecodeError>, Error> {
    if pending_tx.logs.is_none() {
        return Err(Error::MissingReturnLog);
    }

    // safe to unwrap given the previous check
    let logs = &pending_tx.logs.clone().unwrap();

    if logs.is_empty() {
        return Err(Error::MissingReturnLog);
    }

    let ret_line = &logs[logs.len() - 1];

    let decoded_ret_line: Vec<u8> = BASE64
        .decode(&ret_line.0[..])
        .map_err(|e| Error::Msg(format!("BASE64 Decoding error: {e:?}")))?;

    if !check_log_ret(&decoded_ret_line) {
        return Err(Error::MissingReturnLog);
    }

    let abi_encoded = &decoded_ret_line[ABI_RETURN_HASH.len()..decoded_ret_line.len()];
    Ok(match abi_type.decode(abi_encoded) {
        Ok(decoded) => Ok(AbiMethodReturnValue::Some(decoded)),
        Err(e) => Err(AbiReturnDecodeError(format!("{e:?}"))),
    })
}

fn check_log_ret(log_line: &[u8]) -> bool {
    let abi_return_hash_len = ABI_RETURN_HASH.len();
    if log_line.len() < abi_return_hash_len {
        return false;
    }
    for i in 0..abi_return_hash_len {
        if log_line[i] != ABI_RETURN_HASH[i] {
            return false;
        }
    }
    true
}
