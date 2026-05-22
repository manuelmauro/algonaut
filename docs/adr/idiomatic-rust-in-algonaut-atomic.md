---
id: idiomatic-rust-in-algonaut-atomic
title: Idiomatic Rust in algonaut::atomic
abstract: 'Deep refactoring of the atomic module: eliminate all clippy suppressions, introduce helper structs to replace long argument lists, reduce cloning, box the large enum variant, use From/TryFrom traits for ergonomic ABI arguments, return typed identifiers, and replace all Error::Msg sites with structured variants.'
status: accepted
date: 2026-05-21
deciders: []
tags: [api, ergonomics, atomic, idiomatic-rust, refactoring]
---

# Idiomatic Rust in algonaut::atomic

## Status

Accepted. Follows the accepted
[`atomic-transaction-composer-typestate`](atomic-transaction-composer-typestate.md),
[`atomic-module-layout`](atomic-module-layout.md),
[`method-call-builder`](method-call-builder.md), and
[`async-signer-trait`](async-signer-trait.md) ADRs.

## Context

The `atomic` module has undergone significant refactoring: typestate for the
group lifecycle, a fluent `MethodCallBuilder`, async signers, and a clean
submodule layout. The architecture is sound. What remains is a deeper pass to
align the implementation with idiomatic Rust — eliminating lint suppressions,
restructuring functions that grew organically, reducing unnecessary cloning,
and making the type system carry more weight.

The module currently carries **three `#[allow(clippy::...)]` suppressions**,
**~35 `.clone()` calls** (many avoidable), **eight `Error::Msg(String)` sites**,
and several functions with long argument lists. None of these are bugs, but
together they signal technical debt that makes the code harder to maintain and
extend.

### 1. Three clippy suppressions that should be eliminated

| Location | Lint | Current rationale |
|----------|------|-------------------|
| `method_call.rs:26` | `large_enum_variant` | `AbiArgValue::TxWithSigner` is much larger than `AbiArgValue::AbiValue` |
| `method_call.rs:83` | `new_ret_no_self` | `MethodCall::new()` returns `MethodCallBuilder`, not `MethodCall` |
| `encode.rs:208` | `too_many_arguments` | `add_ref_arg_to_method_call` takes 8 arguments |

Each suppression has a better fix:

- **`large_enum_variant`**: Box the large variant. The transaction-typed ABI
  argument is rare; the cost of one allocation per transaction-typed arg is
  negligible compared to the cost of signing and submitting. Removing the size
  imbalance improves cache behaviour for the common `AbiValue` case.

- **`new_ret_no_self`**: Rename `MethodCall::new()` to `MethodCall::builder()`.
  The builder pattern is idiomatic Rust (`Command::new()` returns `Command`,
  but `Request::builder()` returns `RequestBuilder`). The current name misleads
  readers expecting a constructor.

- **`too_many_arguments`**: Introduce helper structs. The 8-argument function
  passes six mutable references that logically group into two concerns:
  foreign-array accumulation and method-argument encoding.

### 2. Functions with too many parameters

`add_ref_arg_to_method_call` signature:

```rust
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
) -> Result<(), Error>
```

`add_to_foreign_array` has a similar shape. These grew organically from
`process_method_call`, which itself declares six local `Vec`s and threads them
through helpers. The pattern is "struct waiting to be born."

### 3. Excessive cloning in encode.rs and group.rs

Examples from `encode.rs`:

```rust
let mut arg_type = arg_type.clone();           // line 65
method_type.encode(method_arg.clone())?        // line 101
on_complete: call.on_complete.clone(),         // line 107
approval_program: call.approval_program.clone(), // line 109
// ... 10+ more in this file
```

Many of these clones exist because `MethodCall` is borrowed but its fields are
moved into `Transaction`. Since `MethodCall` is consumed by
`AtomicGroupBuilder::add_method_call` → `process_method_call`, the fields could
be moved directly instead of cloned.

Examples from `group.rs`:

```rust
let pending_tx = (*txn_result.txn_result).clone();  // line 237
let mut current_pending_tx = pending_tx.clone();    // line 296
tx_info: pending_tx.clone(),                        // line 309
```

The execute loop clones `PendingTransactionResponse` multiple times. A single
owned value threaded through, or references where possible, would eliminate
the copies.

### 4. `AbiArgValue` helpers use `Option` instead of `TryFrom`

```rust
impl AbiArgValue {
    pub(super) fn address(&self) -> Option<Address> { ... }
    pub(super) fn int(&self) -> Option<BigUint> { ... }
}
```

These are "try to extract X" methods that return `None` on type mismatch.
Idiomatic Rust uses `TryFrom` for fallible conversions, which integrates with
`?` and error propagation.

### 5. Transaction IDs returned as `String`, not `TxId`

`ExecuteOutcome` and `SimulateOutcome` expose `tx_ids: Vec<String>`, while
`AbiMethodResult` carries `tx_id: TxId`. The helper function explicitly
extracts the inner string:

```rust
pub(super) fn tx_ids(signed_txs: &[SignedTransaction]) -> Vec<String> {
    signed_txs.iter().map(|t| t.transaction_id().0.clone()).collect()
}
```

This discards type information that callers may need to recover.

### 6. `Error::Msg(String)` sites throughout

Eight sites in `encode.rs`, `outcome.rs`, and `signing.rs` use `Error::Msg`
or `Error::Internal` with string messages. The
[`structured-errors`](structured-errors.md) ADR and
[`ideal-type-safe-ergonomic-api`](ideal-type-safe-ergonomic-api.md) D8 mandate
typed variants for matchable failure modes.

### 7. `Option<Vec<T>>` patterns

`MethodCall` stores `boxes: Option<Vec<BoxReference>>` and similar. When
"absent" and "empty" are semantically equivalent, `Vec<T>` is simpler: no
`.as_deref().unwrap_or(&[])` ceremony.

### 8. `SuggestedParams` cloned into `MethodCall`

```rust
suggested_params: params.clone(),
```

`SuggestedParams` contains heap-allocated strings. The clone is unnecessary if
we store only the primitives we actually use.

## Decision

### 1. Box `AbiArgValue::TxWithSigner`

```rust
pub enum AbiArgValue {
    TxWithSigner(Box<TransactionWithSigner>),
    AbiValue(AbiValue),
}
```

Remove the `#[allow(clippy::large_enum_variant)]`. The `From` impl boxes
transparently:

```rust
impl From<TransactionWithSigner> for AbiArgValue {
    fn from(t: TransactionWithSigner) -> Self {
        Self::TxWithSigner(Box::new(t))
    }
}
```

### 2. Rename `MethodCall::new()` to `MethodCall::builder()`

```rust
impl MethodCall {
    pub fn builder(
        app_id: AppId,
        method: AbiMethod,
        sender: Address,
        signer: Arc<dyn Signer>,
    ) -> MethodCallBuilder { ... }
}
```

Remove the `#[allow(clippy::new_ret_no_self)]`. The name now matches the return
type, following `http::Request::builder()`, `tokio::runtime::Builder::new_*()`,
etc.

### 3. Introduce `ForeignArrays` and `EncodedArgs` structs

```rust
/// Accumulates foreign-array references during ABI method encoding.
#[derive(Default)]
struct ForeignArrays {
    accounts: Vec<Address>,
    assets: Vec<AssetId>,
    apps: Vec<AppId>,
}

impl ForeignArrays {
    /// Add a reference argument and return its index.
    fn add_ref(
        &mut self,
        arg_type: &ReferenceArgType,
        arg_value: &AbiArgValue,
        sender: Address,
        app_id: AppId,
    ) -> Result<usize, Error> { ... }
}

/// Accumulates encoded ABI arguments during method encoding.
#[derive(Default)]
struct EncodedArgs {
    types: Vec<AbiType>,
    values: Vec<AbiValue>,
}

impl EncodedArgs {
    fn push(&mut self, ty: AbiType, val: AbiValue) { ... }
    fn push_ref_index(&mut self, index: usize) -> Result<(), Error> { ... }
    fn wrap_overflow(&mut self) -> Result<(), Error> { ... }
    fn encode(self, selector: Vec<u8>) -> Result<Vec<Vec<u8>>, Error> { ... }
}
```

`process_method_call` becomes:

```rust
pub(super) fn process_method_call(
    call: MethodCall,
    txs: &mut Vec<TransactionWithSigner>,
    method_map: &mut HashMap<usize, AbiMethod>,
) -> Result<(), Error> {
    // validation...

    let mut foreign = ForeignArrays::default();
    let mut args = EncodedArgs::default();
    let mut tx_args = Vec::new();

    for (arg_spec, arg_value) in call.method.args.iter().zip(&call.method_args) {
        match arg_spec.type_()? {
            AbiArgType::Tx(ty) => tx_args.push(extract_tx_arg(arg_value, ty)?),
            AbiArgType::Ref(ty) => {
                let idx = foreign.add_ref(&ty, arg_value, call.sender, call.app_id)?;
                args.push_ref_index(idx)?;
            }
            AbiArgType::AbiObj(ty) => args.push_value(&ty, arg_value)?,
        }
    }

    args.wrap_overflow()?;
    let app_arguments = args.encode(call.method.get_selector()?)?;

    // build transaction from `call`, `foreign`, `app_arguments`...
}
```

Remove the `#[allow(clippy::too_many_arguments)]`.

### 4. Consume `MethodCall` fields instead of cloning

`process_method_call` takes `call: MethodCall` by value. Change:

```rust
on_complete: call.on_complete.clone(),
approval_program: call.approval_program.clone(),
```

to:

```rust
on_complete: call.on_complete,
approval_program: call.approval_program,
```

Where a field is used multiple times before the final move, destructure
`MethodCall` at the start:

```rust
let MethodCall {
    app_id,
    method,
    method_args,
    sender,
    signer,
    on_complete,
    approval_program,
    clear_program,
    // ...
} = call;
```

Then each field is moved exactly once.

### 5. Implement `TryFrom` for `AbiArgValue` extractions

```rust
impl TryFrom<&AbiArgValue> for Address {
    type Error = Error;
    fn try_from(v: &AbiArgValue) -> Result<Address, Error> {
        match v {
            AbiArgValue::AbiValue(AbiValue::Address(a)) => Ok(*a),
            _ => Err(Error::InvalidAbiArgument { ... }),
        }
    }
}

impl TryFrom<&AbiArgValue> for BigUint {
    type Error = Error;
    fn try_from(v: &AbiArgValue) -> Result<BigUint, Error> { ... }
}

impl<'a> TryFrom<&'a AbiArgValue> for &'a TransactionWithSigner {
    type Error = Error;
    fn try_from(v: &'a AbiArgValue) -> Result<&'a TransactionWithSigner, Error> { ... }
}
```

Usage becomes:

```rust
let address: Address = arg_value.try_into()?;
```

Delete the `address()` and `int()` helper methods.

### 6. Return `Vec<TxId>` instead of `Vec<String>`

```rust
pub(super) fn tx_ids(signed_txs: &[SignedTransaction]) -> Vec<TxId> {
    signed_txs.iter().map(|t| t.transaction_id().clone()).collect()
}

pub struct ExecuteOutcome {
    pub tx_ids: Vec<TxId>,
    // ...
}

pub struct SimulateOutcome {
    pub tx_ids: Vec<TxId>,
    // ...
}
```

### 7. Replace all `Error::Msg` sites with structured variants

Add to `Error`:

```rust
AbiArgumentCountMismatch { expected: usize, actual: usize },
TransactionAlreadyGrouped,
TransactionTypeMismatch { expected: String, actual: String },
ExpectedTransactionArgument,
InvalidAbiArgument { expected: &'static str, actual: String },
Base64DecodeError { source: data_encoding::DecodeError },
InternalSigningIncomplete { index: usize },
```

Update each site in `encode.rs`, `outcome.rs`, and `signing.rs`.

### 8. Use `Vec<T>` instead of `Option<Vec<T>>` in builders

In `MethodCall` and `MethodCallBuilder`:

```rust
pub(super) boxes: Vec<BoxReference>,
pub(super) note: Vec<u8>,  // not Option<Vec<u8>>
```

Default to empty; convert to `Option` only at encoding time:

```rust
boxes: if boxes.is_empty() { None } else { Some(boxes) },
```

### 9. Store transaction params as primitives in `MethodCall`

Replace:

```rust
pub(super) suggested_params: SuggestedParams,
```

with:

```rust
pub(super) first_valid: Round,
pub(super) last_valid: Round,
pub(super) genesis_hash: HashDigest,
pub(super) genesis_id: String,
```

`MethodCallBuilder::build` extracts and stores these directly; no
`params.clone()`.

### 10. Flatten `AbiArgValue` with `From` impls

```rust
impl From<AbiValue> for AbiArgValue {
    fn from(v: AbiValue) -> Self {
        Self::AbiValue(v)
    }
}
```

Change `MethodCallBuilder::args` signature:

```rust
pub fn args(mut self, args: impl IntoIterator<Item = impl Into<AbiArgValue>>) -> Self
```

### 11. Add `From` impls for common `AbiValue` types

The ARC-4 ABI supports `uint8` through `uint512`, requiring `BigUint` for the
general case. But the common case is small integers, and the current ceremony
is excessive:

```rust
AbiValue::Int(BigUint::from(2u64))
```

Add `From` impls for native integer types:

```rust
impl From<u64> for AbiValue {
    fn from(n: u64) -> Self {
        AbiValue::Int(BigUint::from(n))
    }
}

impl From<u128> for AbiValue {
    fn from(n: u128) -> Self {
        AbiValue::Int(BigUint::from(n))
    }
}

// Also for Address, bool, String, &str, Vec<u8> (bytes)
impl From<Address> for AbiValue {
    fn from(a: Address) -> Self {
        AbiValue::Address(a)
    }
}

impl From<bool> for AbiValue {
    fn from(b: bool) -> Self {
        AbiValue::Bool(b)
    }
}

impl From<&str> for AbiValue {
    fn from(s: &str) -> Self {
        AbiValue::String(s.to_owned())
    }
}
```

Add transitive `From` impls for `AbiArgValue` to complete the chain:

```rust
impl<T: Into<AbiValue>> From<T> for AbiArgValue {
    fn from(v: T) -> Self {
        Self::AbiValue(v.into())
    }
}
```

Call sites become:

```rust
.args([2u64, 3u64])
```

### 12. Reduce cloning in the execute loop

Refactor `SignedAtomicGroup::execute` to avoid repeated `pending_tx.clone()`:

```rust
pub async fn execute(self, algod: &Algod) -> Result<ExecuteOutcome, Error> {
    algod.send_txns(&self.signed_txs).await?;

    let first_method_idx = self.method_map.keys().min().copied().unwrap_or(0);
    let first_tx_id = self.signed_txs[first_method_idx].transaction_id();
    let confirmed = poll_until_confirmed(algod, first_tx_id).await?;
    let confirmed_round = confirmed.confirmed_round;

    let method_results = self.decode_method_results(algod, &confirmed).await?;

    Ok(ExecuteOutcome {
        confirmed_round,
        tx_ids: tx_ids(&self.signed_txs),
        method_results,
    })
}

impl SignedAtomicGroup {
    async fn decode_method_results(
        &self,
        algod: &Algod,
        first_confirmed: &PendingTransactionResponse,
    ) -> Result<Vec<AbiMethodResult>, Error> {
        // fetch other pending txns only when needed, avoid cloning
    }
}
```

## Consequences

- **Zero clippy suppressions.** The module passes `clippy::pedantic` without
  exceptions. Maintenance burden decreases; new contributors see clean code.

- **Helper structs clarify intent.** `ForeignArrays` and `EncodedArgs` make
  `process_method_call` readable. Each struct's methods are testable in
  isolation.

- **Moves replace clones.** `MethodCall` fields are moved, not cloned. The
  execute loop avoids redundant `PendingTransactionResponse` copies. Fewer
  allocations, clearer ownership.

- **`TryFrom` integrates with `?`.** Extraction failures propagate naturally;
  no manual `match` + `ok_or` at each site.

- **Typed transaction IDs.** `Vec<TxId>` throughout. Callers needing strings
  call `.to_string()`.

- **Structured errors.** All matchable failure modes have typed variants.
  `Error::Msg` is reserved for truly unstructured cases (none remain in
  `atomic`).

- **Simpler builders.** `Vec<T>` defaults eliminate `Option` ceremony. The
  `builder()` name matches the return type.

- **No `SuggestedParams` clone.** Primitives are copied; no heap allocation.

- **Minimal ABI argument ceremony.** Native types convert directly to
  `AbiArgValue` via `From` impls. `.args([2u64, 3u64])` replaces
  `.args(vec![AbiArgValue::AbiValue(AbiValue::Int(BigUint::from(2u64))), ...])`.

- **Consistency with ecosystem.** `From`/`TryFrom`, builder naming, structured
  errors — all match Rust community conventions.
