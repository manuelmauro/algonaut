---
id: transaction-naming-convention
title: 'Transaction naming: omit redundant nouns, spell out the rest'
abstract: Two rules for the transaction noun across handwritten code — omit it where the receiver or argument type already carries it (submit, send, sign, suggested_params), and where it stays (type names, id-based getters, batch methods) spell it out in full, never tx/txn/txid. Rename the shipped TxId/TxGroup newtypes to TransactionId/TransactionGroup. Generated OpenAPI code is out of scope and normalized at the handwritten client edge.
status: accepted
date: 2026-05-22
deciders: []
tags: [api, naming, conventions]
---

# Transaction naming: omit redundant nouns, spell out the rest

## Status

Accepted. Sets a cross-cutting naming convention. It **amends the naming**
of `TxId` and `TxGroup` introduced by
[`identifier-newtypes-at-client-boundary`](identifier-newtypes-at-client-boundary.md)
and builds on the verb naming the north-star
[`ideal-type-safe-ergonomic-api`](ideal-type-safe-ergonomic-api.md) and
[`pending-submission`](pending-submission.md) already settled
(`algod.submit(..)`, `algod.suggested_params()`, `alice.sign(txn)`). It
touches type names from [`closed-signed-transaction`](closed-signed-transaction.md),
[`atomic-module-layout`](atomic-module-layout.md), and
[`one-build-per-transaction`](one-build-per-transaction.md) (the `TxnHeader` /
`TxnBuilder` family).

## Context

"Transaction" is the central noun of this SDK, and the codebase spells it
three different ways depending on where you look. Across the handwritten
(non-generated, non-`target/`) sources the full word already dominates —
`transaction` appears ~628 times, `Transaction` ~154 — against ~116 `tx`,
~4 `Tx`, and ~3 `txn`. The abbreviation is the minority, but it is
concentrated in load-bearing public API, so it is the first thing a new
caller meets.

The split falls along three seams.

### Types: full word, with a few abbreviated holdouts

The domain types spell it out: `Transaction`, `SignedTransaction`,
`TransactionType`, `TransactionWithSigner`, `TransactionError`,
`TransactionSignature`, `TransactionParams`, `SuggestedTransactionParams`,
and every per-kind struct (`AssetConfigurationTransaction`,
`AssetTransferTransaction`, `ApplicationCallTransaction`,
`StateProofTransaction`, …).

Three handwritten public types break the pattern:

| Type        | Definition                                  |
|-------------|---------------------------------------------|
| `TxId`      | `algonaut_core/src/lib.rs:188`              |
| `TxGroup`   | `algonaut_transaction/src/tx_group.rs:24`   |
| `TxnHeader` | `algonaut_transaction/src/builder.rs:26`    |

### Methods and fields: full word up high, abbreviated in the client wrappers

The high-level surface reads in full — `SignedTransaction::transaction()`,
`SignedTransaction::transaction_id()`, `TxGroup::transactions()`. The
handwritten algod/indexer client wrappers do the opposite, in
`src/algod/v2/mod.rs` and `src/indexer/v2/mod.rs`:

```rust
pub async fn send_txn(..)                  // :492
pub async fn send_txns(..)                 // :502
pub async fn send_txn_async(..)            // :560
pub async fn pending_txn(txid: &TxId)      // :470
pub async fn pending_txns(..)              // :334
pub async fn block_txids(round: u64)       // :233
pub async fn txn_proof(.., txid: &TxId)    // :426
pub async fn txn_params(..)                // :672
pub async fn txn_group_state_delta(id: ..) // :288
```

Two distinct problems hide in one name here. `send_txn` both **abbreviates**
(`txn`) and carries a **noun the `&SignedTransaction` argument already
supplies**. The newer API has already dropped that noun:
`algod.submit(&signed) -> PendingSubmission` (`src/algod/v2/mod.rs:523`) was
chosen over `send_transaction` by [`pending-submission`](pending-submission.md),
and the north-star writes `alice.sign(txn)` — though the shipped method is
still `Account::sign_transaction` (`examples/payment.rs:32`). So the
convention has to answer two questions, not one: *whether* the noun belongs
in a name, and *how to spell it* when it does.

### Identifiers: one concept, four spellings

The single notion "transaction id" is written four ways:

- `TxId` — the newtype (`algonaut_core/src/lib.rs:188`);
- `transaction_id` — high-level fields and accessors
  (`SignedTransaction::transaction_id()`);
- `tx_id` — fields and locals (`pending_submission.rs:16`,
  `src/atomic/outcome.rs`, `src/atomic/group.rs`);
- `txid` — wrapper parameters, no separator (`src/algod/v2/mod.rs:429,470`,
  five sites in `src/indexer/v2/mod.rs`).

There is currently **no** `TransactionId`, `TxnId`, or `txn_id` anywhere —
the inconsistency is between these four spellings, not more.

### Generated code is a separate population

`algonaut_algod` and `algonaut_indexer` `src/models/` and `src/apis/` are
generated from the OpenAPI specs. They follow upstream naming: full
`transaction` in method names (`get_pending_transactions`,
`raw_transaction`, `search_for_transactions`), `txn_` field prefixes, and
their own abbreviations — the indexer `TxType` enum
(`algonaut_indexer/src/models/transaction.rs:201`), the `txid: String`
model fields, `DryrunTxnResult`. These are regenerated wholesale per
[`openapi-client-regeneration`](openapi-client-regeneration.md) and are not
ours to hand-edit.

The result is that the same value crosses three naming styles on one call
path — `algod.pending_txn(txid)` (wrapper) → `get_pending_transaction`
(generated) → `PendingTransactionResponse` (generated) — and a caller has
to learn all three.

## Decision

Two rules, applied in that order. The first decides **whether** the noun
belongs in the name at all; the second decides **how to spell** it when it
stays.

### Rule 1 — omit the noun when the receiver or argument type carries it

A name does not repeat what its receiver or argument type already says
(`Vec::push`, not `push_element`). For verb methods that take or return a
transaction, the `&SignedTransaction` / `Transaction` value *is* the noun, so
the method drops it. The project already does this with `submit`, which it
chose over `send_transaction`:

| Today                                  | After                          | Why the noun goes                                   |
|----------------------------------------|--------------------------------|-----------------------------------------------------|
| `submit` / `submit_raw`                | *(unchanged — the model)*      | arg is `&SignedTransaction` / `&[u8]`               |
| `send_txn` / `send_txn_async`          | `send` / `send_async`          | arg is `&SignedTransaction`                         |
| `send_raw_txn` / `send_raw_txn_async`  | `send_raw` / `send_raw_async`  | arg is raw bytes; `_raw` is the real disambiguator  |
| `Account::sign_transaction` / `Kmd::sign_transaction` | `sign`          | arg is a `Transaction`; what else does it sign?     |
| `txn_params`                           | `suggested_params`             | returns `SuggestedParams` — not a transaction at all |

`txn_params` is the clearest case: the value is not a transaction, so the
word is dropped (not spelled out), and `suggested_params` is the name the
north-star already uses.

### Rule 2 — where the noun stays, spell it out in full

Type names, id-based getters that disambiguate among sibling queries, and
batch methods (where the verb alone cannot carry the plural) keep the noun.
Where it stays, it is written `transaction`, never `tx` / `txn` / `txid`.
Three corollaries:

1. **Public types, traits, and named fields spell it out.** No `Tx`, `Txn`,
   `txn`, or `txid` in any public identifier.
2. **`tx` is allowed only as a private, local binding** — loop variables and
   short-lived locals (`for tx in group` in `src/atomic/group.rs`). It never
   appears in a signature, a field, or anything `pub`; `txn` and `txid` are
   not used even as locals.
3. **One spelling for the id: `transaction_id` / `TransactionId`.** Every
   field, parameter, and accessor holding one is named `transaction_id`;
   `tx_id` and `txid` retire.

#### Types

| Today       | After                | Notes                                                          |
|-------------|----------------------|----------------------------------------------------------------|
| `TxId`      | `TransactionId`      | `algonaut_core`; `#[serde(transparent)]` over `String` is unchanged |
| `TxGroup`   | `TransactionGroup`   | `algonaut_transaction`; the module `tx_group` → `group` (the crate already carries the noun, so `transaction::group::TransactionGroup` reads without repetition) |
| `TxnHeader` | `TransactionHeader`  | `algonaut_transaction`; applies to whatever shape [`one-build-per-transaction`](one-build-per-transaction.md) leaves of the header family |

The per-kind transaction structs and every other `Transaction*` type already
comply and do not change.

#### Getters and batch methods

| Today                                | After                                          |
|--------------------------------------|------------------------------------------------|
| `pending_txn` / `pending_txns`       | `pending_transaction` / `pending_transactions` |
| `block_txids`                        | `block_transaction_ids`                        |
| `txn_proof`                          | `transaction_proof`                            |
| `txn_group_state_delta[_for_round]`  | `transaction_group_state_delta[_for_round]`    |
| `submit_txns`                        | `submit_transactions`                          |
| `send_txns` / `send_txns_async`      | `send_transactions` / `send_transactions_async`|
| param `txid: &TxId` / `tx_id: &TxId` | `transaction_id: &TransactionId`               |

`PendingSubmission`'s `tx_id` field and `tx_id()` accessor
(`src/algod/v2/pending_submission.rs:16,25`) and the `tx_id` fields in
`src/atomic/outcome.rs` likewise become `transaction_id`.

### Generated OpenAPI code is out of scope

The convention governs handwritten code only. Generated `algonaut_algod` /
`algonaut_indexer` identifiers (`TxType`, `txid: String`, `DryrunTxnResult`,
`txn_` field prefixes) are left as the generator emits them; chasing them
would mean a post-generation rewrite step in the regeneration pipeline,
which [`openapi-client-regeneration`](openapi-client-regeneration.md)
deliberately keeps thin and diff-able. Normalization happens at the
handwritten edge: the `ext/` wrappers and the `src/algod` / `src/indexer`
client methods present full-word (or noun-free) names and convert to the
generated names internally, exactly as they already convert
`&TransactionId` ↔ `&str` at the HTTP boundary.

## Consequences

- **Breaking change to shipped public types and methods.** `TxId`, `TxGroup`,
  the renamed wrapper methods (`send_txn → send`, `pending_txn →
  pending_transaction`, …), and the `txid` / `tx_id` parameters are all
  public and used in real call sites. The crate is pre-1.0 and the README
  says the API still moves; this rides the same in-flight D-series breakage as
  [`identifier-newtypes-at-client-boundary`](identifier-newtypes-at-client-boundary.md).
  Migration is mechanical and rename-only: no behaviour, no wire format, and
  no `#[serde(transparent)]` representation changes.

- **Singular drops the noun; the batch keeps it.** `submit` / `send` name the
  one-transaction case where the type carries the noun, but `submit_transactions`
  / `send_transactions` need it back — the verb alone cannot carry the plural.
  The asymmetry is deliberate: each name is individually the clearest, and the
  `submit` / `submit_txns` pair already had this shape before the rename.

- **`submit` and `send` coexist as different return shapes.** `submit` returns
  a `PendingSubmission` handle; `send` returns the raw `*200Response`. That
  high-level/raw split is the one [`pending-submission`](pending-submission.md)
  settled — now with both the redundant noun and the abbreviation gone from
  the raw form.

- **It resolves a live inconsistency.** `Account::sign_transaction`
  (`examples/payment.rs:32`) versus the north-star's `alice.sign(txn)`: Rule 1
  settles it toward `sign`, which `LogicSig` / contract-account signing
  already uses (`examples/logic_sig_contract_account.rs:50`).

- **It amends, not supersedes, the identifier-newtypes ADR.** That ADR's
  decision stands in full — newtypes at the client boundary, `TxGroup` as the
  grouped batch with its `TryFrom<Vec<Transaction>>` ctor. Only the two
  *names* `TxId` and `TxGroup` are revised here, to `TransactionId` and
  `TransactionGroup`. Its status stays `accepted`; its prose should be read
  with this rename applied.

- **The four-way id spelling collapses to one.** After this, `transaction_id`
  is the only way the concept is written — searchable, greppable, and
  unambiguous. `tx_id`, `txid`, and `TxId` all disappear from handwritten
  code.

- **`tx` survives where it earns its keep.** Local bindings in tight loops
  stay readable (`for tx in group`) without the line-length cost of the full
  word, and the rule is bright-line: if it is `pub` or it is a field or a
  signature, omit the noun or spell it out; otherwise `tx` is fine. This also
  sidesteps the `tx`/`rx` channel-half ambiguity — `tx` only ever appears in
  local scope, never as a durable API name.

- **Call sites to update.** The renames touch `src/algod/v2/`,
  `src/indexer/v2/`, `src/atomic/`, the examples (`examples/*.rs`), and the
  cucumber step-defs that drive the client — applied everywhere in one pass,
  test scaffolding included, per the project's migration-consistency rule.

- **Generated code stays mixed, by design.** `TxType` and the `txn_` model
  fields remain abbreviated. The cost is a visible seam at the `ext/`
  wrappers; the benefit is that regeneration stays a clean overwrite with no
  bespoke rename pass to maintain. If hiding the generated types (the D3
  follow-up in the index ADR) lands, that seam disappears behind hand-named
  response types anyway.

- **Follow-up wording fixes.** Prose in
  [`identifier-newtypes-at-client-boundary`](identifier-newtypes-at-client-boundary.md),
  [`atomic-module-layout`](atomic-module-layout.md) (the `tx_ids` helper in
  `signing.rs`), and [`one-build-per-transaction`](one-build-per-transaction.md)
  references the old names; those are corrected as the rename lands, not as a
  separate effort.
