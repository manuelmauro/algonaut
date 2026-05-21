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
use algonaut_core::{Address, AppId, AssetId, Round, TxId};
use algonaut_transaction::{
    SignedTransaction, Signer, Transaction, TransactionType,
    builder::TransactionParams,
    signed_transaction,
    transaction::{ApplicationCallTransaction, to_tx_type_enum},
    tx_group,
};

use data_encoding::BASE64;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::{Error, algod::v2::Algod, algod::v2::PendingSubmission};

use instant::Instant;

mod method_call;
pub use method_call::{MethodCall, MethodCallBuilder};

/// Default timeout matching [`crate::algod::v2::PendingSubmission::confirm`].
const COMPOSER_CONFIRM_TIMEOUT: Duration = Duration::from_secs(60);

/// Poll algod for finality of the given transaction id. A signed group
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
/// `signer` is optional: when `None`, signing produces a placeholder
/// `SignedTransaction` whose signature is the all-zero 64-byte sentinel.
/// That mirrors the old `TransactionSigner::Empty` enum variant and is
/// useful for the `/v2/transactions/simulate` "allow-empty-signatures =
/// false" scenarios; never use it against the live submit endpoint.
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
    /// attached. Signing fills the corresponding slot with an all-zero
    /// placeholder signature — only safe for simulate.
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

/// Results of successfully [`executing`](SignedAtomicGroup::execute) a
/// transaction group: the confirmed round, the group's transaction ids,
/// and the decoded ABI return value for each method call.
#[derive(Debug, Clone)]
pub struct ExecuteOutcome {
    /// The round in which the executed transaction group was confirmed on chain
    /// (optional, because the transaction's confirmed round is optional).
    pub confirmed_round: Option<u64>,
    /// A list of the TxIDs for each transaction in the executed group
    pub tx_ids: Vec<String>,
    /// Return values for all the ABI method calls in the executed group
    pub method_results: Vec<AbiMethodResult>,
}

/// Result of [`simulating`](UnsignedAtomicGroup::simulate) a group. Mirrors
/// [`ExecuteOutcome`] with the raw simulate response attached. Because
/// simulate borrows the group (`&self`), the same group can still be
/// signed and executed afterwards.
#[derive(Debug, Clone)]
pub struct SimulateOutcome {
    /// TxIDs for each transaction in the simulated group.
    pub tx_ids: Vec<String>,
    /// ABI return values per method call. Errors are surfaced
    /// per-result (the same way [`ExecuteOutcome`] does it) so callers
    /// can inspect partial successes.
    pub method_results: Vec<AbiMethodResult>,
    /// Raw simulate response from algod, including failure messages,
    /// budget consumed, eval-overrides, and exec-trace when requested.
    pub simulate_response: SimulateTransaction200Response,
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

/// A pending entry in an [`AtomicGroupBuilder`]. Entries are only assembled and
/// validated when [`AtomicGroupBuilder::build`] is called, so the `add_*`
/// methods can stay infallible.
#[derive(Debug, Clone)]
enum AtomicGroupEntry {
    Transaction(TransactionWithSigner),
    MethodCall(MethodCall),
}

/// Builder state of an atomic transaction group.
///
/// Add pre-built transactions and ABI method calls in any mix, then
/// [`build`](AtomicGroupBuilder::build) to stamp the group id and advance to
/// the [`UnsignedAtomicGroup`] state. The `add_*` methods are infallible and
/// only record intent; all validation — group-size limit, ABI argument
/// counts, per-transaction checks — happens in `build`.
///
/// `AtomicGroupBuilder` is `Clone`: clone it to snapshot a common prefix and
/// build several groups from it (this replaces the old
/// `clone_composer`).
#[derive(Debug, Clone, Default)]
pub struct AtomicGroupBuilder {
    entries: Vec<AtomicGroupEntry>,
}

impl AtomicGroupBuilder {
    /// Start a new, empty group builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a pre-built transaction with its signer to the group.
    pub fn add_transaction(mut self, txn_with_signer: TransactionWithSigner) -> Self {
        self.entries
            .push(AtomicGroupEntry::Transaction(txn_with_signer));
        self
    }

    /// Add an ABI method call to the group. Build the [`MethodCall`]
    /// with [`MethodCall::new`] and the [`MethodCallBuilder`] setters.
    pub fn add_method_call(mut self, call: MethodCall) -> Self {
        self.entries.push(AtomicGroupEntry::MethodCall(call));
        self
    }

    /// Finalize the group: assemble every entry, enforce the size and
    /// ABI-argument invariants, stamp the group id, and produce an
    /// [`UnsignedAtomicGroup`].
    ///
    /// Returns [`Error::EmptyTransactionGroup`] if no entries were
    /// added, or [`Error::ComposerGroupFull`] if the assembled group
    /// would exceed the protocol's 16-transaction limit.
    pub fn build(self) -> Result<UnsignedAtomicGroup, Error> {
        let mut txs: Vec<TransactionWithSigner> = Vec::new();
        let mut method_map: HashMap<usize, AbiMethod> = HashMap::new();

        for entry in self.entries {
            match entry {
                AtomicGroupEntry::Transaction(txn_with_signer) => {
                    if txs.len() == MAX_ATOMIC_GROUP_SIZE {
                        return Err(Error::ComposerGroupFull {
                            max: MAX_ATOMIC_GROUP_SIZE,
                        });
                    }
                    validate_tx(&txn_with_signer.tx, TransactionArgType::Any)?;
                    txs.push(txn_with_signer);
                }
                AtomicGroupEntry::MethodCall(call) => {
                    process_method_call(call, &mut txs, &mut method_map)?;
                }
            }
        }

        if txs.is_empty() {
            return Err(Error::EmptyTransactionGroup);
        }
        if txs.len() > 1 {
            let mut group_txs: Vec<&mut Transaction> = txs.iter_mut().map(|t| &mut t.tx).collect();
            tx_group::assign_in_place(&mut group_txs)?;
        }

        Ok(UnsignedAtomicGroup { txs, method_map })
    }
}

/// A built, group-id-stamped transaction group, ready to sign or
/// simulate. Reach this state via [`AtomicGroupBuilder::build`].
#[derive(Debug, Clone)]
pub struct UnsignedAtomicGroup {
    txs: Vec<TransactionWithSigner>,
    method_map: HashMap<usize, AbiMethod>,
}

impl UnsignedAtomicGroup {
    /// The group's transactions, in order, each with its signer.
    pub fn transactions(&self) -> &[TransactionWithSigner] {
        &self.txs
    }

    /// Sign every transaction with its attached signer, advancing to the
    /// [`SignedAtomicGroup`] state. Transactions whose signer is `None` get an
    /// all-zero placeholder signature (simulate-only).
    pub fn sign(self) -> Result<SignedAtomicGroup, Error> {
        let signed_txs = sign_group(&self.txs)?;
        Ok(SignedAtomicGroup {
            signed_txs,
            method_map: self.method_map,
        })
    }

    /// Simulate the group through algod's `/v2/transactions/simulate`
    /// endpoint with no power-pack overrides.
    ///
    /// Takes `&self`: simulation is non-destructive, so the same group
    /// can still be signed and executed afterwards.
    pub async fn simulate(&self, algod: &Algod) -> Result<SimulateOutcome, Error> {
        self.simulate_with(algod, SimulateRequest::new(vec![]))
            .await
    }

    /// Simulate with a caller-supplied [`SimulateRequest`] — use this to
    /// toggle the power-pack fields (extra opcode budget,
    /// allow-more-logging, exec-trace-config, etc.).
    ///
    /// The request's `txn_groups` is ignored; this method always
    /// substitutes the group's own (placeholder-)signed transactions.
    /// Any other field on the request is forwarded as given.
    pub async fn simulate_with(
        &self,
        algod: &Algod,
        mut request: SimulateRequest,
    ) -> Result<SimulateOutcome, Error> {
        let signed_txs = sign_group(&self.txs)?;

        request.txn_groups = vec![SimulateRequestTransactionGroup::new(signed_txs.clone())];

        let response: SimulateTransaction200Response = algod.simulate_txns(request).await?;

        // Build per-method ABI return values from the pending-txn
        // payloads embedded in the simulate response (mirrors execute()).
        let mut method_results: Vec<AbiMethodResult> = Vec::new();
        if let Some(group) = response.txn_groups.first() {
            for (i, txn_result) in group.txn_results.iter().enumerate() {
                if !self.method_map.contains_key(&i) {
                    continue;
                }
                let tx_id = signed_txs[i].transaction_id().clone();
                let pending_tx = (*txn_result.txn_result).clone();
                let return_type = self.method_map[&i].returns.clone().type_()?;
                method_results.push(get_return_value_with_return_type(
                    &pending_tx,
                    &tx_id,
                    return_type,
                )?);
            }
        }

        Ok(SimulateOutcome {
            tx_ids: tx_ids(&signed_txs),
            method_results,
            simulate_response: response,
        })
    }
}

/// A signed transaction group, ready to submit or execute. Reach this
/// state via [`UnsignedAtomicGroup::sign`].
#[derive(Debug, Clone)]
pub struct SignedAtomicGroup {
    signed_txs: Vec<SignedTransaction>,
    method_map: HashMap<usize, AbiMethod>,
}

impl SignedAtomicGroup {
    /// The signed transactions, in group order.
    pub fn signed_transactions(&self) -> &[SignedTransaction] {
        &self.signed_txs
    }

    /// Broadcast the group and return a [`PendingSubmission`] handle.
    /// Call [`PendingSubmission::confirm`] to await finality, or hold the
    /// handle and confirm later.
    pub async fn submit(self, algod: &Algod) -> Result<PendingSubmission, Error> {
        algod.submit_txns(&self.signed_txs).await
    }

    /// Submit the group, wait for it to be confirmed, and decode each ABI
    /// method call's return value.
    pub async fn execute(self, algod: &Algod) -> Result<ExecuteOutcome, Error> {
        algod.send_txns(&self.signed_txs).await?;

        let index_to_wait = (0..self.signed_txs.len())
            .find(|i| self.method_map.contains_key(i))
            .unwrap_or(0);

        let tx_id = self.signed_txs[index_to_wait].transaction_id().clone();
        let pending_tx = poll_until_confirmed(algod, &tx_id).await?;

        let mut method_results: Vec<AbiMethodResult> = vec![];

        for i in 0..self.signed_txs.len() {
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
                        method_results.push(AbiMethodResult {
                            tx_id,
                            tx_info: pending_tx.clone(),
                            return_value: Err(AbiReturnDecodeError(format!("{e:?}"))),
                        });
                        continue;
                    }
                };
            }

            let return_type = self.method_map[&i].returns.clone().type_()?;
            method_results.push(get_return_value_with_return_type(
                &current_pending_tx,
                &current_tx_id,
                return_type,
            )?);
        }

        Ok(ExecuteOutcome {
            confirmed_round: pending_tx.confirmed_round,
            tx_ids: tx_ids(&self.signed_txs),
            method_results,
        })
    }
}

/// Sign every transaction in `txs` with its own signer, one input at a
/// time, so the signed group keeps input order regardless of how many
/// distinct signers are involved. Transactions whose signer is `None`
/// get an all-zero placeholder signature (simulate-only).
fn sign_group(txs: &[TransactionWithSigner]) -> Result<Vec<SignedTransaction>, Error> {
    let mut signed_txs = Vec::with_capacity(txs.len());
    for tx_with_signer in txs {
        let signed = match &tx_with_signer.signer {
            Some(signer) => {
                let one = [tx_with_signer.tx.clone()];
                signer.sign_transactions(&one)?.pop().ok_or_else(|| {
                    Error::Internal("signer returned no signed transactions".to_owned())
                })?
            }
            None => {
                // Mirrors the old `TransactionSigner::Empty` variant:
                // produce a placeholder `SignedTransaction` whose 64-byte
                // signature is all zeros. Algod's simulator detects this
                // as a missing signature; never submit it to the live
                // endpoint.
                signed_transaction::placeholder(tx_with_signer.tx.clone())?
            }
        };
        signed_txs.push(signed);
    }
    Ok(signed_txs)
}

/// Collect the base32 transaction ids of a signed group, in order.
fn tx_ids(signed_txs: &[SignedTransaction]) -> Vec<String> {
    signed_txs
        .iter()
        .map(|t| t.transaction_id().0.clone())
        .collect()
}

/// Encode an ABI method call into its application-call transaction (plus
/// any transaction-typed arguments) and append the result to `txs`,
/// recording the method at the app-call's index in `method_map`.
fn process_method_call(
    call: MethodCall,
    txs: &mut Vec<TransactionWithSigner>,
    method_map: &mut HashMap<usize, AbiMethod>,
) -> Result<(), Error> {
    if call.method_args.len() != call.method.args.len() {
        return Err(Error::Msg(format!(
            "incorrect number of arguments were provided: {} != {}",
            call.method_args.len(),
            call.method.args.len()
        )));
    }
    if txs.len() + call.method.get_tx_count() > MAX_ATOMIC_GROUP_SIZE {
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

    for (arg_type, arg_value) in call.method.args.iter().zip(&call.method_args) {
        let mut arg_type = arg_type.clone();

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
                call.sender,
                call.app_id,
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

    let mut encoded_abi_args = vec![call.method.get_selector()?.into()];
    for (method_type, method_arg) in method_types.iter().zip(&method_args) {
        encoded_abi_args.push(method_type.encode(method_arg.clone())?);
    }

    let app_call = TransactionType::ApplicationCallTransaction(ApplicationCallTransaction {
        sender: call.sender,
        app_id: Some(call.app_id),
        on_complete: call.on_complete.clone(),
        accounts: Some(foreign_accounts),
        approval_program: call.approval_program.clone(),
        app_arguments: Some(encoded_abi_args),
        clear_state_program: call.clear_program.clone(),
        foreign_apps: Some(foreign_apps),
        foreign_assets: Some(foreign_assets),
        global_state_schema: call.global_schema.clone(),
        local_state_schema: call.local_schema.clone(),
        extra_pages: call.extra_pages,
        boxes: call.boxes.clone(),
    });

    let sp = &call.suggested_params;
    let tx = Transaction {
        fee: call.fee,
        first_valid: Round(sp.last_round()),
        genesis_hash: sp.genesis_hash(),
        last_valid: Round(sp.last_round() + 1000),
        txn_type: app_call,
        genesis_id: Some(sp.genesis_id().clone()),
        group: None,
        lease: call.lease,
        note: call.note.clone(),
        rekey_to: call.rekey_to,
    };

    txs.append(&mut txs_with_signer);
    txs.push(TransactionWithSigner {
        tx,
        signer: Some(call.signer.clone()),
    });
    method_map.insert(txs.len() - 1, call.method);

    Ok(())
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
    let logs = pending_tx.logs.as_deref().ok_or(Error::MissingReturnLog)?;
    let ret_line = logs.last().ok_or(Error::MissingReturnLog)?;

    let decoded_ret_line: Vec<u8> = BASE64
        .decode(&ret_line.0[..])
        .map_err(|e| Error::Msg(format!("BASE64 Decoding error: {e:?}")))?;

    if !decoded_ret_line.starts_with(&ABI_RETURN_HASH) {
        return Err(Error::MissingReturnLog);
    }

    let abi_encoded = &decoded_ret_line[ABI_RETURN_HASH.len()..decoded_ret_line.len()];
    Ok(match abi_type.decode(abi_encoded) {
        Ok(decoded) => Ok(AbiMethodReturnValue::Some(decoded)),
        Err(e) => Err(AbiReturnDecodeError(format!("{e:?}"))),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use algonaut_core::MicroAlgos;
    use algonaut_crypto::HashDigest;
    use algonaut_transaction::account::Account;
    use algonaut_transaction::builder::{Pay, TransactionParams};

    struct StubParams {
        genesis_id: String,
    }

    impl TransactionParams for StubParams {
        fn last_round(&self) -> u64 {
            1
        }
        fn min_fee(&self) -> u64 {
            1_000
        }
        fn genesis_hash(&self) -> HashDigest {
            HashDigest([0; 32])
        }
        fn genesis_id(&self) -> &String {
            &self.genesis_id
        }
    }

    fn pay(sender: &Account, receiver: Address) -> Transaction {
        let params = StubParams {
            genesis_id: "testnet-v1.0".to_owned(),
        };
        Pay::new(sender.address(), receiver, MicroAlgos(1_000))
            .build(&params)
            .expect("failed to build payment transaction")
    }

    /// Signing a built group whose transactions are authorized by
    /// *different* signers must yield exactly one signed transaction per
    /// input, in input order.
    #[test]
    fn sign_signs_every_tx_with_distinct_signers() {
        let alice = Account::generate();
        let bob = Account::generate();

        let signed = AtomicGroupBuilder::new()
            .add_transaction(TransactionWithSigner::new(
                pay(&alice, bob.address()),
                Arc::new(alice.clone()),
            ))
            .add_transaction(TransactionWithSigner::new(
                pay(&bob, alice.address()),
                Arc::new(bob.clone()),
            ))
            .build()
            .unwrap()
            .sign()
            .unwrap();

        let txs = signed.signed_transactions();
        assert_eq!(
            txs.len(),
            2,
            "every input transaction must be signed exactly once"
        );
        assert_eq!(txs[0].transaction().sender(), alice.address());
        assert_eq!(txs[1].transaction().sender(), bob.address());
    }

    /// `build` rejects a group with no entries.
    #[test]
    fn build_rejects_empty_group() {
        let err = AtomicGroupBuilder::new().build().unwrap_err();
        assert!(matches!(err, Error::EmptyTransactionGroup));
    }
}
