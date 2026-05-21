//! Atomic transaction groups: bundle transactions and ABI method calls,
//! then sign, simulate, submit, or execute them as one all-or-nothing unit.
//!
//! The entry point is [`AtomicGroupBuilder`]. It advances through a typestate
//! chain — [`AtomicGroupBuilder`] → [`UnsignedAtomicGroup`] →
//! [`SignedAtomicGroup`] — so a call that doesn't make sense in a given state
//! (signing twice, submitting before signing) doesn't compile. See the
//! `atomic-transaction-composer-typestate` ADR for the rationale.
//!
//! ```ignore
//! let outcome = AtomicGroupBuilder::new()
//!     .add_method_call(call)
//!     .build()?
//!     .sign()
//!     .await?
//!     .execute(&algod)
//!     .await?;
//! ```
//!
//! The module is organized into:
//! - `group` — the public typestate chain and its transition methods;
//! - `method_call` — the [`MethodCall`] fluent builder and its argument type;
//! - `encode` — encoding an ABI method call into an application-call transaction;
//! - `outcome` — the result types and ABI return-value decoding;
//! - `signing` — signer orchestration and finality polling.

mod encode;
mod group;
mod method_call;
mod outcome;
mod signing;

pub use group::{
    AtomicGroupBuilder, SignedAtomicGroup, TransactionWithSigner, UnsignedAtomicGroup,
};
pub use method_call::{AbiArgValue, MethodCall, MethodCallBuilder};
pub use outcome::{
    AbiMethodResult, AbiMethodReturnValue, AbiReturnDecodeError, ExecuteOutcome, SimulateOutcome,
};

/// The maximum size of an atomic transaction group.
const MAX_ATOMIC_GROUP_SIZE: usize = 16;
