---
id: async-signer-trait
title: Async signer trait for remote and interactive signing
abstract: Replace the synchronous Signer contract with an async, group-aware Signer so WalletConnect and custodial/KMS flows can await approval or I/O.
status: proposed
date: 2026-05-21
deciders: []
tags: [api, async, signing]
---

# Async signer trait for remote and interactive signing

## Status

Proposed. Follow-up to [`signer-trait`](signer-trait.md) and the
async-signer out-of-scope note in
[`atomic-transaction-composer-typestate`](atomic-transaction-composer-typestate.md).

## Context

[`signer-trait`](signer-trait.md) opened signing to third parties by
replacing the closed `TransactionSigner` enum with this object-safe
synchronous trait:

```rust
pub trait Signer: std::fmt::Debug + Send + Sync {
    fn sign_transactions(
        &self,
        txs: &[Transaction],
    ) -> Result<Vec<SignedTransaction>, TransactionError>;
}
```

That shape is right for local, immediate signers: `Account`,
`ContractAccount`, and `MultisigSigner` can all synchronously turn a
transaction slice into a matching signed slice. It is the wrong public
contract for remote or interactive signing:

- **WalletConnect** signing involves a network round-trip plus user
  approval in a wallet UI. Blocking the current thread until the user
  approves is hostile to async callers, mobile/browser runtimes, and
  cancellation.
- **Custodial APIs, HSMs, KMSs, and MPC services** are naturally async:
  they may require HTTP calls, policy checks, polling, webhooks, or
  multi-party approval before returning signatures.
- **Wallets need atomic-group context.** The current composer signs one
  slot at a time. A WalletConnect wallet should display the whole
  already-grouped transaction array and sign only the slots controlled
  by that account. Showing a single extracted transaction loses the
  thing the user is approving: the atomic group.
- **Blocking adapters are a trap.** A remote signer can technically
  implement today's `Signer` by calling `block_on` or using a blocking
  HTTP client, but that risks runtime deadlocks, makes wasm/browser
  support impractical, and hides timeout/cancellation semantics from the
  SDK API.

This is still pre-1.0 API work. We do not need to preserve the sync
`Signer` contract for compatibility; the cleaner design is to make
signing async and group-aware at the trait boundary instead of adding a
parallel compatibility layer.

## Decision

### Replace `Signer` with an async, group-aware contract

`Signer` remains the single signing extension point, but its method
changes from synchronous per-slice signing to asynchronous group signing.
The trait must remain object safe because the composer stores signers
behind `Arc<dyn Signer>`, so the public shape uses an explicit future
alias rather than `async fn` directly in the trait:

```rust
use std::{future::Future, pin::Pin};

pub type SigningFuture<'a> = Pin<
    Box<
        dyn Future<Output = Result<Vec<SignedTransaction>, TransactionError>>
            + Send
            + 'a,
    >,
>;

pub struct SigningRequest<'a> {
    /// The full group, after group ids have been assigned.
    pub transactions: &'a [Transaction],
    /// Positions in `transactions` this signer is expected to sign.
    pub indexes: &'a [usize],
}

pub trait Signer: std::fmt::Debug + Send + Sync {
    fn sign_transactions<'a>(
        &'a self,
        request: SigningRequest<'a>,
    ) -> SigningFuture<'a>;
}
```

The returned vector contains one `SignedTransaction` per requested index,
in `request.indexes` order. It remains all-or-error: a signer that
cannot sign any requested transaction returns an error instead of a
partial result.

The request is group-aware so WalletConnect-style implementations can
send the full atomic group to the wallet and mark which slots should be
signed. Local signers can ignore non-requested slots and sign only the
indexed transactions.

### Local signers implement the async trait directly

`Account`, `ContractAccount`, and `MultisigSigner` keep their existing
low-level synchronous signing helpers where those helpers are useful on
their own. Their `Signer` implementations become immediate async
wrappers around those helpers:

```rust
impl Signer for Account {
    fn sign_transactions<'a>(
        &'a self,
        request: SigningRequest<'a>,
    ) -> SigningFuture<'a> {
        Box::pin(async move {
            request.indexes.iter()
                .map(|&i| self.sign_transaction(request.transactions[i].clone()))
                .collect()
        })
    }
}
```

No separate `AsyncSigner` trait and no sync/async enum wrapper are
introduced. There is one signer abstraction, and it is capable of
awaiting.

### Keep signer storage simple

`TransactionWithSigner` continues to carry an optional signer, but the
signer is now the async `Signer` trait object:

```rust
pub struct TransactionWithSigner {
    pub tx: Transaction,
    pub signer: Option<Arc<dyn Signer>>,
}
```

`None` keeps the D7 meaning: an unsigned simulate-only slot that the
composer fills with the all-zero placeholder signature. The constructor
shape can stay the same, but all implementations behind the trait use
the new async contract.

`MethodCall::new(...)` also keeps taking `Arc<dyn Signer>`. The breaking
change is borne by signer implementations and by callers that execute
the signing step, not by an extra `new_async` constructor or a second
trait object type.

### Make composer signing async

`UnsignedAtomicGroup::sign()` becomes async:

```rust
impl UnsignedAtomicGroup {
    pub async fn sign(self) -> Result<SignedAtomicGroup, Error>;
}
```

The async signing flow is:

1. Build the full, group-id-stamped transaction array.
2. For each distinct signer object, collect the indexes it must sign.
3. Call `Signer::sign_transactions(SigningRequest { transactions,
   indexes })` and await the result.
4. Validate that the signer returns exactly one signed transaction per
   requested index, in order, and that each signed transaction wraps the
   expected unsigned transaction.
5. Reassemble the signed group in original transaction order; fill
   `None` slots with the existing all-zero placeholder used by simulate.

The implementation may call signers sequentially at first. Parallel
signing can be added later if we can do it without surprising
WalletConnect users with concurrent approval prompts.

Existing async composer operations that internally sign, such as
`simulate_with`, await the same signing machinery. Callers that want to
simulate without prompting a wallet still attach
`TransactionWithSigner::unsigned(tx)` for those slots, exactly as in D7.

### Validate remote signer output

A remote signer may return signed msgpack blobs rather than SDK-native
`SignedTransaction` values. Implementations may deserialize those blobs,
but the composer must still validate the result against the request:
wrong count, wrong order, wrong transaction, or mismatched transaction id
is a signing error, not something to submit to algod.

## Consequences

- **WalletConnect and custodial signing become first-class.** They can
  await user approval, HTTP calls, polling, or MPC coordination without
  blocking the executor or faking synchronous behaviour.
- **The signing model is simpler than a compatibility bridge.** There is
  one `Signer` trait and one `Arc<dyn Signer>` slot in the composer;
  the API does not grow `AsyncSigner`, `SignerHandle`, or sync-vs-async
  constructor pairs.
- **This is a deliberate breaking change.** Every custom `Signer`
  implementation must switch to the new method signature, and every
  caller of `UnsignedAtomicGroup::sign()` must `.await` it. Pre-1.0,
  that is preferable to carrying a long-term compatibility layer.
- **Low-level local signing can remain synchronous.** `Account` and
  other concrete types may still expose direct synchronous helpers for
  one-off offline signing. Only the pluggable composer-facing `Signer`
  trait becomes async.
- **The boxed future is an object-safety cost.** The trait object shape
  requires allocation/dynamic dispatch for signer futures. That cost is
  negligible next to wallet approval or remote API latency, and local
  signing still has direct helper methods when the caller needs a purely
  synchronous fast path.
- **Async signer errors need structured follow-up.** The first cut can
  reuse `TransactionError` for signer failures, but wallet rejection,
  timeout, malformed remote response, and validation mismatches are
  worth typed variants once the implementation lands.
- **Out of scope.** This ADR does not define a concrete WalletConnect
  client, custodial HTTP protocol, retry policy, or UI callback model.
  It only defines the SDK extension point and composer plumbing those
  integrations need.
