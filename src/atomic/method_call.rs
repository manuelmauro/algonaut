//! Fluent builder for ABI method calls added to an atomic group via
//! [`AtomicGroupBuilder::add_method_call`](super::AtomicGroupBuilder::add_method_call).
//!
//! Replaces the previous 18-field `AddMethodCallParams` struct. Only the
//! four genuinely required inputs are positional on
//! [`MethodCall::new`]; everything else is an optional setter, so the
//! common call carries no `None`s.

use std::sync::Arc;

use algonaut_abi::{abi_interactions::AbiMethod, abi_type::AbiValue};
use algonaut_core::{Address, AppId, CompiledTeal, MicroAlgos};
use algonaut_crypto::HashDigest;
use algonaut_model::client_types::SuggestedParams;
use algonaut_transaction::{
    Signer,
    transaction::{ApplicationCallOnComplete, BoxReference, StateSchema},
};
use num_bigint::BigUint;

use super::TransactionWithSigner;

/// An argument to an ABI [`MethodCall`]. Either a transaction-typed
/// argument (which contributes its own slot to the group) or a plain ABI
/// value.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum AbiArgValue {
    TxWithSigner(TransactionWithSigner),
    AbiValue(AbiValue),
}

impl AbiArgValue {
    pub(super) fn address(&self) -> Option<Address> {
        match self {
            AbiArgValue::AbiValue(AbiValue::Address(address)) => Some(*address),
            _ => None,
        }
    }

    pub(super) fn int(&self) -> Option<BigUint> {
        match self {
            AbiArgValue::AbiValue(AbiValue::Int(int)) => Some(int.clone()),
            _ => None,
        }
    }
}

/// A fully-built ABI method call, ready to be handed to
/// [`AtomicGroupBuilder::add_method_call`](super::AtomicGroupBuilder::add_method_call).
///
/// Construct one with the fluent [`MethodCallBuilder`] returned by
/// [`MethodCall::new`].
#[derive(Clone, Debug)]
pub struct MethodCall {
    pub(super) app_id: AppId,
    pub(super) method: AbiMethod,
    pub(super) method_args: Vec<AbiArgValue>,
    pub(super) fee: MicroAlgos,
    pub(super) sender: Address,
    pub(super) suggested_params: SuggestedParams,
    pub(super) on_complete: ApplicationCallOnComplete,
    pub(super) approval_program: Option<CompiledTeal>,
    pub(super) clear_program: Option<CompiledTeal>,
    pub(super) global_schema: Option<StateSchema>,
    pub(super) local_schema: Option<StateSchema>,
    pub(super) extra_pages: u32,
    pub(super) note: Option<Vec<u8>>,
    pub(super) lease: Option<HashDigest>,
    pub(super) rekey_to: Option<Address>,
    pub(super) signer: Arc<dyn Signer>,
    pub(super) boxes: Option<Vec<BoxReference>>,
}

impl MethodCall {
    /// Start a new method-call builder.
    ///
    /// `sender` is required because the [`Signer`] trait does not
    /// expose a single sender address (e.g. a multisig signer's
    /// underlying address is the multisig address, not necessarily the
    /// sender of any one transaction). Pass the address the call
    /// should be sent from explicitly.
    #[allow(clippy::new_ret_no_self)] // intentional: builder API
    pub fn new(
        app_id: AppId,
        method: AbiMethod,
        sender: Address,
        signer: Arc<dyn Signer>,
    ) -> MethodCallBuilder {
        MethodCallBuilder {
            app_id,
            method,
            sender,
            signer,
            method_args: Vec::new(),
            fee: None,
            on_complete: ApplicationCallOnComplete::NoOp,
            approval_program: None,
            clear_program: None,
            global_schema: None,
            local_schema: None,
            extra_pages: 0,
            note: None,
            lease: None,
            rekey_to: None,
            boxes: None,
        }
    }
}

/// Fluent builder produced by [`MethodCall::new`].
///
/// All setters take ownership and return `self`. Call
/// [`MethodCallBuilder::build`] with the network's suggested params to
/// finalise the call into a [`MethodCall`].
pub struct MethodCallBuilder {
    app_id: AppId,
    method: AbiMethod,
    sender: Address,
    signer: Arc<dyn Signer>,
    method_args: Vec<AbiArgValue>,
    fee: Option<MicroAlgos>,
    on_complete: ApplicationCallOnComplete,
    approval_program: Option<CompiledTeal>,
    clear_program: Option<CompiledTeal>,
    global_schema: Option<StateSchema>,
    local_schema: Option<StateSchema>,
    extra_pages: u32,
    note: Option<Vec<u8>>,
    lease: Option<HashDigest>,
    rekey_to: Option<Address>,
    boxes: Option<Vec<BoxReference>>,
}

impl MethodCallBuilder {
    /// ABI method arguments, in the order declared by the method
    /// signature. If the method takes no arguments, omit this setter.
    pub fn args(mut self, args: Vec<AbiArgValue>) -> Self {
        self.method_args = args;
        self
    }

    /// Override the transaction fee. Defaults to `params.min_fee`
    /// when [`build`](Self::build) is called.
    pub fn fee(mut self, fee: MicroAlgos) -> Self {
        self.fee = Some(fee);
        self
    }

    /// On-complete action for the application call. Defaults to
    /// [`ApplicationCallOnComplete::NoOp`].
    pub fn on_complete(mut self, on_complete: ApplicationCallOnComplete) -> Self {
        self.on_complete = on_complete;
        self
    }

    /// Approval program. Only required for application-creation calls
    /// (`app_id == AppId(0)`) or `UpdateApplication` on-complete.
    pub fn approval_program(mut self, approval: CompiledTeal) -> Self {
        self.approval_program = Some(approval);
        self
    }

    /// Clear-state program. Required for creation/update calls.
    pub fn clear_program(mut self, clear: CompiledTeal) -> Self {
        self.clear_program = Some(clear);
        self
    }

    /// Global state schema. Only meaningful for creation calls.
    pub fn global_schema(mut self, schema: StateSchema) -> Self {
        self.global_schema = Some(schema);
        self
    }

    /// Local state schema. Only meaningful for creation calls.
    pub fn local_schema(mut self, schema: StateSchema) -> Self {
        self.local_schema = Some(schema);
        self
    }

    /// Number of extra program pages to allocate. Only meaningful for
    /// creation calls. Defaults to 0.
    pub fn extra_pages(mut self, pages: u32) -> Self {
        self.extra_pages = pages;
        self
    }

    /// Transaction note.
    pub fn note(mut self, note: Vec<u8>) -> Self {
        self.note = Some(note);
        self
    }

    /// Transaction lease.
    pub fn lease(mut self, lease: HashDigest) -> Self {
        self.lease = Some(lease);
        self
    }

    /// Rekey the sender to this address at the conclusion of the call.
    pub fn rekey_to(mut self, rekey_to: Address) -> Self {
        self.rekey_to = Some(rekey_to);
        self
    }

    /// Box references this call is permitted to access.
    pub fn boxes(mut self, boxes: Vec<BoxReference>) -> Self {
        self.boxes = Some(boxes);
        self
    }

    /// Finalise the builder with the network's current suggested
    /// parameters, producing a [`MethodCall`] that can be passed to
    /// [`AtomicGroupBuilder::add_method_call`](super::AtomicGroupBuilder::add_method_call).
    ///
    /// The fee defaults to `params.min_fee` if no [`fee`](Self::fee)
    /// override was supplied.
    pub fn build(self, params: &SuggestedParams) -> MethodCall {
        let fee = self.fee.unwrap_or(params.min_fee);
        MethodCall {
            app_id: self.app_id,
            method: self.method,
            method_args: self.method_args,
            fee,
            sender: self.sender,
            suggested_params: params.clone(),
            on_complete: self.on_complete,
            approval_program: self.approval_program,
            clear_program: self.clear_program,
            global_schema: self.global_schema,
            local_schema: self.local_schema,
            extra_pages: self.extra_pages,
            note: self.note,
            lease: self.lease,
            rekey_to: self.rekey_to,
            signer: self.signer,
            boxes: self.boxes,
        }
    }
}
