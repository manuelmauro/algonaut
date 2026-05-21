---
id: atomic-module-layout
title: 'Atomic module: rename and submodule layout'
abstract: 'Rename the src/atomic_transaction_composer/ module to atomic (algonaut::atomic) — the directory named a type (AtomicTransactionComposer) deleted by the typestate refactor — and split its 1115-line mod.rs into focused submodules: group, method_call, encode, outcome, signing.'
status: accepted
date: 2026-05-21
deciders: []
tags: [api, module-layout, atomic-transaction-composer]
---

# Atomic module: rename and submodule layout

## Status

Accepted. Follows the now-landed
[`atomic-transaction-composer-typestate`](atomic-transaction-composer-typestate.md),
which deleted the `AtomicTransactionComposer` type the directory was named
after.

## Context

The module lives at `src/atomic_transaction_composer/` and is exported as
`algonaut::atomic_transaction_composer`. The name dates from when a single
`AtomicTransactionComposer` struct (plus an `AtomicTransactionComposerStatus`
enum) was the module's centrepiece.

[`atomic-transaction-composer-typestate`](atomic-transaction-composer-typestate.md)
deleted that struct, splitting it into a typestate chain —
`AtomicGroupBuilder` → `UnsignedAtomicGroup` → `SignedAtomicGroup`. The
directory name now points at a type that no longer exists, and it stutters:
`algonaut::atomic_transaction_composer::AtomicGroupBuilder`. Doc comments in
`method_call.rs` still reference the deleted `AtomicTransactionComposer`.

Two problems, then:

### 1. The module name is stale

No public type carries "composer" anymore. The domain noun the surviving
types share is the **atomic (transaction) group**: `AtomicGroupBuilder`,
`UnsignedAtomicGroup`, `SignedAtomicGroup`.

### 2. `mod.rs` is a 1115-line grab-bag

A single file holds five distinct concerns:

- the typestate chain and its transition methods (`sign`/`simulate`/`submit`/`execute`);
- ABI method-call → transaction encoding (`process_method_call`, the
  foreign-array packing, `validate_tx`);
- result/outcome types and ABI return-value decoding (`ExecuteOutcome`,
  `SimulateOutcome`, `AbiMethodResult`, `get_return_value_*`);
- signer orchestration (`sign_group`, `placeholder_group`);
- finality polling (`poll_until_confirmed`).

The encoding and decoding helpers — the bulk of the line count — are
internal plumbing that obscures the small public typestate surface a reader
comes here to find.

## Decision

### Rename the module to `atomic`

`src/atomic_transaction_composer/` → `src/atomic/`, exported as
`algonaut::atomic`. Call sites read `algonaut::atomic::AtomicGroupBuilder`.

`atomic` over the alternatives:

- **`atomic` (chosen)** — short; within an Algorand SDK "atomic" already
  connotes atomic transaction groups. Mirrors the standard library's
  `std::sync::atomic` precedent: a module named `atomic` whose types are
  `Atomic*`.
- **`atomic_group`** — the most literal match to the type stem, but stutters
  against the type names (`atomic_group::AtomicGroupBuilder`).
- **`composer`** — keeps the ecosystem-familiar "AtomicTransactionComposer"
  heritage, but the typestate ADR deliberately moved *away* from "Composer"
  naming, so reintroducing it at the module level works against that.

### Split into submodules

`mod.rs` becomes a thin re-export + shared constants file. The concerns move
into siblings:

| File             | Holds                                                                                          |
|------------------|------------------------------------------------------------------------------------------------|
| `mod.rs`         | module docs, `pub use` re-exports, shared constants                                            |
| `group.rs`       | the public typestate chain: `TransactionWithSigner`, `AtomicGroupBuilder`, `UnsignedAtomicGroup`, `SignedAtomicGroup` |
| `method_call.rs` | `MethodCall`, `MethodCallBuilder`, `AbiArgValue`                                                |
| `encode.rs`      | ABI method-call → transaction encoding, foreign-array packing, `validate_tx`                   |
| `outcome.rs`     | `ExecuteOutcome`, `SimulateOutcome`, `AbiMethodResult`, `AbiMethodReturnValue`, `AbiReturnDecodeError`, return-value decoding |
| `signing.rs`     | `sign_group`, `placeholder_group`, `tx_ids`, `poll_until_confirmed`                            |

The public API is unchanged except for the module path: the same types are
re-exported from `atomic`, so `algonaut::atomic::AtomicGroupBuilder` and the
rest resolve exactly as before.

## Consequences

- **Breaking change** for the import path only: `algonaut::atomic_transaction_composer::*`
  → `algonaut::atomic::*`. No type names, signatures, or behaviour change.
  Consistent with the in-flight D-series breaking changes; the crate is
  pre-1.0.
- **The public surface is legible.** A reader opening `atomic/` sees the
  typestate chain in `group.rs` without wading through ~600 lines of ABI
  encode/decode plumbing; the internals sit in named siblings.
- **Stale references retire.** The `AtomicTransactionComposer` mentions in
  `method_call.rs` doc comments are corrected to the typestate types as part
  of the move.
- **Call sites to update:** `lib.rs`, `error.rs`, `simulate.rs`, the
  `atomic_transaction_composer` example, and the cucumber step-defs. The
  example file is renamed to match.
- **No new public types**, so nothing downstream depends on the internal file
  boundaries; a later reshuffle of the private submodules needs no further ADR.
