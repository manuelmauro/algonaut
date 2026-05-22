//! Fluent builder for ABI method calls added to an atomic group via
//! [`AtomicGroupBuilder::add_method_call`](super::AtomicGroupBuilder::add_method_call).
//!
//! Replaces the previous 18-field `AddMethodCallParams` struct. The
//! call-context inputs (`app_id`, `sender`, `signer`) are positional on
//! [`MethodCall::builder`]; the method and its arguments arrive together via
//! [`MethodCallBuilder::invoke`] (typically `.invoke(abi_call!(…))`), and
//! everything else is an optional setter, so the common call carries no
//! `None`s.

use std::sync::Arc;

use algonaut_abi::{
    MethodInvocation, abi_error::AbiError, abi_interactions::AbiMethod, abi_type::AbiValue,
};
use algonaut_core::{Address, AppId, CompiledTeal, MicroAlgos, Round};
use algonaut_crypto::HashDigest;
use algonaut_model::client_types::SuggestedParams;
use algonaut_transaction::{
    Signer,
    transaction::{ApplicationCallOnComplete, BoxReference, StateSchema},
};
use num_bigint::BigUint;
use num_traits::ToPrimitive;

use crate::Error;

use super::TransactionWithSigner;

/// An argument to an ABI [`MethodCall`]. Either a transaction-typed
/// argument (which contributes its own slot to the group) or a plain ABI
/// value.
///
/// Build one with `.into()`: native types (`u64`, `bool`, [`Address`],
/// `&str`, `Vec<u8>`, …) and [`AbiValue`] convert into
/// [`AbiArgValue::AbiValue`], and a [`TransactionWithSigner`] converts into
/// [`AbiArgValue::TxWithSigner`]. The transaction-typed variant is boxed so
/// the enum stays small for the common plain-value case.
#[derive(Debug, Clone)]
pub enum AbiArgValue {
    TxWithSigner(Box<TransactionWithSigner>),
    AbiValue(AbiValue),
}

impl From<AbiValue> for AbiArgValue {
    fn from(v: AbiValue) -> Self {
        Self::AbiValue(v)
    }
}

impl From<TransactionWithSigner> for AbiArgValue {
    fn from(t: TransactionWithSigner) -> Self {
        Self::TxWithSigner(Box::new(t))
    }
}

// Native-type conveniences mirror `AbiValue`'s `From` impls so that
// `Invocation::new(method, [2u64, 3u64])` works without spelling out
// `AbiValue`. A blanket `impl<T: Into<AbiValue>> From<T> for AbiArgValue`
// would collide with both the reflexive `From<AbiArgValue>` and the
// `TransactionWithSigner` impl above, so the conversions are enumerated.
macro_rules! abi_arg_value_from {
    ($($t:ty),* $(,)?) => {$(
        impl From<$t> for AbiArgValue {
            fn from(v: $t) -> Self {
                Self::AbiValue(AbiValue::from(v))
            }
        }
    )*};
}

abi_arg_value_from!(u64, u128, bool, Address, &str, String, Vec<u8>);

/// Extract an [`Address`] from an account-typed ABI argument. Replaces the
/// old `AbiArgValue::address(&self) -> Option<Address>` helper so the
/// fallible extraction integrates with `?`.
impl TryFrom<&AbiArgValue> for Address {
    type Error = Error;

    fn try_from(value: &AbiArgValue) -> Result<Self, Self::Error> {
        match value {
            AbiArgValue::AbiValue(AbiValue::Address(address)) => Ok(*address),
            other => Err(Error::InvalidAbiArgument {
                expected: "address",
                actual: format!("{other:?}"),
            }),
        }
    }
}

/// Extract a [`BigUint`] from an integer-typed ABI argument. Replaces the
/// old `AbiArgValue::int(&self) -> Option<BigUint>` helper.
impl TryFrom<&AbiArgValue> for BigUint {
    type Error = Error;

    fn try_from(value: &AbiArgValue) -> Result<Self, Self::Error> {
        match value {
            AbiArgValue::AbiValue(AbiValue::Int(int)) => Ok(int.clone()),
            other => Err(Error::InvalidAbiArgument {
                expected: "uint",
                actual: format!("{other:?}"),
            }),
        }
    }
}

/// Extract a `u64` from an integer-typed ABI argument — used for the asset-
/// and application-id foreign-array references, which are `u64` on the
/// wire. Errors if the value is not an integer, or if it overflows `u64`.
impl TryFrom<&AbiArgValue> for u64 {
    type Error = Error;

    fn try_from(value: &AbiArgValue) -> Result<Self, Self::Error> {
        let int = BigUint::try_from(value)?;
        int.to_u64().ok_or_else(|| {
            Error::from(AbiError::ValueOutOfRange {
                abi_type: "uint64".to_owned(),
                reason: format!("value {int} exceeds u64 capacity"),
            })
        })
    }
}

/// A method paired with its arguments — what [`MethodCallBuilder::invoke`]
/// consumes. Arguments only make sense relative to a method, so they arrive
/// together rather than through a separate `.args(...)` setter.
///
/// Two ways to build one:
///
/// - **Compile-time checked** — the [`abi_call!`](algonaut_abi::abi_call) macro
///   produces an [`algonaut_abi::MethodInvocation`] that converts in:
///   `.invoke(abi_call!("add(uint64,uint64)uint64", 2u64, 3u64))`. The
///   signature and every argument's type are checked by `cargo build`.
/// - **Dynamic** — for signatures sourced at run time (app-spec JSON, user
///   input), or for transaction- and reference-typed arguments, use
///   [`Invocation::new`] with an [`AbiMethod`] from
///   [`AbiMethod::from_signature`] and arguments as [`AbiArgValue`]s.
#[derive(Clone, Debug)]
pub struct Invocation {
    method: AbiMethod,
    args: Vec<AbiArgValue>,
}

impl Invocation {
    /// Pair a method with arguments supplied at run time. Arguments convert
    /// into [`AbiArgValue`], so native types, [`AbiValue`]s, and
    /// transaction-typed [`TransactionWithSigner`](super::TransactionWithSigner)
    /// arguments are all accepted — the path for app-spec-loaded contracts and
    /// transaction/reference arguments the `abi_call!` macro does not yet bind.
    pub fn new(method: AbiMethod, args: impl IntoIterator<Item = impl Into<AbiArgValue>>) -> Self {
        Invocation {
            method,
            args: args.into_iter().map(Into::into).collect(),
        }
    }
}

/// The compile-time-checked [`abi_call!`](algonaut_abi::abi_call) output drops
/// straight into [`MethodCallBuilder::invoke`]: its already-encoded
/// [`AbiValue`]s become plain value arguments.
impl From<MethodInvocation> for Invocation {
    fn from(invocation: MethodInvocation) -> Self {
        let (method, values) = invocation.into_parts();
        Invocation {
            method,
            args: values.into_iter().map(AbiArgValue::AbiValue).collect(),
        }
    }
}

/// A fully-built ABI method call, ready to be handed to
/// [`AtomicGroupBuilder::add_method_call`](super::AtomicGroupBuilder::add_method_call).
///
/// Construct one with the fluent [`MethodCallBuilder`] returned by
/// [`MethodCall::builder`].
#[derive(Clone, Debug)]
pub struct MethodCall {
    pub(super) app_id: AppId,
    pub(super) method: AbiMethod,
    pub(super) method_args: Vec<AbiArgValue>,
    pub(super) fee: MicroAlgos,
    pub(super) sender: Address,
    // The transaction-validity window and genesis identifiers resolved from
    // the suggested params at build time. Storing the primitives we use
    // avoids cloning the whole `SuggestedParams` (and its heap strings).
    pub(super) first_valid: Round,
    pub(super) last_valid: Round,
    pub(super) genesis_hash: HashDigest,
    pub(super) genesis_id: String,
    pub(super) on_complete: ApplicationCallOnComplete,
    pub(super) approval_program: Option<CompiledTeal>,
    pub(super) clear_program: Option<CompiledTeal>,
    pub(super) global_schema: Option<StateSchema>,
    pub(super) local_schema: Option<StateSchema>,
    pub(super) extra_pages: u32,
    pub(super) note: Vec<u8>,
    pub(super) lease: Option<HashDigest>,
    pub(super) rekey_to: Option<Address>,
    pub(super) signer: Arc<dyn Signer>,
    pub(super) boxes: Vec<BoxReference>,
}

impl MethodCall {
    /// Start a new method-call builder.
    ///
    /// `sender` is required because the [`Signer`] trait does not
    /// expose a single sender address (e.g. a multisig signer's
    /// underlying address is the multisig address, not necessarily the
    /// sender of any one transaction). Pass the address the call
    /// should be sent from explicitly.
    ///
    /// Named `builder` (not `new`) because it returns a
    /// [`MethodCallBuilder`], following `http::Request::builder()` and the
    /// rest of the ecosystem.
    ///
    /// The method and its arguments are supplied together via
    /// [`MethodCallBuilder::invoke`] — typically `.invoke(abi_call!(…))` — so
    /// they are not positional here.
    pub fn builder(app_id: AppId, sender: Address, signer: Arc<dyn Signer>) -> MethodCallBuilder {
        MethodCallBuilder {
            app_id,
            method: None,
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
            note: Vec::new(),
            lease: None,
            rekey_to: None,
            boxes: Vec::new(),
        }
    }
}

/// Fluent builder produced by [`MethodCall::builder`].
///
/// All setters take ownership and return `self`. Call
/// [`MethodCallBuilder::build`] with the network's suggested params to
/// finalise the call into a [`MethodCall`].
pub struct MethodCallBuilder {
    app_id: AppId,
    /// Set by [`MethodCallBuilder::invoke`]; required before [`build`](Self::build).
    method: Option<AbiMethod>,
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
    note: Vec<u8>,
    lease: Option<HashDigest>,
    rekey_to: Option<Address>,
    boxes: Vec<BoxReference>,
}

impl MethodCallBuilder {
    /// Supply the method and its arguments as one checked [`Invocation`].
    ///
    /// The common form is `.invoke(abi_call!("add(uint64,uint64)uint64", 2u64,
    /// 3u64))`: the macro validates the signature and type-checks the
    /// arguments at compile time. For runtime-sourced signatures, pass
    /// `Invocation::new(method, args)`. This is required before
    /// [`build`](Self::build).
    pub fn invoke(mut self, invocation: impl Into<Invocation>) -> Self {
        let Invocation { method, args } = invocation.into();
        self.method = Some(method);
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

    /// Transaction note. Defaults to empty (no note).
    pub fn note(mut self, note: Vec<u8>) -> Self {
        self.note = note;
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

    /// Box references this call is permitted to access. Defaults to empty.
    pub fn boxes(mut self, boxes: Vec<BoxReference>) -> Self {
        self.boxes = boxes;
        self
    }

    /// Finalise the builder with the network's current suggested
    /// parameters, producing a [`MethodCall`] that can be passed to
    /// [`AtomicGroupBuilder::add_method_call`](super::AtomicGroupBuilder::add_method_call).
    ///
    /// The fee defaults to `params.min_fee` if no [`fee`](Self::fee)
    /// override was supplied. Only the validity-window and genesis
    /// primitives are read out of `params`; the struct is not cloned.
    pub fn build(self, params: &SuggestedParams) -> MethodCall {
        let fee = self.fee.unwrap_or(params.min_fee);
        let method = self
            .method
            .expect("MethodCall: call `.invoke(...)` before `.build(...)`");
        MethodCall {
            app_id: self.app_id,
            method,
            method_args: self.method_args,
            fee,
            sender: self.sender,
            first_valid: params.last_round,
            last_valid: Round(params.last_round.0 + 1000),
            genesis_hash: params.genesis_hash,
            genesis_id: params.genesis_id.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use algonaut_abi::abi_call;

    /// The `abi_call!` macro expands to a checked invocation that converts into
    /// the builder's [`Invocation`]: the right method and the encoded value
    /// arguments, in order.
    #[test]
    fn abi_call_macro_produces_checked_invocation() {
        let invocation: Invocation = abi_call!("add(uint64,uint64)uint64", 2u64, 3u64).into();

        assert_eq!(invocation.method.name, "add");
        assert_eq!(invocation.method.args.len(), 2);
        assert_eq!(invocation.args.len(), 2);

        let values: Vec<&AbiValue> = invocation
            .args
            .iter()
            .map(|a| match a {
                AbiArgValue::AbiValue(v) => v,
                AbiArgValue::TxWithSigner(_) => panic!("expected plain ABI values"),
            })
            .collect();
        assert_eq!(values[0], &AbiValue::from(2u64));
        assert_eq!(values[1], &AbiValue::from(3u64));
    }

    /// A widening native type (`u32` where `uint64` is expected) is accepted,
    /// matching the `format!`-style "value fits" rule the `AbiArg` impls encode.
    #[test]
    fn abi_call_macro_allows_widening() {
        let invocation: Invocation = abi_call!("add(uint64,uint64)uint64", 2u32, 3u8).into();
        assert_eq!(invocation.args.len(), 2);
    }

    /// `abi_call!` also checks `bool`, `string`, and `byte[]` slots.
    #[test]
    fn abi_call_macro_checks_non_integer_value_types() {
        let invocation: Invocation =
            abi_call!("f(bool,string,byte[])void", true, "hi", vec![1u8, 2u8]).into();
        assert_eq!(invocation.method.name, "f");
        assert_eq!(invocation.args.len(), 3);
    }
}
