---
id: contract-macro-typed-decoding
title: Typed decoding and created-app-id API for the contract! macro
abstract: Generate a reverse-of-encode abi_decode per named struct and a symmetric AbiDecode<Marker> runtime trait; expose typed reads via simulate_decoded returning the struct/scalar directly; make a literal default an Option<T> override; and add ExecuteOutcome::created_app_ids (per-transaction) alongside the awaited-only created_app_id.
status: accepted
date: 2026-05-27
deciders: []
tags: [api, abi, macros, codegen, arc56, ergonomics]
---

# Typed decoding and created-app-id API for the contract! macro

## Status

Accepted. Implements three of the feature-coverage items left open in #345 after
[`contract-macro-arc56-app-spec`](contract-macro-arc56-app-spec.md): typed
return/value decoding, overriding a literal default, and capturing a created app
id beyond the awaited transaction.

## Context

The ARC-56 `contract!` macro
([`contract-macro-arc56-app-spec`](contract-macro-arc56-app-spec.md)) generates
named-struct **encode** (Phase 2: `Struct::abi_encode(self) -> AbiValue`) but no
decode. Three concrete gaps remained, tracked in #345:

1. **Typed return / value decoding.** `simulate` results, ARC-28 event payloads,
   and global-state reads surface a raw `AbiValue`. The `struct` overlay on a
   return or event arg is dropped at read time, so a caller who passed a typed
   `Pair` in must match an `AbiValue::Array([...])` back out — the inverse of the
   Phase-2 mapping is missing.
2. **Overriding a literal default.** A `literal` `defaultValue` argument is
   dropped from the generated signature entirely (`param: None`), so a caller who
   wants a value *other* than the spec default has no way to supply it through
   the typed client.
3. **`created_app_id` beyond the awaited transaction.**
   `ExecuteOutcome::created_app_id` reflects only the awaited transaction (the
   first method call, else the first txn). A create at another slot of a mixed
   group — e.g. a bare-create grouped after a setup payment — is invisible.

The forces: stay symmetric with the existing encode path; keep the decode error
type shared across multiple `contract!` invocations in one module (generated
struct names are already crate-global, so a per-invocation error type would
collide); and keep additions backward-compatible (the existing raw `simulate`,
`global_<key>`, `decode_events`, and `created_app_id` surfaces stay as they are).

## Decision

### D1 — A symmetric `AbiDecode<Marker>` trait, dual to `AbiArg<Marker>`

The encode side pins a Rust type to an ABI type with `AbiArg<Marker>::encode`.
We add the dual in the same `algonaut_abi::macro_support` module:
`AbiDecode<Marker>::decode(AbiValue) -> Result<Self, AbiDecodeError>`, with one
impl per scalar marker (`Uint<N>` for the native widths and `BigUint`, `Byte`,
`Bool`, `Address`, `AbiString`, `Bytes`). Integer decoding is range-checked by
the target type's `TryFrom<&BigUint>`, so a `uint64` value that does not fit a
chosen `u32` is an error, not a silent truncation. The macro's
`type_map::arg_decode_expr` mirrors `arg_encode_expr` exactly, routing scalars
through the marker impls and arrays through element-wise decode + `collect`
(static arrays additionally check arity and build `[T; N]`).

The error type, `AbiDecodeError(String)`, lives in the runtime crate rather than
being emitted per `contract!` invocation, so two clients in one module share one
error type (generated struct names are already assumed unique crate-wide).

### D2 — Decoded accessors return the typed value directly, not an enum

A generated struct gains `Struct::abi_decode(AbiValue) -> Result<Struct,
AbiDecodeError>`, the exact inverse of `abi_encode`: it walks the tuple
positionally, decodes each field (recursing into named/nested struct fields), and
errors on a shape mismatch or wrong arity.

For **returns**, each method builder additionally gains an async
`simulate_decoded(algod, params) -> Result<T, Error>` sibling to the raw
`simulate`. `T` is the return's Rust type: the generated struct when the return
carries a `struct` overlay, otherwise the scalar/array Rust type. We chose
**returning the typed value directly** over a wrapper enum (e.g.
`Decoded::Struct(..) | Decoded::Scalar(..)`): the return type is statically known
per method at macro-expansion time, so an enum would only force callers to
re-match something the type system already proves. `simulate_decoded` is *added*
beside `simulate` rather than replacing it, so callers who want the raw
`SimulateOutcome` (budget, failure messages, multi-result groups) keep it.

Naming: the `_decoded` suffix marks the typed variant; `abi_decode` mirrors the
existing `abi_encode`. Events and raw state reads keep surfacing `AbiValue` (the
struct `abi_decode` is callable on whatever `AbiValue` they yield), so no
existing accessor's return type changes.

### D3 — A literal default becomes an `Option<T>` override

A `literal`-default value argument is generated as an `Option<T>` parameter
(where `T` is the argument's Rust type) instead of being dropped:

- `None` → the spec's literal default is encoded (today's behaviour, now
  explicit at the call site);
- `Some(v)` → `v` is encoded the same way a plain value argument is, overriding
  the default.

This is preferred over a second `_with_default` method or a builder setter: the
argument keeps its position in the signature, the override path reuses the exact
value-encode of a normal argument, and `None` documents intent at the call site.
A compound/struct-typed or AVM-typed default that is not an ABI value falls back
to the old "supplied automatically, omitted from the signature" behaviour.

### D4 — `ExecuteOutcome::created_app_ids`, per transaction

`created_app_id: Option<AppId>` stays — it remains the awaited transaction's
created id, the common lone-create / `deploy` case, and is unchanged for
existing callers. We **add** `created_app_ids: Vec<(usize, AppId)>`: every
application the group created, paired with the group index of the creating
transaction, in group order. A convenience `created_app_id_at(index)` looks one
up. The execute loop already fetches each method call's pending transaction; it
now also fetches non-method slots' responses to collect their `application_index`
(a fetch failure is tolerated — that slot simply contributes no created id),
making creates at any slot of a mixed group visible.

## Consequences

- **The encode/decode mapping is symmetric by construction.** `arg_decode_expr`
  mirrors `arg_encode_expr` and the `AbiDecode` impls mirror `AbiArg`, so the two
  directions cannot drift; a round-trip (`abi_encode` → `abi_decode`) returns an
  equal value, asserted in tests.
- **Typed reads need no manual `AbiValue` matching.** `simulate_decoded` returns
  `Pair`/`u64`/… directly; the raw `simulate`/`global_<key>`/`decode_events`
  surfaces are untouched, so this is purely additive.
- **A literal default is now overridable** — the typed client can express both
  "use the spec default" (`None`) and "use my value" (`Some(v)`). This is a
  *signature change* for such arguments (from no parameter to one `Option<T>`
  parameter); existing call sites and the example were updated to pass `None`.
- **Mixed-group creates are observable.** `created_app_ids` captures a create at
  any index; `created_app_id` keeps its meaning, so no caller breaks.
- **One shared decode error type.** `AbiDecodeError` is defined once in
  `algonaut_abi::macro_support`, so multiple `contract!` clients in one module do
  not collide on it.
- **Deferred.** Decoding into *typed* state/event accessors (returning the struct
  directly from `global_<key>` / event variants) is left out to avoid changing
  those existing return types; the struct `abi_decode` is the building block a
  caller uses in the meantime. Sourced (non-literal) defaults, local/box/map
  state, and richer deploy remain tracked in #345.
