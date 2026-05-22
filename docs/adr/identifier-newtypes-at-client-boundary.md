---
id: identifier-newtypes-at-client-boundary
title: Identifier newtypes and domain types at the client boundary
abstract: Push the AppId / AssetId / Address / TransactionId newtypes through the algod / indexer / kmd public method signatures, turn the to_app_address free function into AppId::address, replace the TransactionGroup::assign_group_id slice-of-mut-refs API with a TransactionGroup::new constructor, add StateSchema::empty / new, and replace teal_compile's Option<bool> with a SourceMap two-variant enum. First sub-ADR addressing items D4 and D9 of the ideal-type-safe-ergonomic-api index.
status: accepted
date: 2026-05-20
deciders: []
tags: [api, ergonomics, type-safety]
---

# Identifier newtypes and domain types at the client boundary

## Status

Accepted. Implements decision items **D4** and **D9** of
[`ideal-type-safe-ergonomic-api`](ideal-type-safe-ergonomic-api.md).

## Context

`algonaut_core::{AppId, AssetId, TransactionId}` already exist as transparent
newtypes (PR #281). The transaction builders adopted them — `Pay`,
`CreateApplication::foreign_apps(Vec<AppId>)`, `Transaction::id` — but
the *client* methods that consume those same values still take bare
primitives:

```rust
algod.app(application_id: u64)         // accepts an AssetId silently
algod.asset(asset_id: u64)
algod.account(address: &str)           // takes a string, parses it back
algod.pending_transaction(transaction_id: &str)
algod.transaction_proof(round: u64, transaction_id: &str)
indexer.lookup_application_by_id(application_id: u64)
indexer.lookup_transaction(transaction_id: &str)
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
- `TransactionGroup::assign_group_id(&mut [&mut t1, &mut t2])` mutates through a
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
| `txid: &str` (and `Option<&str>`)                  | `transaction_id: &TransactionId` / `Option<&TransactionId>`         |

Stringification stays at the HTTP boundary inside the client: the
generated `algonaut_algod` / `algonaut_indexer` / `algonaut_kmd`
operations still consume `&str` / `i32` / `u64`, and the hand-written
wrappers convert at the edge. The public surface stops demanding the
conversion from callers.

### `to_app_address` becomes a method

`algonaut_core::to_app_address(u64)` deletes; `AppId::address(self) ->
Address` replaces it. Both existing callers (`tests/step_defs/integration/abi.rs`,
`src/util/dryrun_printer.rs`) drop the `.0` unwrap.

### `TransactionGroup` *is* the grouped batch

The old `TransactionGroup` was a passive msgpack-hashing helper holding
`Vec<HashDigest>` — exposed publicly only so the in-place
`assign_group_id(&mut [&mut Transaction])` API had somewhere to live as
a static method. Two iterations during this ADR — first a public
`TransactionGroup::new(Vec<Transaction>) -> Result<Vec<Transaction>>`, then a
rename to `TransactionGroup::assign` — both kept that shape and both were
unsatisfying: the type had no domain meaning, and `new` returning
something other than `Self` is precisely the pattern
`clippy::new_ret_no_self` exists to flag.

The end state is `TransactionGroup` representing what callers think it does — a
batch of transactions sharing a group ID — and a `TryFrom<Vec<Transaction>>`
impl as the single construction path:

```rust
let group: TransactionGroup = vec![t1, t2].try_into()?;
// or
let group = TransactionGroup::try_from(vec![t1, t2])?;

for tx in group {
    // IntoIterator over the grouped transactions
}
```

`TransactionGroup::transactions(&self) -> &[Transaction]` borrows, and
`TransactionGroup::into_transactions(self) -> Vec<Transaction>` consumes.

The msgpack hashing form (the previous public struct) moves to a
private `TransactionGroupDigests` inside `transaction_group.rs`; the in-place mutating
form survives as `transaction_group::assign_in_place(&mut [&mut Transaction])` —
a free function in the module, `#[doc(hidden)] pub` so the atomic
transaction composer can reach it across the workspace boundary without
leaking it into the public surface.

The same-named `assign_group_id` method on `Transaction` (the
per-transaction setter) is unaffected and keeps its name; the clash
that the rename was working around no longer exists once the batch
operation is a `TryFrom` impl on `TransactionGroup`.

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
  with `&account.address()`, replace `transaction_id.as_str()` with `&transaction_id`.
- **Mixed-ID mistakes become compile errors.** `algod.app(asset.id())`
  no longer compiles; the newtypes share no `From` between each other.
- **No serialization change.** `AppId` / `AssetId` are
  `#[serde(transparent)]` over `u64`; `TransactionId` over `String`; `Address`
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
