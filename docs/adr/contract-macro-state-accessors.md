---
id: contract-macro-state-accessors
title: contract! macro state accessors and sourced defaults
abstract: 'Generate local/box/map state read accessors and resolve sourced (non-literal) defaultValue at call time for the contract! macro: async, &Algod-taking getters returning Option<AbiValue>, and async methods that read global/box/local/method-sourced defaults so the argument is dropped.'
status: accepted
date: 2026-05-27
deciders: []
tags: [api, abi, macros, codegen, arc56, state, ergonomics]
---

# contract! macro state accessors and sourced defaults

## Status

Accepted and implemented on the `feat/contract-state-accessors` branch.

Extends [`contract-macro-arc56-app-spec`](contract-macro-arc56-app-spec.md),
which generated only `global_<key>` accessors and auto-supplied only `literal`
default arguments, and left the rest as gaps tracked in #345. This ADR closes
two of them:

- **Local / box / map state accessors** — getters for the storage classes the
  ARC-56 `state` declares beyond global keys.
- **Sourced (non-literal) default values** — `defaultValue.source` of
  `global` / `box` / `local` / `method`, resolved by a runtime read.

## Context

ARC-56's `state` block declares storage in three classes (global, local, box),
each split into *fixed keys* (a named slot with a base64 `key`) and *maps* (a
dynamic key/value collection with an optional `prefix`). The predecessor ADR
generated read accessors only for `state.keys.global`: a `global_<key>(&Algod)`
that fetches the application via algod's app-info endpoint and decodes the
matching entry per the key's declared ARC-56 value type.

Three sources of data were left unread:

- **Local state** is per account, so a getter needs an account argument, and it
  comes from a *different* algod endpoint (`account_app`, which returns the
  account's `app-local-state` key/value list) than global state.
- **Boxes** are not in the app-info response at all; they are read one at a time
  through algod's box endpoint (`app_box`), which takes a goal-encoded box name
  and returns a flat byte buffer with **no TEAL type tag** (unlike a
  `TealValue`, which self-describes as uint or bytes).
- **Maps** have no fixed key — the caller supplies one at run time, which must
  be encoded to bytes per the map's `keyType` and (for prefixed maps) combined
  with the prefix before it can locate the entry.

Separately, ARC-56 `defaultValue` lets a method argument carry a value the
caller may omit. The predecessor handled only `source == "literal"` (a constant
decoded at macro-expansion time and injected into the synchronous builder). The
other four sources — `global`, `box`, `local`, `method` — each require a read
*at call time*, which the synchronous `vault.method(args).build(&params)` shape
could not perform, so those arguments stayed required parameters.

## Decision

### State accessors

Generate four families of getter from the contract's declared `state`, all
`async` (every one hits algod) and all returning
`Result<Option<AbiValue>, algonaut::Error>` — `Option` because a key/box/account
may be absent, and `AbiValue` to match the existing `global_<key>` decoder
(typed return decoding is a separate gap in #345). The shapes:

| family | generated signature | algod read |
| --- | --- | --- |
| global key | `global_<key>(&self, &Algod)` | `app(app_id).params.global_state` |
| local key | `local_<key>(&self, &Algod, account: &Address)` | `account_app(account, app_id).app_local_state` |
| box key | `box_<key>(&self, &Algod)` | `app_box(app_id, "b64:<key>")` |
| map | `<class>_<map>(&self, &Algod, [account: &Address,] key: <K>)` | the read for `<class>` |

Design points:

- **The account is an explicit parameter for local reads, not the client's
  sender.** Local state is genuinely per account, and a client is often used to
  read *another* account's state (e.g. an indexer-style query), so taking it as
  an argument is strictly more expressive than defaulting to `self.sender`.
- **Map keys are typed by the declared `keyType`.** AVM key types map to raw,
  unframed bytes (`AVMString` → `&str` → UTF-8 bytes; `AVMUint64` → `u64` →
  8 big-endian bytes; `AVMBytes` → `&[u8]`); ABI key types reuse the macro's
  existing argument type-mapping (`rust_param_type` / `arg_encode_expr`) and are
  ABI-encoded. The optional `prefix` (base64 in the spec) is decoded at
  macro-expansion time and prepended to the encoded key bytes.
- **Locating a map entry differs by class.** For global/local the full key bytes
  are base64-encoded and matched against algod's base64 key strings; for box
  they become the goal-encoded box name (`b64:<base64>`).
- **Box decoding accounts for the missing type tag.** A box value is raw bytes,
  so AVM `uint64` is read from up to 8 big-endian bytes directly (not from a
  `TealValue.uint`), AVM bytes/string pass through, and ABI/struct value types
  are ABI-decoded. A missing box surfaces as a 404, which the getter maps to
  `Ok(None)` so callers can probe for existence without special-casing the error.
- **Undecodable locations are skipped, not errors.** As with the global getters,
  a key/map whose key or value type the macro can't model (an inline-struct
  literal value, a struct-typed map *key*, `ufixed`, …) simply produces no
  accessor — keeping the partial-client philosophy from the predecessor ADR.

### Sourced default values

When a method has one or more *sourced* (non-literal) default arguments, the
generated method becomes `async`, gains an `&Algod` first parameter, and returns
`Result<<Builder>, Error>` instead of the bare builder; it resolves each sourced
default by reading at call time before assembling the invocation. The defaulted
argument is dropped from the parameter list exactly as a literal default already
is. Methods with only literal defaults (or none) keep their synchronous,
infallible signature. The divergence is flagged in the generated method's doc.

Per source:

- **`global` / `box`** read the declared storage, keyed by the default's base64
  `data` (the storage key / box name).
- **`local`** reads the client's configured `self.sender`. Unlike the local
  *accessor* (which takes an explicit account), a `local` default has nowhere to
  carry an account and conventionally means "this caller's value", so the sender
  is the sensible implicit input. This is the one place a sourced default needs
  an input beyond `&Algod`, and it is documented on the generated method.
- **`method`** calls the named read-only method through the existing `simulate`
  path with no arguments and takes its decoded ABI return value.

Mixing sync and async method signatures within one generated client (driven by
each method's spec) is accepted as the pragmatic choice: a sourced default
*requires* an await, and making every method async to keep them uniform would
penalise the common no-default case. The `&Algod` parameter and `async`/`Result`
shape are discoverable from the signature and the doc.

A new `algonaut_abi::macro_support::{b64_encode, b64_decode}` pair backs the
runtime key↔base64 conversions the map accessors need, so the generated code
names only `algonaut_abi` (already a guaranteed dependency of any `contract!`
consumer) rather than a bare `::base64`. `classify_arg` was promoted to `pub` in
`algonaut_abi_sig` so the state codegen can reuse the same argument
classification the method codegen uses.

## Consequences

- A generated client now exposes the contract's full declared read surface from
  one app-spec file: global/local/box keys and maps, decoded per their ARC-56
  types, plus call-time resolution of every `defaultValue` source.
- Sourced-default methods are `async + Result`, a visible divergence from the
  sync builder methods. This is the cost of doing reads at call time without a
  separate resolution step; it is localised to methods that actually declare a
  sourced default and is documented on each.
- Local reads take an explicit account; `local` *defaults* use the sender. The
  asymmetry is deliberate (a default has no place to carry an account) and
  documented, but is a small API wrinkle to learn.
- Accessor and default values are still raw `AbiValue` (and the box/AVM decoding
  is best-effort for the untagged byte buffer). Typed decoding into the
  generated named structs remains the separate gap in #345.
- A map *key* that is a struct/tuple, or a value that is an inline-struct
  literal, yields no accessor. Real specs (e.g. AlgoKit's `ARC56Test`) carry
  exactly these, so the partial-client behaviour is exercised rather than
  hypothetical.
- The Vault fixture gained a local `seen` key (written on opt-in) and a `boxes`
  box map, with the local schema bumped to 1 uint and the embedded TEAL base64
  regenerated, so the new accessors are exercised end-to-end against a node.

Verification: offline integration tests assert the generated accessor and
sourced-default method *shapes* via the type-checker (the reads need a node);
the `e2e` suite reads local `seen` after opt-in and a box through the box-map
accessor on chain. The full check/clippy/fmt/integration suite is green; e2e is
compile-checked locally and runs in CI against a sandbox node.
