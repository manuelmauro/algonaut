---
id: remove-abi-call-macros
title: Remove the abi_call! and abi_method! macros
abstract: Remove abi_call!/abi_method! (and the MethodInvocation type). The contract! macro now generates complete, fully-typed compile-time-checked clients from an ARC-4/ARC-56 spec, and Invocation::new + AbiMethod::from_signature covers runtime/one-off calls; the partial, stringly-typed abi_call! niche no longer justifies its surface. Supersedes abi-method-signature-macro.
status: accepted
date: 2026-06-08
deciders: []
supersedes: "abi-method-signature-macro"
tags: [api, abi, macros, simplification]
---

# Remove the abi_call! and abi_method! macros

## Status

Accepted. Supersedes [abi-method-signature-macro](abi-method-signature-macro.md).

## Context

[abi-method-signature-macro](abi-method-signature-macro.md) introduced
`abi_call!("add(uint64,uint64)uint64", 2u64, 3u64)` and the signature-only
`abi_method!("…")`, modeled on `format!`: the ARC-4 signature literal is a
format string whose argument *types* are the specifiers, validated and
type-checked at compile time, expanding to a `MethodInvocation`. When that ADR
landed (2026-05-22) it was the only compile-time-checked path for an ABI call.

Since then the `contract!` macro
([contract-macro-from-abi-json](contract-macro-from-abi-json.md) and the ARC-56
extensions) became the project's primary type-safe surface: from an ARC-4 ABI or
a full ARC-56 app spec it generates a complete typed client — typed-struct
arguments, arrays, tuples, `ufixed`, references, transaction arguments, a
`deploy` constructor, state readers, and ARC-28 events — using real Rust types
rather than a signature string. That reshapes where `abi_call!` sits:

- **It is redundant for spec-having callers.** Anyone calling a real contract
  has its ABI/app spec (that is how they know the signatures), and `contract!`
  is strictly better for them — real types, full coverage, no stringly-typed
  literal.
- **It is partial by its own design** (the prior ADR's D5): `ufixed`,
  transaction, and reference arguments fall back to the dynamic path unchecked,
  so it never was a complete one-off-call solution.
- **The runtime path already covers the rest.** A signature that is not a
  compile-time literal — or any argument kind `abi_call!` does not bind — already
  goes through `AbiMethod::from_signature` + `Invocation::new`, which remains for
  app-spec JSON and user-sourced signatures regardless.

That leaves `abi_call!` a narrow niche — *"I want a compile-time check, I know
the exact ARC-4 signature string, I do not have/want a spec file, and my
arguments are simple value types"* — which overlaps `contract!` on one side and
`Invocation::new` on the other. The macros are also a third, stringly-typed way
to express a call, the very pattern
[ideal-type-safe-ergonomic-api](ideal-type-safe-ergonomic-api.md) wants to move
away from and that `contract!` realises better. We have no pre-1.0
backward-compatibility constraint.

Crucially, removing them is cheap and well-contained: `abi_call!`/`abi_method!`
sit *on top of* the shared foundation (`algonaut_abi_sig`, the `AbiArg<T>`
trait, the marker types, `AbiDecode`) that `contract!` and `from_signature` need
anyway. `contract!` builds `atomic::Invocation` directly; `MethodInvocation`
(the value `abi_call!` expands to) is produced *only* by `abi_call!` and merely
converts into `Invocation`. So removal touches only the two proc-macros, the
`MethodInvocation` type and its `From` impl, and a handful of examples/tests/
docs — no shared infrastructure is lost.

## Decision

Remove `abi_call!` and `abi_method!`, and the `MethodInvocation` type they
expand to. Consolidate on two paths:

- **Compile-time, spec-driven** — `contract!` generates a fully-typed client
  from an ARC-4/ARC-56 spec.
- **Runtime / one-off** — `MethodCall::builder(..).invoke(Invocation::new(
  AbiMethod::from_signature("add(uint64,uint64)uint64")?, [2u64, 3u64]))`.

Kept unchanged: the `algonaut_abi_sig` grammar crate, the `AbiArg<T>` trait and
marker types, `AbiDecode`, the `Ufixed` newtype, and `AbiMethod::from_signature`
— all still used by `contract!` and the runtime parser. The `algonaut_abi_macros`
crate stays; it now exposes only `contract!`.

The `examples/method_call_dynamic.rs` (formerly `app_call.rs`) and
`examples/atomic.rs` examples and the `method_call` unit tests move to the
`Invocation::new` + `from_signature` form.

## Consequences

- **One fewer way to make a call.** The API surface consolidates on `contract!`
  (compile-time, complete) and `Invocation::new` (runtime), removing a partial,
  overlapping third option and the public `MethodInvocation` type.
- **No loss of capability.** Everything `abi_call!` checked, `contract!` checks
  more completely from a spec; everything it could not check already required
  the runtime path, which is unchanged.
- **Lost: the spec-less compile-time check.** A caller who wants compile-time
  validation of a hand-written signature without a spec file no longer has it;
  they use `from_signature` (runtime-validated) or add a small spec for
  `contract!`. This was the macros' only unique niche, judged too narrow to keep.
- **Smaller maintenance surface.** ~300 lines of proc-macro and the
  `MethodInvocation` type/`From` impl go; the `AbiArg<T>` impls remain (needed by
  `contract!`), so their upkeep is unchanged.
- **The prior ADR is superseded, not erased.** `abi-method-signature-macro`
  records why the macros were built; this ADR records why they were removed once
  `contract!` subsumed them.
</content>
