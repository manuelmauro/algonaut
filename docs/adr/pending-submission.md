---
id: pending-submission
title: Finality polling is a client capability
abstract: `algod.submit(&signed)` returns a `PendingSubmission` handle whose `.confirm()` polls to finality and `.confirm_with(Duration)` overrides the default timeout. The three pasted copies of `wait_for_pending_transaction` (per-example, in `step_defs/util.rs`, in `src/util/`) retire. Sixth sub-ADR addressing decision item D2 of the ideal-type-safe-ergonomic-api index.
status: proposed
date: 2026-05-20
deciders: []
tags: [api, ergonomics]
---

# Finality polling is a client capability

## Status

Proposed. Implements decision item **D2** of
[`ideal-type-safe-ergonomic-api`](ideal-type-safe-ergonomic-api.md).

## Context

`src/util/wait_for_pending_tx.rs` exists. Yet `examples/app_create.rs`
and `examples/asset_create.rs` each carried their own verbatim copy of
a `wait_for_pending_transaction` helper, and `tests/step_defs/util.rs`
carried a third. A utility that ships with the crate but is re-pasted
by every caller that needs it is a utility with the wrong shape:
finality polling belongs on the client, not in user code.

The previous helper signature was `wait_for_pending_transaction(algod:
&Algod, tx_id: &TxId) -> Result<PendingTransactionResponse, Error>` —
polls `algod.pending_txn(tx_id)` every 250ms with a 60-second timeout.
Every call site had to handle the two-step `let resp = algod.send_txn(...).await?;
let pending = wait_for_pending_transaction(algod, &resp.tx_id).await?;`
pattern.

## Decision

`Algod` gains a `submit` family that returns a typed handle:

```rust
impl Algod {
    pub async fn submit(&self, txn: &SignedTransaction) -> Result<PendingSubmission, Error>;
    pub async fn submit_txns(&self, txns: &[SignedTransaction]) -> Result<PendingSubmission, Error>;
    pub async fn submit_raw(&self, rawtxn: &[u8]) -> Result<PendingSubmission, Error>;
    pub fn pending_submission(&self, tx_id: TxId) -> PendingSubmission;
}

pub struct PendingSubmission { /* algod handle + tx_id */ }

impl PendingSubmission {
    pub fn tx_id(&self) -> &TxId;
    pub async fn confirm(self) -> Result<PendingTransactionResponse, Error>;
    pub async fn confirm_with(self, timeout: Duration) -> Result<PendingTransactionResponse, Error>;
}
```

`confirm` is `confirm_with(Duration::from_secs(60))` — same 60-second
default as the old helper, same `Error::Msg("Pending transaction timed
out ({timeout:?})")` wording on expiry (so any caller substring-matching
the old message still works). Between checks the implementation awaits
algod's `wait-for-block-after/{round}` long-poll instead of sleeping
on a 250ms timer: the future stays parked until the next block lands,
so the cadence becomes network-driven (~3–4s on mainnet/testnet)
rather than an arbitrary tick. Each unconfirmed iteration costs two
HTTP calls (`pending_txn` + `wait-for-block`), versus ~16 polls per
block under the old timer-driven loop.

The quickstart from the index ADR now reads:

```rust
let confirmed = algod
    .submit(&alice.sign_transaction(txn)?)
    .await?
    .confirm()
    .await?;
```

### What retires

- `src/util/wait_for_pending_tx.rs` — deleted.
- `wait_for_pending_transaction` in `tests/step_defs/util.rs` — deleted.
- The per-example copies in `examples/app_create.rs` and
  `examples/asset_create.rs` — deleted.
- All call sites switch to `submit(...).await?.confirm().await?` or
  `pending_submission(tx_id).confirm().await?` for the "I already have
  a saved tx_id" case.

### What stays

- `send_txn` / `send_raw_txn` / `send_txns` — the raw forms returning
  `RawTransaction200Response`. Anyone who needs the raw response shape
  (or doesn't want the polling) still has the escape hatch.
- The atomic-transaction-composer keeps an **internal**
  `poll_until_confirmed(&Algod, &TxId)` helper. The composer polls on
  `signed_txs[index_to_wait].transaction_id` — *not* the id returned by
  `send_txns` — so the public `PendingSubmission` doesn't quite fit;
  the private helper mirrors `confirm_with`'s shape (60s deadline,
  `wait-for-block-after` between checks) but is scoped to the
  composer's module. Users of the composer never see it.

## Consequences

- **Compile-error breaking change** for any caller that imported
  `algonaut::util::wait_for_pending_transaction` (the path that
  technically existed but was undocumented). The three in-tree copies
  are migrated in this PR.
- **The examples shrink.** `examples/app_create.rs` and
  `examples/asset_create.rs` each lose ~15 lines of pasted polling
  loop.
- **Cadence is now event-driven.** The old helper slept 250ms between
  `pending_txn` calls; the new implementation awaits algod's
  `wait-for-block-after/{round}` instead, so the loop is woken by the
  network rather than a timer. Timeout default (60s) is unchanged —
  only the wait between checks moves from timer-driven to
  event-driven.
- **Pool errors fail fast.** A new failure path: if algod reports a
  non-empty `pool-error` on `pending_txn` (the tx was kicked out of the
  node's pool — expired, underfunded, group invalid, etc.), `confirm`
  returns `Error::PendingTransactionPoolError { reason }` immediately
  rather than waiting out the 60s timeout. The old helper ignored the
  `pool-error` field entirely and surfaced everything as a generic
  timeout, hiding the real cause. Caveat: `pool-error` is a node-local
  view — a tx evicted from *this* node's pool could in principle still
  be alive in peers' pools. The other Algorand SDKs accept the same
  trade-off.
- **Structured error variants** (following the pattern established by
  the structured-errors ADR / PR #301):
  - `Error::PendingTransactionTimeout { timeout: Duration }` replaces
    the old `Error::Msg("Pending transaction timed out (..)")`.
  - `Error::PendingTransactionPoolError { reason: String }` carries
    the algod-reported pool-error verbatim.
  Both apply equally to `PendingSubmission::confirm_with` and the
  composer's internal `poll_until_confirmed`. Callers (and tests)
  match on the variant; the inner message is diagnostic. **Compile-
  error breaking change** for any caller substring-matching the old
  `Error::Msg("Pending transaction timed out ..")` text.
- **Out of scope.** A retry policy / exponential backoff /
  cancellation token would be improvements, but they're behavior
  changes the existing helper didn't have either. If they ever land
  they should land as setter-fluents on `PendingSubmission`
  (`.with_retry_policy(...)`, `.with_cancellation_token(...)`) rather
  than as parameters on `confirm`.
