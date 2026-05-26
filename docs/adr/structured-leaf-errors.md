---
id: structured-leaf-errors
title: Structured leaf-crate errors
abstract: 'Eliminate the pure-Msg(String) error types in the leaf crates: give AbiError a typed parse/encode/decode/signature taxonomy, replace the AlgodError/IndexerError placeholders (which discard their serde source) with real #[from] source-chaining variants, and retire TransactionError::Msg and CoreError::General. Follow-up to structured-errors (D8), covering the leaf-crate sites that ADR explicitly reserved.'
status: accepted
date: 2026-05-21
deciders: []
tags: [api, errors, type-safety]
---

# Structured leaf-crate errors

## Status

Accepted. Follow-up to [`structured-errors`](structured-errors.md), which
implemented decision item **D8** of
[`ideal-type-safe-ergonomic-api`](ideal-type-safe-ergonomic-api.md) for the
top-level `Error` and explicitly deferred the leaf-crate error types:

> Out of scope. The audit also surfaced ~14 abi-arg-validation
> `Error::Msg(...)` sites … a follow-up sub-ADR can promote them to a typed
> `AbiArgError` family if the need arises.

This ADR is that follow-up, widened to cover every remaining pure-`Msg`
error type in the workspace, not only the abi-arg sites.

## Context

`structured-errors` fixed the *top* of the error stack — the variants
callers branch on now have names. But the leaf crates that feed it are
still stringly typed, and two of them are *only* a string:

| Type | Crate | Shape today | Construction sites |
|---|---|---|---|
| `AbiError` | `algonaut_abi` | `enum AbiError { Msg(String) }` — nothing else | ~58 (31 `abi_encode`, 18 `abi_type`, ~9 `abi_interactions`/`lib`/`biguint_ext`) |
| `AlgodError` | `algonaut` (`src/algod`) | `enum { Msg(String) }`, built via `format!("{:?}", e)` | placeholder |
| `IndexerError` | `algonaut` (`src/indexer`) | `enum { Msg(String) }`, built via `format!("{:?}", e)` | placeholder |
| `CoreError::General(String)` | `algonaut_core` | catch-all alongside real variants | a handful |
| `TransactionError::Msg(String)` | `algonaut_transaction` | catch-all alongside real variants | 1 (`api_model.rs:644`) |

Three distinct smells, not one:

1. **Whole error types that are just a string.** `AbiError` is the worst:
   the type carries no information a caller can act on, yet it sits on the
   `source()` chain under the top-level `Error::Abi(#[from] AbiError)`. The
   ABI surface has the richest, most enumerable failure modes in the
   workspace (type-string grammar, TLV decode, method-signature parsing)
   and expresses all of them as prose.

2. **Source-discarding placeholders.** `AlgodError` and `IndexerError`
   each wrap their cause with `format!("{:?}", e)`. That is the exact
   opposite of what commit `33e45ab` established for the top-level `Error`
   (preserve `std::error::Error::source()` via `#[from]`): the serde /
   msgpack / HTTP cause is flattened into a debug string and the chain is
   severed. A caller cannot ask "was this a transport timeout or a
   malformed response body?".

3. **Catch-alls beside real variants.** `CoreError::General` and
   `TransactionError::Msg` are escape hatches in otherwise-typed enums.
   `CoreError::General` is additionally laundered into `AbiError::Msg` by
   the `From<CoreError> for AbiError` impl (`abi_interactions.rs:38`), so a
   core failure reaches the top level as twice-stringified prose.

### Why now

The `AbiError` sites fall into clean families once read in bulk:

- **Type-string parsing** (`abi_type.rs`): `"ill formed uint type"`,
  `"Regex for ufixed didn't match"`, `"unpaired parentheses"`,
  `"no consecutive commas"`, array-length parse failures, bit-size bounds.
- **Decode** (`abi_encode.rs`): `"dynamic array format corrupted"`,
  `"string format corrupted"`, `"input byte not enough to decode"`,
  `"Couldn't convert slice to array/string"`, address decode.
- **Encode** (`abi_encode.rs`): value-out-of-range, length mismatches on
  the encode path.
- **Method-signature parsing** (`abi_interactions.rs`):
  `"method signature is missing an open parenthesis"`, argument-count
  mismatches.
- **Value bounds** (`biguint_ext.rs`, `lib.rs`): big-int → fixed-width
  conversions.

These are enumerable, not open-ended. They are exactly the kind of thing a
`thiserror` enum is for, and several already format `{:?}` debug values
that callers can't branch on — the same tell `structured-errors` used to
separate "promote" from "reserve".

## Decision

Eliminate the pure-`Msg` error types and the leftover catch-alls. The
target is: **no error type in the workspace is `Msg(String)`-only, and no
error discards its source.** A narrow free-form variant may remain *only*
where the body is a genuinely unstructured diagnostic, mirroring the
`Msg`/`Internal` reservation `structured-errors` already blessed at the top
level.

### 1. `AbiError` gains a typed taxonomy

Replace `enum AbiError { Msg(String) }` with variants grouped by the
families above. Indicative shape (field names to be settled in
implementation; the *families* are the decision):

```rust
pub enum AbiError {
    /// An ABI type string could not be parsed (e.g. "uint256[]").
    #[error("invalid ABI type {input:?}: {reason}")]
    TypeParse { input: String, reason: String },

    /// Encoding a value into its ABI byte representation failed.
    #[error("ABI encode error: {reason}")]
    Encode { reason: String },

    /// Decoding ABI bytes failed (corrupted framing, short input, …).
    #[error("ABI decode error: {reason}")]
    Decode { reason: String },

    /// A method selector / signature string could not be parsed.
    #[error("invalid ABI method signature {input:?}: {reason}")]
    MethodSignature { input: String, reason: String },

    /// A value was outside the range its ABI type allows.
    #[error("value out of range for {abi_type}: {reason}")]
    ValueOutOfRange { abi_type: String, reason: String },
}
```

The `reason: String` bodies preserve today's diagnostics (including the
`{:?}` debug renderings) as one-shot text. The *variant* is what callers
and tests match on; the body stays free to reword. This is the same
contract `structured-errors` set with `ComposerStatusInvalid(String)` —
structure where it's branched on, free text for the diagnostic tail.

Whether to fold `ValueOutOfRange` into `Encode` and whether the four-or-five
split is right is an implementation detail; the binding decision is that
`AbiError` stops being a single `Msg` variant and grows a parse / encode /
decode / signature spine.

### 2. `AlgodError` and `IndexerError` preserve their source

Replace the `format!("{:?}", e)` placeholders with variants that carry the
cause via `#[from]`, so the `source()` chain survives to the top level:

```rust
pub enum AlgodError {
    #[error("response decode error")]
    Decode(#[from] serde_json::Error),
    #[error(transparent)]
    Msgpack(#[from] rmp_serde::decode::Error),
    // … transport / request variants as the call sites require
}
```

The `From<AlgodError> for Error` / `From<IndexerError> for Error` impls in
`src/error.rs` (today a single `Msg(msg) => Error::Msg(msg)` arm) map each
new variant onto a top-level variant — reusing `Error::Request` /
`Error::Internal` where they fit rather than minting parallel ones.

### 3. Retire `CoreError::General` and `TransactionError::Msg`

- `CoreError::General` → typed variants for its real call sites (length
  mismatch on a 64-byte key, vec→array conversion, the msgpack paths). The
  `From<CoreError> for AbiError` impl maps them onto the new `AbiError`
  variants instead of `Self::Msg(msg)`.
- `TransactionError::Msg` (one site, `api_model.rs:644`, a deserialization
  detail) folds into the existing `TransactionError::Deserialization(String)`.

### 4. Top-level `Error::Msg` — out of scope, stays reserved

`structured-errors` already ruled on the top-level `Error::Msg` /
`Internal` sites and reserved them for unstructured cases (BASE64 decode,
arg-type `{:?}` mismatches). This ADR does **not** reopen that. Where the
new `AbiError` variants make a top-level site newly typed (e.g. the arg-type
mismatches that currently build `Error::Msg` could instead surface as
`Error::Abi(AbiError::ValueOutOfRange { … })`), the improvement falls out
for free; no new top-level variants are minted here.

### Phasing

Independent, separately landable steps (no test asserts on these messages
today except via the top-level `Error`, so the blast radius is contained to
each crate):

1. `algonaut_core` — `CoreError` variants.
2. `algonaut_abi` — `AbiError` taxonomy (depends on 1 for the `From` impl).
3. `algonaut_transaction` — drop `TransactionError::Msg`.
4. `src/algod` + `src/indexer` — source-chaining client errors and the
   `From … for Error` arms.

## Consequences

- **Breaking change, compile-error class.** Every `AbiError::Msg(...)`,
  `AlgodError::Msg(...)`, `IndexerError::Msg(...)`, `CoreError::General(...)`
  and `TransactionError::Msg(...)` construction site must move to a named
  variant — ~70 sites across four crates. All are in-tree; downstream
  crates that constructed these (unlikely, they're mostly internal) break
  too. This is consistent with `structured-errors` accepting the same
  breakage at the top level.
- **`source()` chains become real.** A malformed algod response decode
  failure will carry the underlying `serde_json::Error` as its source
  instead of a flattened `"{:?}"` string, restoring the property
  `external-signature-ingress` / the signer ADRs assume when they talk
  about preserving causes.
- **Per the migration-consistency rule, the change lands everywhere at
  once within each crate** — test scaffolding, `dryrun_printer`, the
  integration step-defs in `tests/cucumber/step_defs` included. No "promote the
  enum but leave the call sites on a deprecated `Msg`" half-state.
- **Tests gain something to match on.** ABI parse/decode failures become
  variant-matchable, so future step-defs can assert
  `Err(AbiError::TypeParse { .. })` instead of substring-matching prose —
  the hygiene win `structured-errors` set out to spread.
- **Risk: over-fitting the taxonomy.** Reading 58 sites could tempt a
  variant per message. The mitigation is the `reason: String` tail — keep
  the *variant* count to the four-or-five families and let the body carry
  specifics, so the enum stays legible.
- **Deferred / not decided here.** The exact field shapes (`reason: String`
  vs richer structured fields like `{ expected, actual }`), and whether
  `AlgodError`/`IndexerError` should simply collapse into `Error::Request`
  rather than keep distinct enums, are settled during implementation of
  each phase. The top-level `Error::Msg`/`Internal` reservation from
  `structured-errors` is unchanged.
