---
id: atomic-transaction-composer-typestate
title: Atomic transaction composer as typestate, not status enum
abstract: Split `AtomicTransactionComposer` into a chain of state-specific types — `GroupBuilder` → `UnsignedGroup` → `SignedGroup` — replacing the runtime-checked `AtomicTransactionComposerStatus` enum with compile-time enforcement. Calls that don't make sense in a given state (signing twice, submitting before signing, adding transactions after `build_group`) stop compiling instead of returning `Err(Error::ComposerStatusInvalid)`. Eighth sub-ADR addressing the composer-specific friction not covered by D1–D9 of the ideal-type-safe-ergonomic-api index.
status: proposed
date: 2026-05-20
deciders: []
tags: [api, ergonomics, type-safety, atomic-transaction-composer]
---

# Atomic transaction composer as typestate, not status enum

## Status

Proposed. Refines and complements the composer-touching items
([D2](pending-submission.md), [D6](method-call-builder.md),
[D7](signer-trait.md)) of
[`ideal-type-safe-ergonomic-api`](ideal-type-safe-ergonomic-api.md).

## Context

`AtomicTransactionComposer` is the single most stateful API in the
SDK. Its public surface today, after the D-series work landed:

```rust
pub struct AtomicTransactionComposer { /* status, txs, signed_txs, method_map */ }

impl AtomicTransactionComposer {
    pub fn add_transaction(&mut self, t: TransactionWithSigner) -> Result<(), Error>;
    pub fn add_method_call(&mut self, m: MethodCall)            -> Result<(), Error>;
    pub fn build_group(&mut self)                                -> Result<Vec<TransactionWithSigner>, Error>;
    pub fn gather_signatures(&mut self)                          -> Result<Vec<SignedTransaction>, Error>;
    pub async fn submit(&mut self,   &Algod) -> Result<Vec<String>, Error>;
    pub async fn execute(&mut self,  &Algod) -> Result<ExecuteResult, Error>;
    pub async fn simulate(&mut self, &Algod) -> Result<AtcSimulateResult, Error>;
    pub async fn simulate_with(&mut self, ...);
    pub fn status(&self) -> AtomicTransactionComposerStatus;
    pub fn clone_composer(&self) -> AtomicTransactionComposer;
}
```

Six runtime states (`Building → Built → Signed → Simulated → Submitted
→ Committed`), six mutating methods, and every method begins by
checking the status and returning `Err(Error::ComposerStatusInvalid)`
when the call doesn't make sense in the current state. The friction:

### 1. The valid call sequence is documented in comments, not types

The right way to use the composer is:

```rust
let mut atc = AtomicTransactionComposer::default();
atc.add_method_call(call_1)?;
atc.add_method_call(call_2)?;
atc.build_group()?;        // returns Vec, but most callers ignore it
atc.gather_signatures()?;  // returns Vec, most callers ignore it
let res = atc.execute(&algod).await?;
```

Nothing prevents:

- Calling `add_method_call` after `build_group` (returns
  `Err(ComposerStatusInvalid)`).
- Calling `submit` before `gather_signatures` (works, because
  `submit` internally calls `gather_signatures` — but this is hidden).
- Calling `gather_signatures` twice (the second is a no-op that
  returns the same cached signatures — but the API doesn't say so).

Every one of these mistakes is caught at runtime, by a `match
self.status` arm, returning `Err(ComposerStatusInvalid(String))` that
D8 had to introduce specifically to make the error matchable. The
compiler could enforce the same invariants for free if the states
were types.

### 2. `build_group` and `gather_signatures` return values most callers ignore

`build_group` mutates `self.txs` in place (assigns the group id) and
also returns `Vec<TransactionWithSigner>`. The vector is a clone of
the internal state; almost no caller uses it because the next step is
to call another method on `self`.

`gather_signatures` same shape: side-effect (populate
`self.signed_txs`) plus a returned `Vec<SignedTransaction>` that's
almost always discarded because the next call is `submit`/`execute`.

This is the standard tell of a stateful API pretending to be a
functional one. Either the function should be a true mutator returning
`Result<()>`, or the state should travel with the return.

### 3. `submit` vs `execute` vs `simulate` carry overlapping work

- `submit` sends the signed group to algod and returns `Vec<String>`
  (the tx-ids).
- `execute` calls `submit`, then waits for confirmation, then parses
  ABI return values out of the logs. Returns
  `ExecuteResult { confirmed_round, tx_ids, method_results }`.
- `simulate` calls algod's `/v2/transactions/simulate` and returns
  `AtcSimulateResult` — a parallel result type with the same shape
  plus a `simulate_response` field. The composer does *not* transition
  to `Submitted` on simulate; it goes to `Simulated` so a subsequent
  `execute` is still legal.

There is no path to "submit + confirm but skip the ABI parsing", or
"submit and don't wait", or "execute but without simulate's
non-destructive bias". The result-type and method-name combinations
that are actually meaningful are a strict subset of what the surface
exposes.

### 4. `clone_composer()` is a method because `Clone` isn't quite right

The composer holds `Vec<TransactionWithSigner>`, and
`TransactionWithSigner` holds `Option<Arc<dyn Signer>>` (post-D7). The
default `Clone` would shallow-clone the `Arc`s — which is correct, but
the API exposes `clone_composer()` separately so the intent
("snapshot for parallel construction") is named. The pattern says the
underlying type isn't quite Clone-shaped, which is a smell.

### 5. `AbiArgValue::AbiValue(...)` wraps every ABI arg

```rust
.args(vec![
    AbiArgValue::AbiValue(AbiValue::Int(BigUint::from(2u64))),
    AbiArgValue::AbiValue(AbiValue::Int(BigUint::from(3u64))),
])
```

`AbiArgValue` has two variants: `TxWithSigner(TransactionWithSigner)`
for transaction-typed ABI arguments, and `AbiValue(AbiValue)` for
everything else. The transaction-typed case is rare; the value case is
overwhelmingly common, so the inner-wrapper indirection is noise at
every call site.

### 6. `MethodCall.sender` is positional and structurally redundant

D6 made `MethodCall::new(app_id, method, sender, signer)` take four
positional args because the `Signer` trait can't expose a sender
(multisig + HSM cases). True, but in the *common* case the signer is
an `Account` whose `address()` IS the sender. The sender argument is
duplicated information the call site has to remember.

## Decision

Replace the single `AtomicTransactionComposer` + status enum with a
typestate chain. Each state is its own type; transitions consume the
previous state and produce the next one. The compiler enforces the
ordering.

### The shape

```rust
// Building state — accepts add_*; no submit/sign/simulate methods exist.
pub struct GroupBuilder { /* txs, method_map */ }

impl GroupBuilder {
    pub fn new() -> Self;
    pub fn add_transaction(self, t: TransactionWithSigner) -> Self;
    pub fn add_method_call(self, m: MethodCall)            -> Self;
    pub fn build(self) -> Result<UnsignedGroup, Error>;
}

// Built state — group IDs assigned, ready to sign or simulate.
pub struct UnsignedGroup { /* txs with group-ids stamped, method_map */ }

impl UnsignedGroup {
    pub fn transactions(&self) -> &[TransactionWithSigner];
    pub fn sign(self) -> Result<SignedGroup, Error>;
    pub async fn simulate(self, algod: &Algod) -> Result<SimulateOutcome, Error>;
    pub async fn simulate_with(self, algod: &Algod, opts: SimulateOptions) -> Result<SimulateOutcome, Error>;
}

// Signed state — ready to submit or execute.
pub struct SignedGroup { /* signed_txs, method_map */ }

impl SignedGroup {
    pub fn signed_transactions(&self) -> &[SignedTransaction];
    /// Submit and return a PendingSubmission (per D2). Caller decides
    /// whether to .confirm().await? or hold the handle.
    pub async fn submit(self, algod: &Algod) -> Result<PendingSubmission, Error>;
    /// Submit, confirm to finality, and parse ABI return values.
    pub async fn execute(self, algod: &Algod) -> Result<ExecuteOutcome, Error>;
}
```

The quickstart end-to-end:

```rust
let outcome = GroupBuilder::new()
    .add_method_call(call_1)
    .add_method_call(call_2)
    .build()?
    .sign()?
    .execute(&algod)
    .await?;
```

Each `.foo()` consumes `self` and returns the next state's type.
Calling `.submit()` on `GroupBuilder` doesn't compile. Calling
`.add_method_call()` on `SignedGroup` doesn't compile. The
`AtomicTransactionComposerStatus` enum, the `ComposerStatusInvalid`
error variant introduced in D8, and the runtime status checks all
retire together.

### What replaces `clone_composer`

`GroupBuilder` derives `Clone` (it's just owned state). The "snapshot
for parallel construction" use case becomes `let snapshot =
builder.clone();`. No special method needed. `UnsignedGroup` and
`SignedGroup` are also `Clone` since the signing-path types
(`Arc<dyn Signer>`, the typed transactions) are themselves cheap to
clone.

### What `submit`/`execute`/`simulate` look like after

The overlap collapses to one decision per state:

| Current API                                       | After                                                                  |
|---------------------------------------------------|------------------------------------------------------------------------|
| `composer.submit(&algod)` → `Vec<String>`         | `signed.submit(&algod)` → [`PendingSubmission`](pending-submission.md) |
| `composer.execute(&algod)` → `ExecuteResult`      | `signed.execute(&algod)` → `ExecuteOutcome`                            |
| `composer.simulate(&algod)` → `AtcSimulateResult` | `unsigned.simulate(&algod)` → `SimulateOutcome`                        |

`PendingSubmission::confirm()` plugs into the submit path (the D2
type already exists). `ExecuteOutcome` is the renamed `ExecuteResult`,
unchanged in shape. `SimulateOutcome` keeps `AtcSimulateResult`'s
shape but the "the composer can still be executed after a simulate"
property is automatic: simulate consumes `UnsignedGroup` by reference,
not by value — `pub async fn simulate(&self, ...)`. The
`UnsignedGroup` value survives, so `unsigned.simulate(&algod).await?`
followed by `unsigned.sign()?.execute(&algod).await?` is legal at the
type level.

### What stays the same

- `TransactionWithSigner`, `MethodCall`, `MethodCallBuilder`,
  `MethodCall::new(app_id, method, sender, signer)` — D6 already
  established these shapes; this ADR doesn't disturb them.
- `Signer` trait (D7) and `Option<Arc<dyn Signer>>` on
  `TransactionWithSigner` — unchanged.
- `PendingSubmission` (D2) becomes the submit path's return type.

### What this ADR explicitly does not propose

- **Flattening `AbiArgValue`** so the common `AbiValue(...)` case
  doesn't need the wrapper. That's a separate cut on the
  `MethodCallBuilder::args` setter (a `From<AbiValue> for AbiArgValue`
  impl plus an `impl Into<Vec<AbiArgValue>>` for `Vec<AbiValue>` would
  do it). Worth doing, but a follow-up.
- **Removing `MethodCall.sender`** in favour of derivation from the
  signer. D6's positional sender is right for the
  `Signer`-doesn't-expose-an-address general case; a convenience
  `MethodCall::with_account(app_id, method, &account)` constructor
  could remove the duplication for the common `Account`-signer case
  without changing the underlying shape. Follow-up.
- **Async signers.** `Signer::sign_transactions` is sync today; an
  async HSM signer needs the trait method to be async. That's a
  separate ADR — touching the typestate plumbing isn't a precondition.

## Consequences

- **Compile-error breaking change** for every caller of
  `AtomicTransactionComposer`. The composer is widely used in step-defs
  and at least one example; the migration is mechanical (`mut atc:
  AtomicTransactionComposer` → fluent chain) but visible.
- **`Error::ComposerStatusInvalid` retires.** Introduced in D8; gone
  here. So does `AtomicTransactionComposerStatus` and the
  `clone_composer()` method. The `status()` accessor goes — there is
  no status to query once the state IS the type.
- **The "simulate then execute" use case stays expressible**,
  through `simulate` taking `&self`. The composer's
  `Simulated`-between-`Signed`-and-`Submitted` enum gymnastics retire
  with the enum.
- **Result types collapse.** `ExecuteResult` and `AtcSimulateResult`
  become `ExecuteOutcome` / `SimulateOutcome` — same fields, the
  rename just stops them from looking like sibling types of a thing
  called "Composer". `Vec<String>` (today's `submit` return) is gone:
  callers get the typed `PendingSubmission` D2 added.
- **`AddMethodCallParams` was already gone (D6).** Nothing additional
  to migrate there.
- **Out of scope.** Async signers, `AbiArgValue` flattening,
  `MethodCall::with_account` — each is its own follow-up. The current
  ADR's value is replacing one runtime-checked state machine with a
  compile-time one; the small ergonomic wins are independent of that
  shape change.
