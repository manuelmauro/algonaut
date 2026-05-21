---
id: closed-signed-transaction
title: SignedTransaction constructable only via signing
abstract: Close `SignedTransaction`'s pub fields, compute `transaction_id` from the transaction inside the signing paths, and add a `MultisigSigningSession` builder + `SignedLogic::sign` convenience so no example writes a `SignedTransaction` struct literal. Fifth sub-ADR addressing decision item D5 of the ideal-type-safe-ergonomic-api index.
status: accepted
date: 2026-05-20
deciders: []
tags: [api, ergonomics, type-safety, signing]
---

# SignedTransaction constructable only via signing

## Status

Accepted. Implements decision item **D5** of
[`ideal-type-safe-ergonomic-api`](ideal-type-safe-ergonomic-api.md).
Depends on **D7** ([`signer-trait`](signer-trait.md)) — the
`MultisigSigningSession` builder shares its accumulation logic with the
`MultisigSigner: Signer` impl from that ADR.

## Context

`examples/multi_sig.rs`, `examples/logic_sig_delegated.rs`,
`examples/logic_sig_delegated_multi.rs`, and the simulate step-defs all
build `SignedTransaction` literals:

```rust
SignedTransaction {
    transaction: t,
    transaction_id: "".to_owned(),   // placeholder
    sig,
    auth_address: None,
}
```

`transaction_id` is a derived value — the SHA-512/256 hash of the
transaction — yet the type lets callers store an empty string there. A
struct whose invariants are not enforced by its constructor will, given
enough call sites, be constructed wrong.

## Decision

`SignedTransaction`'s fields close to `pub(crate)`. Public access goes
through borrow-returning accessors:

```rust
impl SignedTransaction {
    pub fn transaction(&self) -> &Transaction;
    pub fn transaction_id(&self) -> &TxId;
    pub fn sig(&self) -> &TransactionSignature;
    pub fn auth_address(&self) -> Option<&Address>;
}
```

`transaction_id` is no longer a constructor argument — every signing
path computes it from `Transaction::id()` once when the
`SignedTransaction` is built.

### `MultisigSigningSession` builder

A fluent multi-step session for accumulating multisig signatures:

```rust
let signed = MultisigSigningSession::new(addr)
    .sign(txn, &alice)?       // -> InProgressMultisigSigningSession
    .sign_more(&bob)?         // -> InProgressMultisigSigningSession
    .finish()?;               // -> SignedTransaction
```

Two-state typestate: the first `sign(tx, &account)` consumes a
`MultisigSigningSession` (which only holds an address) and returns an
`InProgressMultisigSigningSession` (which now holds an address + a
transaction + a partial msig). Subsequent `sign_more(&account)?` calls
chain on the in-progress state. `finish()?` produces the
`SignedTransaction`.

The one-shot `MultisigSigner: Signer` impl from D7 keeps working — it
internally drives the same session, so the composer's per-slot signing
path is unchanged.

### `SignedLogic::sign` convenience

`SignedLogic` (used by the delegated logic-sig flow) gains a
`SignedLogic::sign(self, transaction) -> SignedTransaction` method so
`examples/logic_sig_delegated.rs` and friends stop writing struct
literals.

`SignedLogic`'s own `pub` fields stay open in this PR — closing them
needs a `LogicSigSession`-style entry point that's a follow-up. The
`SignedLogic { logic, args, sig }` literal still exists in two examples
and `tests/test_logic_signature.rs`; both produce `SignedLogic` for
`LogicSignature::DelegatedSig` / `DelegatedMultiSig`, which D5 leaves
untouched.

### `placeholder(tx)` for the composer

The atomic-transaction-composer's "unsigned slot" path (added in D7 to
replace the old `TransactionSigner::Empty` variant) needs to construct a
`SignedTransaction` with the all-zero placeholder signature from inside
the `algonaut` umbrella crate. `algonaut_transaction::signed_transaction::placeholder(tx)`
— `#[doc(hidden)] pub` — is the cross-crate ingress. Users never see
it; the composer's `gather_signatures` `None` branch is the only caller.

## Consequences

- **Compile-error breaking change** for every caller that wrote a
  `SignedTransaction { ... }` struct literal or read its fields
  directly across a crate boundary. Pre-1.0, no shim. The four examples
  and two step-defs that touched the literal form are migrated in this
  PR.
- **`transaction_id` cannot be wrong by construction.** The "store an
  empty string here, we'll fill it in later" footgun is gone.
- **Pluggable signers (D7) plus closed `SignedTransaction` (D5)
  compose**: a third-party HSM signer implements
  `Signer::sign_transactions` and produces `SignedTransaction`s through
  the same signing API the built-in signers use — no special access to
  closed fields needed.
- **Out of scope.** `SignedLogic`'s pub fields stay open; closing them
  is a future sub-ADR once a `LogicSigSession` design lands.
  `TransactionWithSigner` (the composer's signer-bearing tx record)
  keeps its `pub` fields — that's a separate type whose ergonomics are
  bound up with the upcoming `MethodCall` builder (D6).
