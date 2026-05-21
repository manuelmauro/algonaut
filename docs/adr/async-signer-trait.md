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
Depends on [`external-signature-ingress`](external-signature-ingress.md) for
the public path that lets an out-of-crate signer turn a remote signature
into a `SignedTransaction`. That gap is orthogonal to async — it already
blocks the accepted synchronous trait — so it is decided separately and
treated here as a prerequisite, not re-litigated.

This ADR makes two coupled changes: signing becomes **async** (so a signer
can await wallet approval or remote I/O) and **group-aware** (so a wallet
sees the whole atomic group and signs only its slots). They ship together
because neither half alone serves the WalletConnect case that motivates
the work: async-but-per-slot loses the group context the wallet needs to
show, and group-aware-but-blocking still deadlocks the executor.

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

The future is hand-spelled as a `Pin<Box<dyn Future + Send + 'a>>` alias
rather than written with `async fn` in the trait on purpose: native
`async fn` in traits is stable but `dyn`-incompatible, so a `dyn Signer`
behind an `Arc` could not name the method. The explicit boxed alias keeps
the trait object safe without pulling in the `async-trait` or
`trait-variant` proc-macro machinery.

The returned vector contains one `SignedTransaction` per requested index,
in `request.indexes` order. It remains all-or-error: a signer that
cannot sign any requested transaction returns an error instead of a
partial result.

The request is group-aware so WalletConnect-style implementations can
send the full atomic group to the wallet and mark which slots should be
signed. Local signers can ignore non-requested slots and sign only the
indexed transactions.

### The `Send` bound and single-threaded runtimes

`SigningFuture` is `+ Send`, and `Signer` keeps the `Send + Sync` bound it
has today. That is what lets `UnsignedAtomicGroup::sign()` stay `Send`, so
the composer's signing step can run inside `tokio::spawn`-ed,
multi-threaded tasks — the common native case, and the path of least
surprise there.

The cost is real and worth naming, because it is exactly the wasm/browser
scenario the Context invokes. A WalletConnect signer built on
`wasm-bindgen` JS interop typically yields a `!Send` future, which cannot
satisfy `SigningFuture: Send`. So async removes the `block_on` deadlock
problem for wasm, but the `Send` bound does not by itself make the trait
usable from a `!Send` browser signer — the two points in the Context
bullet are not the same win.

We take the `Send` bound for the first cut: the native multi-threaded
executor is the primary target, and a non-`Send` composer would surprise
those callers. A `?Send` variant — a second future alias behind a feature
flag, or a `maybe-send` shim — is the known escape hatch if and when a
real wasm WalletConnect integration lands. It is deferred deliberately,
not by oversight, rather than designed speculatively now.

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
2. Group slot indexes by signer **identity** — `Arc::ptr_eq` on the
   `Arc<dyn Signer>` — so every slot a given signer instance owns is
   collected into one request and signed in one call.
3. Call `Signer::sign_transactions(SigningRequest { transactions,
   indexes })` and await the result.
4. Validate the result against the request: exactly one signed
   transaction per requested index — no missing, duplicate, or extra
   entries — in `request.indexes` order, each wrapping the *expected*
   unsigned transaction (the id recomputed from the request's transaction
   must match the returned id). A signer returning a valid signed
   transaction for the *wrong* transaction is the case this guards
   against. How a signer constructs those values is decided in
   [`external-signature-ingress`](external-signature-ingress.md).
5. Reassemble the signed group in original transaction order; fill
   `None` slots with the existing all-zero placeholder used by simulate.

**Grouping by `Arc::ptr_eq` is deliberate, and has a caller-visible
consequence.** The trait exposes no address or key identity — multisig and
HSM signers cannot supply a sender, which is the D6/D7 rationale — so
pointer identity is the only thing the composer can group on. Two slots
built from *separate* `Arc::new(signer)` calls for the same logical wallet
are therefore two signers, and produce two approval prompts. The caller
contract is: **share one `Arc<dyn Signer>` clone across every slot a
single wallet owns.** That is consistent with
[`atomic-transaction-composer-typestate`](atomic-transaction-composer-typestate.md),
which already leans on `Arc<dyn Signer>` being cheap to clone.

The implementation may call signers sequentially at first. Parallel
signing can be added later if we can do it without surprising
WalletConnect users with concurrent approval prompts.

### Simulate does not invoke real signers

`UnsignedAtomicGroup::simulate`/`simulate_with` borrow `&self` so the group
survives to be signed and executed afterwards — the property
[`atomic-transaction-composer-typestate`](atomic-transaction-composer-typestate.md)
introduced. Async interactive signers turn that property into a trap if
simulate runs the real signing flow: dry-running a group that carries a
WalletConnect signer would pop an approval prompt in the user's wallet,
and the "attach `unsigned(tx)` instead" workaround would force the caller
to build a *different* group for simulate than for execute, defeating the
one-group flow.

So simulate does **not** call `Signer::sign_transactions`. It signs every
slot — `Some(signer)` and `None` alike — with the all-zero placeholder
signature (the same mechanism the `None` slots already use), setting
algod's allow-empty-signatures option as needed, and never awaits a real
signer. Simulate stays cheap and prompt-free; only `sign()` and the
`execute` path that follows it exercise real signers. A caller that
specifically needs to simulate the *actually-signed* group can
`sign().await?` first and simulate the signed group — but that is the
explicit, prompt-incurring choice, not the default. This is a behavioral
change from the pre-async composer, which signed `Some(signer)` slots
during simulate.

### Constructing remote signer output is a prerequisite, not part of this ADR

A remote signer returns its result as bytes: a canonical signed-transaction
msgpack blob, which is what WalletConnect wallets and custodial APIs hand
back (and what the Go/JS reference SDKs treat as signer output). Turning
those bytes into the `SignedTransaction` this trait returns is
`rmp_serde::from_slice::<SignedTransaction>(&blob)?` — but that decode path
is closed and currently buggy under
[`closed-signed-transaction`](closed-signed-transaction.md) (D5).
[`external-signature-ingress`](external-signature-ingress.md) blesses and
corrects it as the single public construction ingress. This ADR assumes it
exists; without it, the async trait's own motivating signers cannot produce
the `Vec<SignedTransaction>` the method returns. The composer's
responsibility, regardless of how the value was decoded, is the
request-side validation in step 4 above: it never trusts a returned
transaction it didn't ask for.

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
- **One wallet, one prompt — if the caller cooperates.** Because slots
  are grouped by `Arc::ptr_eq`, sharing a single `Arc<dyn Signer>` clone
  across a wallet's slots yields one approval round-trip; building
  separate `Arc`s for the same wallet yields several. The headline
  ergonomic win is contingent on that caller contract, which the API
  cannot enforce.
- **Simulate stops signing for real.** Simulate now always uses the
  placeholder signature, even for slots that carry a real signer, so a
  dry-run never prompts a wallet. This is a behavioral change from the
  pre-async composer, which signed `Some(signer)` slots during simulate.
- **The `Send` bound excludes `!Send` wasm signers for now.** Native
  multi-threaded executors get a `Send` composer; a single-threaded
  browser WalletConnect signer needs the deferred `?Send` variant. The
  trade is made deliberately, not by omission.
- **The remote-signer construction path is a separate, prerequisite
  decision.** [`external-signature-ingress`](external-signature-ingress.md)
  amends D5 to make `SignedTransaction` buildable from an external
  signature. This ADR is not implementable without it.
- **Out of scope.** This ADR does not define a concrete WalletConnect
  client, custodial HTTP protocol, retry policy, or UI callback model.
  It only defines the SDK extension point and composer plumbing those
  integrations need.
