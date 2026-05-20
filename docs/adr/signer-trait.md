---
id: signer-trait
title: Signer is a trait
abstract: Replace the closed `TransactionSigner` enum (with its inhabitable `Empty` do-nothing variant) with an open `Signer` trait. `Account`, `ContractAccount`, and a new `MultisigSigner` struct implement it; third parties can implement it for an HSM or remote KMS. `TransactionWithSigner.signer` becomes `Option<Arc<dyn Signer>>`, so "no signer yet" is `None`, not an enum variant that can be passed to the submit path by mistake. Fourth sub-ADR addressing decision item D7 of the ideal-type-safe-ergonomic-api index.
status: proposed
date: 2026-05-20
deciders: []
tags: [api, ergonomics, type-safety, signing]
---

# Signer is a trait

## Status

Proposed. Implements decision item **D7** of
[`ideal-type-safe-ergonomic-api`](ideal-type-safe-ergonomic-api.md).

## Context

`TransactionSigner` is a closed `enum` with four variants —
`BasicAccount(Account)`, `ContractAccount(ContractAccount)`,
`MultisigAccount { address, accounts }`, and `Empty`. The first three are
the real signers; `Empty` is an inhabited do-nothing variant that
produces a `SignedTransaction` with an all-zero 64-byte placeholder
signature so the simulate flow can ask "what would happen if I signed?"
without actually signing.

Two problems:

1. **Third parties cannot plug in.** A hardware wallet (Ledger), a
   remote KMS, or a custom-policy signer would need a new variant
   merged into the enum, which means a new release of `algonaut`. The
   correct shape is an open trait.
2. **`Empty` is a footgun in every `match`.** Every `match` on a
   `TransactionSigner` has to either handle `Empty` (and decide what to
   do with a "signer" that doesn't sign) or risk a runtime panic on the
   `unreachable!()` arm. The simulate-only behavior should live where
   it's used — the composer's group-assembly step — not as an
   inhabitable signer variant that any code path can be handed.

## Decision

A new `Signer` trait, in a new module `algonaut_transaction::signer`:

```rust
pub trait Signer: std::fmt::Debug + Send + Sync {
    fn sign_transactions(
        &self,
        txs: &[Transaction],
    ) -> Result<Vec<SignedTransaction>, TransactionError>;
}
```

The trait is `Send + Sync` so signers can be shared across async tasks;
`Debug`-bound because the composer derives `Debug`. Implementations
shipped with the crate:

- `impl Signer for Account` — the basic-keypair case.
- `impl Signer for ContractAccount` — passes empty program-args, matching
  the old enum variant.
- `impl Signer for MultisigSigner` — new struct holding `pub address:
  MultisigAddress` and `pub accounts: Vec<Account>`. Replaces
  `TransactionSigner::MultisigAccount { address, accounts }`.

`TransactionWithSigner.signer` changes from the enum to
`Option<Arc<dyn Signer>>`. `Arc` is cheap to clone (the composer derives
`Clone`), and `Option` encodes the "unsigned, for simulate-only" state
that `Empty` formerly carried as an inhabited variant:

```rust
pub struct TransactionWithSigner {
    pub tx: Transaction,
    pub signer: Option<Arc<dyn Signer>>,
}

impl TransactionWithSigner {
    pub fn new(tx: Transaction, signer: Arc<dyn Signer>) -> Self { ... }
    pub fn unsigned(tx: Transaction) -> Self { ... }
}
```

`AddMethodCallParams.signer` is required (method calls always have a
signer; the simulate path uses a different code path), so it stays
non-optional: `signer: Arc<dyn Signer>`.

The composer's `gather_signatures` switches to a per-slot loop: for each
`TransactionWithSigner`, if `signer.is_some()` it calls
`Signer::sign_transactions(&[tx])`; if `None` it synthesizes a
`SignedTransaction` with the all-zero placeholder signature inline. The
behavior the old `Empty` variant produced is preserved exactly; it just
lives at the one site that uses it instead of being a globally available
type.

### A latent bug retired along with the variant

The previous `gather_signatures` looped over each transaction's signer
and *overwrote* `signed_txs` on every iteration instead of appending —
so groups with multiple distinct signers lost all but the last batch.
The per-slot rewrite is correct, simpler, and removes the bug. No test
covered the multi-distinct-signer path, which is presumably why it
survived; the simulate-flow tests all happen to use a single signer per
group.

### Caller migration

```rust
// Was:
TransactionSigner::BasicAccount(account)
// Now:
Arc::new(account) as Arc<dyn Signer>

// Was:
TransactionSigner::MultisigAccount { address, accounts }
// Now:
Arc::new(MultisigSigner { address, accounts })

// Was:
TransactionWithSigner { tx, signer: TransactionSigner::Empty }
// Now:
TransactionWithSigner::unsigned(tx)
```

## Consequences

- **Compile-error breaking change** for every external caller that
  constructed a `TransactionSigner::Variant(...)`. Pre-1.0; mechanical
  migration.
- **Pluggable signers.** Third parties (HSM bindings, remote KMS
  bridges, multi-party-computation signers) implement the trait
  themselves. No `algonaut` release needed.
- **`Empty` is gone.** The `unreachable!()` arm in every `match
  TransactionSigner` is gone with it; the simulate-only behavior is
  encoded at the composer where it's actually used.
- **Out of scope.** `SignedTransaction::transaction_id` is still
  publicly settable, so an attacker (or a careless caller) can still
  produce a `SignedTransaction` with a stub ID. Closing that ingress is
  **D5** — `SignedTransaction` constructable only via signing — a
  separate sub-ADR that this one unblocks.
