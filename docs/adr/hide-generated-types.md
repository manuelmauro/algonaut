---
id: hide-generated-types
title: Hand-named response types at the client edge (first cut)
abstract: New `algonaut_model::client_types` module with hand-named wrappers — `SuggestedParams`, `NodeStatus`, `Supply` — replacing the OpenAPI-generator names (`TransactionParams200Response`, `GetStatus200Response`, `GetSupply200Response`) on the most-touched algod methods. `TransactionParams` trait moves from `algonaut_transaction::builder` to `algonaut_model::client_types` so `SuggestedParams` plugs into every `Builder::build(&params)` site. First slice of decision item D3 — the full coverage is staged across follow-up PRs.
status: superseded
superseded_by: relocate-generated-models
date: 2026-05-20
deciders: []
tags: [api, ergonomics, type-safety]
---

# Hand-named response types at the client edge (first cut)

## Status

Superseded by [`relocate-generated-models`](relocate-generated-models.md).
This was the first slice of decision item D3 from
[`ideal-type-safe-ergonomic-api`](ideal-type-safe-ergonomic-api.md) — three
hand-named wrappers (`SuggestedParams`, `NodeStatus`, `Supply`). Carrying the
wrap-every-response strategy to completion did not scale (it would require
hand-re-typing the ~142-type generated graph), so it is replaced by relocating
the generated models into `algonaut_model` and renaming the response envelopes
at generation. The three wrappers fold back into generation (their domain types
move to `type-overrides.json`); the merged first cut stays in place until that
work lands.

## Context

`algod.send_txn()` returns `RawTransaction200Response`;
`algod.txn_params()` returns `TransactionParams200Response`;
`algod.status()` returns `GetStatus200Response`. These are
machine-generated names from `algonaut_algod`. They appear in user
code, in `rustdoc`, and in error messages. Worse, the same type is
reachable by two different paths — `examples/app_create.rs` imports
`algonaut_algod::models::PendingTransactionResponse` while
`examples/asset_create.rs` imports
`algonaut::openapi_algod::models::PendingTransactionResponse` for the
identical type. There is no single blessed name.

## Decision

### A new home: `algonaut_model::client_types`

Hand-named domain types live in `algonaut_model::client_types`,
constructed by converting the generated response at the client edge
inside the umbrella crate. The first cut wraps three high-traffic
types:

| Generated name                       | Hand-named domain type | Methods migrated                                                |
|--------------------------------------|------------------------|-----------------------------------------------------------------|
| `TransactionParams200Response`       | `SuggestedParams`      | `Algod::txn_params`                                             |
| `GetStatus200Response`               | `NodeStatus`           | `Algod::status`, `Algod::status_after_block`                    |
| `GetSupply200Response`               | `Supply`               | `Algod::supply`                                                 |

The remaining algod/indexer/kmd responses keep their generated names
for now — each migrates as its own follow-up PR. The shape this cut
establishes is the template: a new struct in
`algonaut_model::client_types`, a `From`-shaped conversion at the
client edge, the wrapper method's return type changes from the
generated name to the hand-named one.

### `TransactionParams` trait moves to `algonaut_model::client_types`

Today the trait lives in `algonaut_transaction::builder` and the
generated `TransactionParams200Response` implements it. To let
`SuggestedParams` (now in `algonaut_model`) implement the same trait —
without inducing a workspace dependency cycle
(`algonaut_transaction → algonaut_model` already exists) — the trait
moves to `algonaut_model::client_types`.
`algonaut_transaction::builder` re-exports it via `pub use`, so every
existing `use algonaut_transaction::builder::TransactionParams` import
keeps working without change.

### Why the typed `Round` / `MicroAlgos` in the hand-named types

`SuggestedParams.last_round` is `Round` (not `u64`);
`Supply.total_money` is `MicroAlgos` (not `u64`). The hand-named layer
is the right place to typify these values — the generated layer can
only return what the OpenAPI spec says, which is `u64`. Two example
sites that read `last_round` as `u64` migrate to `last_round.0` in this
PR.

### Out of scope

- **The `pub use algonaut_algod as openapi_algod` re-exports stay**
  while the migration is in progress. Removing them needs every
  surfaced type wrapped, which is a multi-PR effort. The re-export is
  the escape hatch for callers that need a type the hand-named layer
  hasn't reached yet.
- **`PendingTransactionResponse`, `Account`, `Application`, `Asset`,
  `BlockResponse`, all indexer responses, all kmd responses** — each
  ships as its own follow-up PR. The pattern is the one in this PR:
  add a hand-named struct, wire the conversion at the wrapper edge,
  migrate the public method's return type.

## Consequences

- **Compile-error breaking change** for any caller that named
  `TransactionParams200Response` / `GetStatus200Response` /
  `GetSupply200Response` explicitly. Pre-1.0; mechanical migration.
- **The most-touched method (`txn_params`) is the most-visible win.**
  Every example reads `algod.txn_params()` — the rustdoc now shows
  `SuggestedParams` instead of `TransactionParams200Response`. The
  builders' `build(&params)` argument still type-checks because
  `SuggestedParams: TransactionParams`.
- **The follow-up cuts are mechanical.** Each is one struct + one
  conversion + one method return-type swap. They can land in any order
  and don't conflict structurally; only the `algonaut_model` file may
  see merge conflicts.
- **The umbrella `openapi_algod` re-export retires when full coverage
  lands.** That's the closure condition for the D3 line item in
  `ideal-type-safe-ergonomic-api`.
