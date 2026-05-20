---
id: ideal-type-safe-ergonomic-api
title: An ideal type-safe, ergonomic API for algonaut
abstract: Set a north-star direction for the algonaut public API — newtype identifiers, single-stage transaction builders, client-bound submit/confirm helpers, a Signer trait, a method-call builder, domain types at the network boundary, and structured errors — so the common flows in examples/ and tests/step_defs stop repeating boilerplate and stop relying on stringly-typed escape hatches.
status: proposed
date: 2026-05-19
deciders: []
tags: [api, ergonomics, type-safety]
---

# An ideal type-safe, ergonomic API for algonaut

## Status

Proposed

## Context

The `examples/` directory and the cucumber `tests/step_defs/` are the two
most honest descriptions of what using algonaut actually feels like:
`examples/` is what we tell new users to copy, and the step-defs are the
code we wrote ourselves to drive every feature in the
`algorand-sdk-testing` contract. Reading them side by side surfaces a
consistent set of friction points. None is a bug; together they are a
direction problem. This ADR records the target shape so future
changes — most of them breaking, hence pre-1.0 — can be sequenced
against a single agreed picture instead of being argued case by case.

### 1. Every transaction is built twice

The canonical line, repeated in `payment.rs`, `atomic_swap.rs`,
`asset_create.rs`, `app_create.rs`, `key_reg.rs`, and the step-defs:

```rust
let t = TxnBuilder::with(
    &params,
    Pay::new(alice.address(), bob.address(), MicroAlgos(123_456)).build(),
)
.build()?;
```

There are two `build()` calls. The inner one turns a `Pay` into a
`TransactionType` enum; the outer one turns header fields plus that
enum into a `Transaction`. The inner builder is infallible, the outer
one returns `Result` — so the same word means two different things one
nesting level apart. A reader cannot tell from the call site why one
`build` can fail and the other cannot.

### 2. The confirm loop is copy-pasted, not imported

`src/util/wait_for_pending_tx.rs` exists. Yet `examples/app_create.rs`
and `examples/asset_create.rs` each carry their own verbatim copy of a
`wait_for_pending_transaction` helper, and `tests/step_defs/util.rs`
carries a third. A utility that ships with the crate but is re-pasted by
every caller that needs it is a utility with the wrong shape: finality
polling belongs on the client, not in user code.

### 3. Generated OpenAPI names leak into the public surface

`algod.send_txn()` returns `RawTransaction200Response`;
`algod.txn_params()` returns `TransactionParams200Response`;
`algod.status()` returns `GetStatus200Response`. These are
machine-generated names from `algonaut_algod`. They appear in user code,
in `rustdoc`, and in error messages. Worse, the same type is reachable
by two different paths — `examples/app_create.rs` imports
`algonaut_algod::models::PendingTransactionResponse` while
`examples/asset_create.rs` imports
`algonaut::openapi_algod::models::PendingTransactionResponse` for the
identical type. There is no single blessed name.

### 4. Identifiers are bare `u64`

Application IDs, asset IDs, and foreign-app/asset references are all
`u64`. `CreateApplication::foreign_assets(Vec<u64>)` and
`CallApplication::new(sender, app_id: u64)` accept the same primitive,
so passing an asset ID where an app ID is expected compiles cleanly.
The free function `to_app_address(app_id: u64)` in `algonaut_core`
exists precisely because `u64` carries no identity — a method on a real
type would not need to be a free function. `Round` is already a
newtype; the rest of the identifier family is not.

### 5. `SignedTransaction` is hand-constructible with a fake ID

`examples/multi_sig.rs`, `examples/logic_sig_delegated.rs`, and the
`simulate.rs` step-def all build a `SignedTransaction` literal:

```rust
SignedTransaction {
    transaction: t,
    transaction_id: "".to_owned(),   // placeholder
    sig,
    auth_address: None,
}
```

`transaction_id` is a derived value — it is a hash of the
transaction — yet the type lets callers store an empty string there. A
struct whose invariants are not enforced by its constructor will, given
enough call sites, be constructed wrong.

### 6. The method-call path is a wall of `None`

`AtomicTransactionComposer::add_method_call` takes
`&mut AddMethodCallParams`, an 18-field struct. The `abi.rs` step-def
copes by funnelling everything through one private helper with **twelve
positional arguments**, most of them `Option`:

```rust
add_method_call(w, account, oc, None, None, None, None, None, None, None, false, None).await;
```

Six distinct call sites pass six different `None`-patterns into that
helper. This is the single least ergonomic corner of the SDK, and it is
the corner that ARC-4 application development depends on most.

### 7. The network boundary speaks `&str`, not domain types

`algod.account(address: &str)`, `algod.pending_txn(txid: &str)`. Callers
holding a typed `Address` must stringify it —
`algod.account(&transient_account.address().to_string())` in
`applications.rs` and `assets.rs` — so the client can parse it back.
The type system knows the value is an address; the API throws that
knowledge away at the call and reconstructs it on the other side.

### 8. Domain errors are stringly typed

`build_group()` reports an empty group as `Error::Msg("attempting to
build group with zero transactions")`. The `abi.rs` step-def asserts on
it by **substring match**. `Error` has `Msg(String)` and
`Internal(String)` catch-alls; any logic that wants to branch on a
failure mode has to pattern-match on English prose. A message reword
silently breaks callers.

### 9. Smaller cuts

- `TxGroup::assign_group_id(&mut [&mut t1, &mut t2])` mutates through a
  slice of mutable references — see `atomic_swap.rs`.
- `algod.teal_compile(source, None)` takes `Option<bool>` for
  source-map emission, while a separate `teal_compile_with_sourcemap`
  also exists; the `None` at the call site explains nothing.
- `TransactionSigner` is a closed enum with an `Empty` variant — a
  "signer" that does not sign — so a hardware wallet or remote KMS
  cannot be plugged in, and `Empty` is a footgun in every `match`.
- `StateSchema { number_ints: 0, number_byteslices: 0 }` is written as
  a bare literal twice in `app_create.rs`.

## Decision

Adopt the following as the target public API. It is the reference shape
for all pre-1.0 breaking changes; individual pieces ship as their own
PRs and ADRs, but each must move toward — never away from — this
picture.

### Target: the quickstart end to end

```rust
let algod = Algod::new(url, token)?;
let alice = Account::from_mnemonic(MNEMONIC)?;
let bob: Address = "2FML...".parse()?;

let params = algod.suggested_params().await?;

let txn = Pay::new(alice.address(), bob, MicroAlgos(123_456))
    .note(b"hello")
    .build(&params)?;                       // ONE build

let confirmed = algod
    .submit(&alice.sign(txn)?)              // -> PendingSubmission
    .confirm()                              // polls to finality
    .await?;

println!("confirmed in round {}", confirmed.round);
```

Every numbered item below is one step from today's code to that block.

### D1 — One build per transaction

Fold `TxnBuilder` into the transaction-type builders. `Pay`,
`CreateAsset`, `CallApplication`, … keep their fluent setters and gain
the header setters (`note`, `lease`, `rekey_to`, `group`) plus a single
terminal `build(&params) -> Result<Transaction, BuildError>`. The
intermediate `TransactionType`-returning `build()` and the separate
`TxnBuilder` type are removed. `with_fee` becomes a `fee(MicroAlgos)`
setter. Validation that can fail stays in the one remaining `build`;
everything else is infallible chaining.

### D2 — Finality is a client capability

`submit` returns a `PendingSubmission` handle, not a raw response.
`PendingSubmission::confirm()` polls to finality with a sane default
timeout; `confirm_with(Duration)` overrides it; `tx_id()` is available
without awaiting. Delete the per-example `wait_for_pending_transaction`
copies and the unused `src/util/wait_for_pending_tx.rs`; the one
implementation lives behind `submit`.

### D3 — Domain types at the boundary, generated types hidden

`algonaut_algod`, `algonaut_indexer`, `algonaut_kmd` remain
generated and remain a dependency, but they stop being part of the
public surface. Each client method returns a hand-named type from
`algonaut_model` (e.g. `SuggestedParams`, `SubmitResult`,
`PendingTransaction`, `NodeStatus`), constructed by converting the
generated response at the client edge. The `pub use algonaut_algod as
openapi_algod` re-exports in `lib.rs` are removed. No `*200Response`
name and no `openapi_*` path appears in user code or `rustdoc`.

### D4 — Newtype the identifier family

Introduce `AppId(u64)` and `AssetId(u64)` in `algonaut_core`, alongside
the existing `Round`. They are `Copy`, parse/display, and serialize
transparently. `to_app_address` becomes `AppId::address(self)`. Builder
and client signatures take the newtypes: `CallApplication::new(sender,
AppId)`, `foreign_assets(Vec<AssetId>)`, `algod.app(AppId)`,
`algod.asset(AssetId)`. Mixing an asset ID and an app ID becomes a
compile error.

### D5 — `SignedTransaction` cannot be built wrong

`SignedTransaction` becomes constructible only through signing paths;
its fields stop being `pub`. `transaction_id` is computed from the
transaction, never passed in. Multisig and logic-sig signing get
first-class entry points so no example needs a struct literal:

```rust
let signed = Multisig::new(1, 2, [alice.addr(), bob.addr()])?
    .sign(txn, &alice)?
    .sign_more(&bob)?
    .finish()?;
```

### D6 — A builder for method calls

Replace the 18-field `AddMethodCallParams` and
`add_method_call(&mut params)` with a fluent `MethodCall` builder. Only
the four genuinely required inputs are positional; everything else is an
optional setter, so the common call carries no `None`:

```rust
let call = MethodCall::new(AppId(id), method, &alice)
    .args(args)
    .on_complete(OnComplete::NoOp)
    .boxes(boxes)            // optional, omitted when unused
    .build(&params)?;
composer.add_method_call(call)?;
```

This is the highest-leverage single change for ARC-4 users and it
deletes the twelve-argument helper in `abi.rs` outright.

### D7 — `Signer` is a trait

Replace the `TransactionSigner` enum with a `Signer` trait
(`fn sign_transactions(&self, &[Transaction]) -> Result<Vec<SignedTransaction>, ...>`).
`Account`, `Multisig`, and `LogicSig` implement it; third parties can
implement it for an HSM or remote KMS. The `Empty` variant is gone —
"a transaction with no signer yet" is modelled as `Option<&dyn Signer>`
at the one place (composer group assembly) that needs it, instead of an
inhabitable do-nothing signer.

### D8 — Structured errors

Give `Error` real variants for the failure modes callers branch on —
`EmptyTransactionGroup`, `OverspendFee`, `InvalidProgram`, an
`Http { status, .. }` already exists — and reserve `Msg`/`Internal` for
genuinely unstructured cases. Tests and downstream code match variants;
no test asserts on a substring of an error message.

### D9 — Accept what you mean

Client methods take `&Address`, `&TxId`, `AppId`, `AssetId` — not
`&str`/`u64`. `teal_compile` either drops the `Option<bool>` and always
returns the source map slot (`CompiledTeal` with an optional map), or
the boolean is replaced by a `SourceMap::Emit` / `SourceMap::Skip`
two-variant enum so the call site reads as English. `TxGroup` exposes
`TxGroup::new([t1, t2]) -> Result<Vec<Transaction>>` returning grouped
copies rather than mutating through `&mut [&mut T]`. `StateSchema` gains
`StateSchema::empty()` and `StateSchema::new(ints, slices)`.

## Consequences

- **This is a breaking-change roadmap.** It is viable only because the
  crate is pre-1.0 (`README`: "the API is stabilising but still
  moves"). Each item ships as its own PR with its own ADR and
  `CHANGELOG` entry; this ADR is the index they all reference.
- **The examples shrink.** `payment.rs` loses its double `build`;
  `app_create.rs` and `asset_create.rs` lose their pasted confirm
  loops; `multi_sig.rs` and `logic_sig_delegated.rs` lose their
  `SignedTransaction` literals. The examples become the regression test
  for ergonomics — if an example regrows boilerplate, the API regressed.
- **The step-defs get simpler and stricter.** The twelve-argument
  `add_method_call` helper collapses into `MethodCall` builder calls;
  the `build_group` error assertion matches `Error::EmptyTransactionGroup`
  instead of an English substring.
- **The generated crates stay, but become an implementation detail.**
  Regenerating `algonaut_algod` from an updated OpenAPI spec no longer
  ripples generated type names into user code — the conversion layer at
  the client edge absorbs it. This complements, and is bounded by, the
  existing [domain-types-serialize-for-both-json-and-msgpack](domain-types-serialize-for-both-json-and-msgpack.md)
  decision.
- **Cost.** A hand-named model layer and an identifier-newtype family
  are real surface to write and keep in sync with the spec. The
  conversion layer is the main ongoing tax; it is paid once per
  endpoint and is mechanical.
- **Non-goals.** This ADR does not propose typestate-encoded builders
  (a `Builder<Unsigned>` → `Builder<Signed>` chain), a blocking/sync
  client facade, or removing the `algonaut_*` workspace split. Those
  are larger bets and, if pursued, get their own ADRs. The line drawn
  here is: fix the shapes that the examples and step-defs already prove
  are wrong, and stop there.
</content>
</invoke>
