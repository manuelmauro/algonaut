---
id: identifier-newtypes-at-client-boundary
title: Identifier newtypes and domain types at the client boundary
abstract: Push the AppId / AssetId / Address / TxId newtypes through the algod / indexer / kmd public method signatures, turn the to_app_address free function into AppId::address, replace the TxGroup::assign_group_id slice-of-mut-refs API with a TxGroup::new constructor, add StateSchema::empty / new, and replace teal_compile's Option<bool> with a SourceMap two-variant enum. First sub-ADR addressing items D4 and D9 of the ideal-type-safe-ergonomic-api index.
status: proposed
date: 2026-05-20
deciders: []
tags: [api, ergonomics, type-safety]
---

# Identifier newtypes and domain types at the client boundary

## Status

Proposed. Implements decision items **D4** and **D9** of
[`ideal-type-safe-ergonomic-api`](ideal-type-safe-ergonomic-api.md).

## Context

`algonaut_core::{AppId, AssetId, TxId}` already exist as transparent
newtypes (PR #281). The transaction builders adopted them — `Pay`,
`CreateApplication::foreign_apps(Vec<AppId>)`, `Transaction::id` — but
the *client* methods that consume those same values still take bare
primitives:

```rust
algod.app(application_id: u64)         // accepts an AssetId silently
algod.asset(asset_id: u64)
algod.account(address: &str)           // takes a string, parses it back
algod.pending_txn(txid: &str)
algod.txn_proof(round: u64, txid: &str)
indexer.lookup_application_by_id(application_id: u64)
indexer.lookup_transaction(txid: &str)
kmd.delete_key(_, _, address: &str)
…
```

Twelve algod methods, sixteen indexer methods, three kmd methods. The
type system knows the value is an `Address` or an `AppId`; the API
throws that knowledge away at the call boundary and reconstructs it on
the other side. Real call sites compensate with `.to_string()` /
`.as_str()` stringification — `examples/rekey.rs` does it twice,
`tests/step_defs/integration/assets.rs` and `general.rs` do it for every
`account(...)` call.

The companion D9 cuts are the same problem in miniature:

- `to_app_address(app_id: u64)` is a free function precisely because
  `AppId` did not exist when it was written. Both current callers
  unwrap the newtype with `app_id.0`.
- `TxGroup::assign_group_id(&mut [&mut t1, &mut t2])` mutates through a
  slice of mutable references — the only API in the crate that does.
- `StateSchema { number_ints: 0, number_byteslices: 0 }` is the empty
  literal in `examples/app_create.rs` twice; no constructor exists.
- `algod.teal_compile(source, None)` is `Option<bool>` for source-map
  emission, paired with a separate `teal_compile_with_sourcemap` —
  `None` at the call site explains nothing.

## Decision

### Client signatures take the newtypes

For every public method on `Algod`, `Indexer`, and `Kmd` that today
accepts an identifier as `&str` / `u64`:

| Today                                              | After                                              |
|----------------------------------------------------|----------------------------------------------------|
| `address: &str`, `account_id: &str`                | `address: &Address`                                |
| `application_id: u64` (and `Option<u64>`)          | `app_id: AppId` / `Option<AppId>`                  |
| `asset_id: u64` (and `Option<u64>`)                | `asset_id: AssetId` / `Option<AssetId>`            |
| `txid: &str` (and `Option<&str>`)                  | `txid: &TxId` / `Option<&TxId>`                    |

Stringification stays at the HTTP boundary inside the client: the
generated `algonaut_algod` / `algonaut_indexer` / `algonaut_kmd`
operations still consume `&str` / `i32` / `u64`, and the hand-written
wrappers convert at the edge. The public surface stops demanding the
conversion from callers.

### `to_app_address` becomes a method

`algonaut_core::to_app_address(u64)` deletes; `AppId::address(self) ->
Address` replaces it. Both existing callers (`tests/step_defs/integration/abi.rs`,
`src/util/dryrun_printer.rs`) drop the `.0` unwrap.

### `TxGroup::assign` returns owned transactions

`TxGroup::assign_group_id(&mut [&mut t1, &mut t2])` is replaced by

```rust
TxGroup::assign(vec![t1, t2]) -> Result<Vec<Transaction>, TransactionError>
```

which consumes its inputs and returns the grouped copies. The verb-y
name matters: the ADR index originally proposed `TxGroup::new`, but a
`new` that returns `Vec<Transaction>` rather than a `TxGroup` is exactly
what `clippy::new_ret_no_self` is built to catch — the caller writes
`TxGroup::new(...)` expecting a `TxGroup` back and gets something else.
`assign` describes the action and lets the lint stay on.

The composer-internal in-place form survives as
`TxGroup::assign_in_place(&mut [&mut Transaction])`, `#[doc(hidden)]
pub` so the workspace can reach it without leaking it into the public
surface. The `assign_group_id` name is also retired to avoid the clash
with the same-named method on `Transaction` itself.

### `StateSchema` gains constructors

```rust
StateSchema::empty()           // both zero
StateSchema::new(ints, slices) // explicit counts
```

The struct keeps `pub` fields (it is a plain `(u64, u64)`) but the four
"empty literal" sites and the two "with values" sites stop reaching for
the field names.

### `teal_compile` takes `SourceMap::{Emit, Skip}`

```rust
pub enum SourceMap {
    Emit,
    Skip,
}

algod.teal_compile(&source, SourceMap::Skip).await?;
```

The `algod.teal_compile_with_sourcemap` companion stays — it returns a
parsed `SourceMap` value, a strictly different return shape — but the
boolean `Option` on the plain `teal_compile` is gone, so the eight call
sites that pass `None` get to say what they mean.

## Consequences

- **Compile-error breaking change** for every external caller of the
  thirty-one renamed methods. Pre-1.0; the crate's `README` calls out
  that the API still moves. Migration is mechanical: wrap a literal
  with `AppId(..)` / `AssetId(..)`, replace `&account.address().to_string()`
  with `&account.address()`, replace `tx_id.as_str()` with `&tx_id`.
- **Mixed-ID mistakes become compile errors.** `algod.app(asset.id())`
  no longer compiles; the newtypes share no `From` between each other.
- **No serialization change.** `AppId` / `AssetId` are
  `#[serde(transparent)]` over `u64`; `TxId` over `String`; `Address`
  already round-trips correctly per
  [`domain-types-serialize-for-both-json-and-msgpack`](domain-types-serialize-for-both-json-and-msgpack.md).
  Wire bytes are identical before and after.
- **The conversion layer is one line per method.** Each wrapper gains
  `&address.to_string()` / `app_id.0` at the call into the generated
  client. The cost is paid once per endpoint; user code stops paying it
  at every call site.
- **The examples shrink.** `examples/atomic_swap.rs` drops the
  `&mut [&mut t1, &mut t2]` shape. `examples/app_create.rs` drops two
  `StateSchema { number_ints: 0, number_byteslices: 0 }` literals and
  two `teal_compile(.., None)`s. `examples/rekey.rs` drops two
  `.to_string()` calls.
- **Out of scope.** The remaining client-edge surface (response types —
  D3 in the index ADR) is unchanged here. Methods continue to return
  `*200Response` types and `pub use algonaut_algod as openapi_algod`
  stays. D2 (finality on the client) and D3 (hide generated types) are
  separate sub-ADRs that build on this one.
