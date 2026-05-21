---
id: external-signature-ingress
title: Constructing SignedTransaction from an external signature
abstract: Bless and correct the existing msgpack-deserialize path as the single public ingress by which an out-of-crate signer turns externally-produced signed bytes into a SignedTransaction. Recompute transaction_id from the decoded transaction and preserve the decoded sgnr, fixing a path that today yields an empty id and drops the rekey auth address. Add no new public constructor — the fix reopens nothing D5 closed and matches how reference SDKs return signed transactions as bytes.
status: accepted
date: 2026-05-21
deciders: []
tags: [api, signing, type-safety]
---

# Constructing SignedTransaction from an external signature

## Status

Accepted. Amends [`closed-signed-transaction`](closed-signed-transaction.md)
(**D5**) and builds on [`signer-trait`](signer-trait.md) (**D7**).
Prerequisite for [`async-signer-trait`](async-signer-trait.md): that ADR's
remote signers cannot produce their return value without the ingress
decided here.

## Context

[`signer-trait`](signer-trait.md) (D7) made signing an open trait so that
third parties — Ledger, a remote KMS, a custodial API, an MPC service —
can plug in. The trait's contract is to return `SignedTransaction`s.
[`closed-signed-transaction`](closed-signed-transaction.md) (D5) then made
`SignedTransaction`'s fields `pub(crate)` with no public constructor, so
the id can't be stored wrong:

```rust
pub struct SignedTransaction {
    pub(crate) transaction: Transaction,
    pub(crate) transaction_id: TxId,
    pub(crate) sig: TransactionSignature,
    pub(crate) auth_address: Option<Address>,
}
```

These two accepted decisions contradict each other for the case D7 exists
to serve. D5's own consequence claims a third-party HSM signer "produces
`SignedTransaction`s through the same signing API the built-in signers
use." That is only true for a signer that owns an `Account`-shaped key and
can call `Account::sign_transaction`. A genuine remote signer holds
neither the key nor an in-crate signing primitive — it holds bytes that
came back from somewhere else, and cannot turn them into the closed type.

This gap is **orthogonal to async**: it already blocks the *accepted
synchronous* `Signer` trait. [`async-signer-trait`](async-signer-trait.md)
surfaces it because its motivating signers are all remote, but the fix
belongs to D5's construction story, which is why it is recorded here.

### The bytes are the lingua franca

What does a remote signer actually return? In every realistic case, a
fully-formed signed transaction encoded as canonical Algorand msgpack:

- **WalletConnect** wallets return signed transaction bytes — that is the
  wire contract.
- **Custodial APIs and KMS front-ends** return the encoded signed
  transaction.

This matches the reference SDKs: `go-sdk` and `js-sdk` define a signer as
something that returns `[][]byte` — raw signed-transaction bytes — not a
structured type. Bytes, not a hand-built struct, are the universal output
of signing.

algonaut already has a public path for those bytes: `impl Deserialize for
SignedTransaction` (via `ApiSignedTransaction`). So decoding a blob
*looks* available. It is not safe today:

```rust
// api_model.rs — TryFrom<ApiSignedTransaction> for SignedTransaction
Ok(SignedTransaction {
    transaction: api_t.transaction.clone().try_into()?,
    transaction_id: api_t.transaction_id.clone(), // see below
    sig: transaction_signature(&api_t)?,
    auth_address: None,                            // drops the wire sgnr
})
```

`ApiSignedTransaction.transaction_id` is `#[serde(skip)]` — the wire
format carries no `txid` — so after decoding a real blob it is
`TxId::default()`, i.e. **empty**. The path also hardcodes
`auth_address: None`, discarding the `sgnr` field it just decoded. So the
one public construction path produces a `SignedTransaction` whose id is
wrong and whose rekey authoriser is lost — exactly the invariant violation
D5 set out to prevent, hiding behind the `Deserialize` impl.

## Decision

Make the corrected deserialize path the **single** public ingress for
externally-produced signatures. Keep `SignedTransaction`'s fields
`pub(crate)`; add no new constructor.

### Correct the deserialize path, and bless it

The msgpack→`SignedTransaction` path becomes a supported ingress by being
made correct:

- recompute `transaction_id` from the decoded transaction
  (`Transaction::id()`), never from the skipped wire field;
- carry the decoded `auth_address` (the `sgnr` field) instead of
  hardcoding `None`.

```rust
fn try_from(api_t: ApiSignedTransaction) -> Result<Self, Self::Error> {
    let transaction: Transaction = api_t.transaction.clone().try_into()?;
    let transaction_id = transaction.id()?;          // was: api_t.transaction_id
    Ok(SignedTransaction {
        transaction,
        transaction_id,
        sig: transaction_signature(&api_t)?,
        auth_address: api_t.auth_address,            // was: None
    })
}
```

A remote signer's `Signer` implementation then turns its bytes into the
trait's return value with one line:

```rust
let signed: SignedTransaction = rmp_serde::from_slice(&blob)?;
```

The id is recomputed from the decoded transaction, so the value satisfies
D5's invariant regardless of what the blob claimed. This is the only
construction-from-outside the crate offers; the `#[doc(hidden)]
placeholder(tx)` helper and the in-crate signing paths from D5 are
unchanged.

### Why only this, and not a `with_signature` constructor

An earlier draft also added `SignedTransaction::with_signature(tx, sig,
addr)` for signers that hold a *raw* signature (64 bytes, no envelope)
rather than a full blob. It is dropped, for three reasons:

1. **It adds public surface that reopens what D5 narrowed.** D5's whole
   point was that `SignedTransaction` is not freely constructible. A
   public `with_signature` is a free constructor; the deserialize fix is
   not — it is a correction to a path that already exists and already must
   exist (algod responses, stored transactions, and tests all decode
   signed transactions).
2. **The realistic signers don't need it.** WalletConnect and custodial
   APIs return bytes, not bare signatures. The bytes path serves them
   directly and matches the reference SDKs.
3. **It would not actually save the hard part.** A pure raw-signature
   signer still has to assemble a *canonical* signed-transaction encoding
   — and canonical msgpack (key ordering, omitempty, integer width) is the
   error-prone part. `with_signature` produced a `SignedTransaction` whose
   canonical `ToMsgPack` then did that encoding, but a signer that can
   build a canonical blob can equally hand that blob to `from_slice`.

### Raw-signature-only signers are a deferred follow-up

A signer whose backend returns *only* a bare signature (some HSMs, some
MPC coordinators) and cannot itself produce a canonical signed-transaction
blob is **not fully served** by this ADR: it would have to reconstruct the
Algorand wire format by hand. If such a signer turns out to be a real
need, the right answer is a dedicated, tested **canonical encoder** — a
helper that takes `(Transaction, TransactionSignature, signer_address)`
and returns canonical bytes (or, equivalently, a narrowly-scoped
constructor whose output is only reachable as bytes). That is a separate
decision, made when a concrete raw-signature integration exists, rather
than speculative surface added now. This ADR records the limitation
explicitly instead of pretending the deserialize path covers it.

## Consequences

- **D7's blob-returning signers become buildable.** A WalletConnect or
  custodial signer can now satisfy `Signer` from outside
  `algonaut_transaction`: `from_slice::<SignedTransaction>(&blob)?`. This
  unblocks both the accepted synchronous trait and the proposed
  [`async-signer-trait`](async-signer-trait.md), and it aligns with the
  bytes-in/bytes-out shape the reference SDKs use.
- **D5's invariant is preserved, and the surface does not grow.** No new
  public constructor; fields stay `pub(crate)`; `transaction_id` is
  recomputed on the one corrected path, so it still cannot be stored
  wrong. Deserialize is acknowledged as a blessed, vetted construction
  path rather than an accidental hole.
- **Two latent bugs are fixed.** Decoding any real signed-transaction blob
  previously produced an empty `transaction_id` and silently dropped the
  rekey `auth_address`. Both are corrected. Any code relying on the
  empty-id behavior was already wrong.
- **The deserialize path trusts the bytes.** It performs no signature
  verification — a blob can carry a signature that doesn't match its
  transaction. The composer's request-side validation (count, order, id
  match — defined in [`async-signer-trait`](async-signer-trait.md))
  catches a signer that returns the *wrong transaction*; cryptographic
  validity is algod's call.
- **Raw-signature-only signers remain unserved, on purpose.** Documented
  as a deferred canonical-encoder follow-up rather than half-built now.
- **Alternative considered: a bytes-typed trait return.** Since every
  remote signer ends up producing bytes and decoding them, the trait could
  return `Vec<Vec<u8>>` (as `go-sdk`/`js-sdk` do) and let the composer
  decode centrally, avoiding per-signer `from_slice`. That reshapes the
  [`async-signer-trait`](async-signer-trait.md) signature and is out of
  scope here; this ADR keeps the `Vec<SignedTransaction>` return and makes
  the decode a one-liner inside each remote impl.
- **Additive and low-churn.** A corrected `TryFrom`; no in-crate signing
  path changes. Pre-1.0, the correction needs no shim.
- **Out of scope.** Closing `SignedLogic`'s still-public fields remains
  D5's open follow-up. Signature verification, remote transport, and the
  async trait shape are decided elsewhere.
