---
id: one-build-per-transaction
title: One build per transaction
abstract: Fold the outer `TxnBuilder` into each per-type builder (`Pay`, `CreateAsset`, `CallApplication`, …). Each builder grows the six header setters (`fee`, `note`, `lease`, `rekey_to`, `group`, `genesis_id`) and a single terminal `build(&params) -> Result<Transaction, TransactionError>` — replacing today's two-stage `TxnBuilder::with(&params, Inner::new(...).build()).build()?` pattern. Second sub-ADR addressing decision item D1 of the ideal-type-safe-ergonomic-api index.
status: proposed
date: 2026-05-20
deciders: []
tags: [api, ergonomics, type-safety]
---

# One build per transaction

## Status

Proposed. Implements decision item **D1** of
[`ideal-type-safe-ergonomic-api`](ideal-type-safe-ergonomic-api.md).

## Context

The canonical transaction-construction line, repeated in every example
and most step-defs:

```rust
let t = TxnBuilder::with(
    &params,
    Pay::new(alice.address(), bob.address(), MicroAlgos(123_456)).build(),
)
.build()?;
```

Two `build()` calls. The inner one (`Pay::build`) turns a `Pay` into a
`TransactionType` enum; the outer one (`TxnBuilder::build`) turns header
fields plus that enum into a `Transaction`. The inner builder is
infallible, the outer one returns `Result` — so the same word means two
different things, one nesting level apart. A reader cannot tell from the
call site why one `build` can fail and the other cannot.

The outer header setters (`note`, `lease`, `rekey_to`, …) live on
`TxnBuilder`, separated from the type-specific setters on `Pay` /
`CreateAsset` / … . Setting a note and a close-remainder requires
threading both through the two-stage call chain.

## Decision

Every per-type builder (`Pay`, `RegisterKey`, `CreateAsset`,
`UpdateAsset`, `DestroyAsset`, `TransferAsset`, `AcceptAsset`,
`ClawbackAsset`, `FreezeAsset`, `CreateApplication`,
`UpdateApplication`, `CallApplication`, `ClearApplication`,
`CloseApplication`, `DeleteApplication`, `OptInApplication`) embeds a
shared `TxnHeader` and exposes the six header setters directly. The
terminal `build(self) -> TransactionType` is replaced by
`build(self, params: &impl TransactionParams) -> Result<Transaction,
TransactionError>`, which finalises both the header and the
type-specific fields in one shot.

```rust
let t = Pay::new(alice.address(), bob.address(), MicroAlgos(123_456))
    .note(b"hello".to_vec())
    .build(&params)?;
```

The outer `TxnBuilder` retires.

### `TxnHeader` and `impl_txn_header_setters!`

A new `TxnHeader { fee, note, lease, rekey_to, group, genesis_id }`
struct lives in `algonaut_transaction::builder` and stores the six
optional header fields. Every per-type builder gains a
`header: TxnHeader` field, defaulted in its constructor(s).

A `macro_rules! impl_txn_header_setters!` at the top of the same module
mints the six fluent setters (`fee`, `note`, `lease`, `rekey_to`,
`group`, `genesis_id`) on a target builder. The macro is applied once
per builder — at the bottom of its `impl` block — so the setters are
inherent methods, callable without a trait import:

```rust
impl_txn_header_setters!(Pay);
impl_txn_header_setters!(CreateAsset);
// ...
```

A trait + blanket-impl alternative was considered (`HasTxnHeader` plus
`TxnHeaderSetters` default methods). The macro wins on grep-ability: a
reader looking for `Pay::note` finds it via `rustdoc`, IDE
auto-complete, and `rg '\.note\('` without needing to know the trait
exists.

### `TxnHeader::into_transaction`

Each builder's new `build(&params)` finishes by calling
`self.header.into_transaction(params, txn_type)`, the helper that
formerly lived inside `TxnBuilder::build_tx`. It applies the suggested
params (`first_valid` = `last_round`, `last_valid` = `last_round +
1000`, `fee` defaults to `min_fee`, `genesis_id` falls back to params)
and returns the `Transaction`. `pub(crate)` — only the per-type
builders and the atomic-transaction-composer need it.

### The composer

`AtomicTransactionComposer::add_method_call` used to build its
`ApplicationCallTransaction` enum variant by hand and wrap it in
`TxnBuilder::with_fee(&params, fee, app_call).rekey_to(...)...build()?`.
After this change, the same construction happens inline (the
`Transaction` struct's fields are `pub`), using the same `last_round +
1000` shape `TxnHeader::into_transaction` uses. The composer does not
go through any per-type builder because its `AddMethodCallParams` is
the union of every application-call variant — the `MethodCall`
fluent builder (D6) is the eventual replacement for that path.

## Consequences

- **Compile-error breaking change** for every caller writing
  `TxnBuilder::with(...)` or `TxnBuilder::with_fee(...)`. Pre-1.0;
  mechanical migration: drop `TxnBuilder`, append `.build(&params)?` to
  the inner builder, move any header setters onto the inner builder.
  ~40 call sites across `examples/` and `tests/step_defs/` migrated in
  this PR.
- **The same word stops meaning two things.** There is one `build` per
  transaction now, and it is the one that can fail. Type-specific
  setters and header setters share a namespace; the call site reads in
  natural order.
- **Out of scope.** D6 — the `MethodCall` fluent builder replacing
  `AddMethodCallParams` / `add_method_call(&mut params)` — is a
  separate sub-ADR. The composer's current internal construction path
  is preserved as-is, just inlined now that `TxnBuilder` is gone.
- **No serialization or wire-format change.** `Transaction` is byte-
  identical before and after; only the path to constructing it
  changes.
