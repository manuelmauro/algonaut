---
id: external-signature-ingress
title: Constructing SignedTransaction from an external signature
abstract: Add a public ingress so an out-of-crate signer can turn a signature produced elsewhere into a SignedTransaction — a SignedTransaction::with_signature constructor for the raw-signature case and a corrected msgpack-deserialize path for fully-formed blobs. Amends D5 (closed-signed-transaction), whose pub(crate) fields and constructor-free design block third-party HSM, KMS, and remote signers from satisfying the Signer trait.
status: proposed
date: 2026-05-21
deciders: []
tags: [api, signing, type-safety]
---

# Constructing SignedTransaction from an external signature

## Status

Proposed. Amends [`closed-signed-transaction`](closed-signed-transaction.md)
(**D5**) and builds on [`signer-trait`](signer-trait.md) (**D7**).
Prerequisite for [`async-signer-trait`](async-signer-trait.md): that ADR's
remote signers cannot produce their return value without the ingress
decided here.

## Context

[`signer-trait`](signer-trait.md) (D7) made signing an open trait so that
third parties — Ledger, a remote KMS, a custodial API, an MPC service —
can plug in without an `algonaut` release. The trait's contract is to
return `SignedTransaction`s:

```rust
pub trait Signer: std::fmt::Debug + Send + Sync {
    fn sign_transactions(
        &self,
        txs: &[Transaction],
    ) -> Result<Vec<SignedTransaction>, TransactionError>;
}
```

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
use — no special access to closed fields needed." That is only true for a
signer that owns an `Account`-shaped private key and can call
`Account::sign_transaction`. A genuine remote signer holds neither the key
nor an in-crate signing primitive. It holds one of two things:

- **A raw signature** — the common HSM/KMS/MPC shape: hand the service the
  transaction bytes, get back 64 signature bytes. There is **no public
  way** to attach those bytes to a `Transaction` and obtain a
  `SignedTransaction`. `TransactionSignature` is public, but the struct
  that wraps it is not constructible.
- **A fully-formed signed msgpack blob** — some custodial APIs return the
  whole encoded signed transaction. A public `impl Deserialize for
  SignedTransaction` exists (via `ApiSignedTransaction`), so this *looks*
  available. It is not safe:

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
  format carries no `txid` — so after deserializing a real blob it is
  `TxId::default()`, i.e. **empty**. The path also hardcodes
  `auth_address: None`, discarding the `sgnr` field it just decoded. So
  the one public construction path produces a `SignedTransaction` whose id
  is wrong and whose rekey authoriser is lost — exactly the class of
  invariant violation D5 set out to prevent, hiding behind the
  `Deserialize` impl.

This gap is **orthogonal to async**. It already blocks the *accepted
synchronous* `Signer` trait; an HSM signer can't satisfy D7 today. The
[`async-signer-trait`](async-signer-trait.md) ADR surfaces it because its
motivating signers are all remote, but the fix belongs to D5's invariant,
not to the async change, which is why it is recorded here rather than
folded into that ADR.

## Decision

Keep `SignedTransaction`'s fields `pub(crate)`. Add two vetted public
ingresses, each of which **recomputes the id from the transaction** so
D5's "the id is always the hash of the carried transaction" invariant
holds for every construction path, in-crate or not.

### 1. `SignedTransaction::with_signature` for the raw-signature case

```rust
impl SignedTransaction {
    /// Wrap a `Transaction` in a signature produced outside this crate
    /// (a remote KMS/HSM, an MPC service, a hardware wallet).
    ///
    /// `transaction_id` is computed from `transaction`; the caller cannot
    /// supply it. `signer_address` is the address whose key produced the
    /// signature, used to derive the rekey `auth_address` exactly as the
    /// built-in signing paths do.
    ///
    /// This trusts that `sig` is a valid signature over `transaction`; it
    /// does not verify the cryptography. Verification, if wanted, is the
    /// signer's job.
    pub fn with_signature(
        transaction: Transaction,
        sig: TransactionSignature,
        signer_address: Address,
    ) -> Result<SignedTransaction, TransactionError> {
        let transaction_id = transaction.id()?;
        let auth_address = auth_address(&transaction, &signer_address);
        Ok(SignedTransaction { transaction, transaction_id, sig, auth_address })
    }
}
```

It reuses the existing `pub(crate) fn auth_address(tx, signing_address)`
helper, so a rekeyed remote signer gets the same `sgnr` handling as
`Account::sign_transaction`. The constructor is signature-agnostic: a
`TransactionSignature::Single`, `Multi`, or `Logic` all go through the
same door. Passing the sender's own address yields `auth_address = None`,
the non-rekeyed common case.

### 2. Correct the deserialize path for the full-blob case

The msgpack→`SignedTransaction` path becomes a *blessed* ingress by being
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

After this, a remote signer that returns a full signed blob can
`rmp_serde::from_slice::<SignedTransaction>(&blob)?` and get a value whose
id and authoriser are correct.

### What stays closed

The fields remain `pub(crate)`. `transaction_id` remains uncomputable by
the caller. The `#[doc(hidden)] placeholder(tx)` helper from D5 is
unchanged. The only new surface is `with_signature` and the *behavioral
correction* of an already-public `Deserialize` impl — every one of the
now-four construction paths derives the id rather than accepting it.

## Consequences

- **D7's third-party signers become buildable.** An HSM/KMS/MPC/remote
  signer can finally return `Vec<SignedTransaction>` from outside
  `algonaut_transaction`: raw signature via `with_signature`, full blob
  via the corrected deserialize. This unblocks both the *accepted
  synchronous* `Signer` trait and the proposed
  [`async-signer-trait`](async-signer-trait.md).
- **D5's invariant is preserved, not weakened.** Opening construction did
  not reopen the id footgun: every ingress recomputes `transaction_id`
  from the transaction. There is still no way to store a mismatched id.
- **Two latent bugs are fixed.** Deserializing any real signed-transaction
  blob previously produced an empty `transaction_id` and silently dropped
  the rekey `auth_address`. Both are corrected here. Any code that relied
  on the empty-id behavior was already wrong.
- **`with_signature` trusts the bytes.** It performs no signature
  verification — a caller can wrap a `Transaction` in a signature that
  doesn't match it. That is the signer's responsibility. The composer's
  request-side validation (count, order, id match — defined in
  [`async-signer-trait`](async-signer-trait.md)) catches a signer that
  returns the *wrong transaction*, but neither layer checks cryptographic
  validity; algod is the final arbiter.
- **Additive and low-churn.** A new `pub fn` plus a corrected `TryFrom`.
  No existing in-crate signing path changes. Pre-1.0, the deserialize
  correction needs no shim.
- **Out of scope.** Closing `SignedLogic`'s still-public fields remains
  D5's open follow-up. Signature verification, remote transport, and the
  async trait shape are decided elsewhere.
