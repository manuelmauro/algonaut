---
id: contract-macro-ufixed-args
title: Ufixed arguments in the contract! macro
abstract: 'Map ufixedNxM ABI arguments in the contract! macro to a generic raw-unscaled-integer newtype Ufixed<BITS, PRECISION> wrapping BigUint, encoding to AbiValue::Int exactly as the same-width uint does — closing the ufixed type gap from #342 without adding a fixed-point dependency or a new AbiValue variant.'
status: accepted
date: 2026-05-27
deciders: []
tags: [api, abi, macros, codegen, arc4, ergonomics]
---

# Ufixed arguments in the contract! macro

## Status

Accepted and implemented on `feat/contract-tuple-ufixed-args`.

Extends [`contract-macro-arc56-app-spec`](contract-macro-arc56-app-spec.md) and
[`contract-macro-from-abi-json`](contract-macro-from-abi-json.md), both of which
named `ufixed` as an explicit unsupported boundary ("no native Rust type"); and
[`abi-method-signature-macro`](abi-method-signature-macro.md), which declared the
`UFixed<B, P>` marker type but left its `AbiArg` slot unimplemented ("no `AbiArg`
impl yet (no native Rust type); present so the macro can name the slot"). This
ADR fills that slot.

## Context

ARC-4 defines `ufixedNxM`: an unsigned fixed-point decimal with an `N`-bit
mantissa (`N` a multiple of 8 in `8..=512`) and `M` decimal places of precision
(`M` in `1..=160`). A `ufixedNxM` value `real` is encoded on the wire as the
integer `round(real * 10^M)` — **bit-for-bit identical to a `uintN`**. The ABI
distinguishes the two only by the declared type; the bytes are the same.

The `contract!` macro maps each ABI argument type to a Rust parameter type
(`type_map.rs::rust_param_type`) and an encode expression
(`type_map.rs::arg_encode_expr`). Until now `ufixed` returned `Err`, so any
method with a `ufixed` argument was omitted from the generated client and listed
in its doc comment (the partial-client behaviour from the predecessor ADRs' D7).
Issue #342 tracked closing this gap and explicitly left the Rust representation
"TBD (possibly a newtype or external crate)."

Three forces shape the choice:

1. **The `AbiValue` model has no fixed-point variant.** `AbiValue` is
   `Bool | Byte | Int(BigUint) | Address | String | Array`. A `ufixed` value
   already round-trips through `AbiValue::Int`, and the `uintN` encoder
   (`encode_int`) handles it unchanged — the runtime never needed a fixed-point
   case. Whatever the macro picks must encode to `AbiValue::Int`.

2. **The macro's checking model is marker-type based.** `abi_call!`/`contract!`
   pin each argument to an ABI type with a zero-sized marker (`Uint<N>`,
   `UFixed<N, M>`, …) and a `AbiArg<Marker>` impl that names the accepted Rust
   representation(s). `UFixed<BITS, PRECISION>` already exists as a marker; it
   simply had no value type to stand in for it.

3. **Precision is part of the type, not the value.** A bare `u64` carrying a
   `ufixed64x2` would silently lose the "two decimal places" contract: nothing
   would stop a caller passing the same `u64` to a `ufixed64x3` slot, and the
   scale would be invisible at the call site. The width/precision belong in the
   type so the compiler can keep them straight.

### Options considered

- **(A) Map to the unscaled `uintN` Rust type, document the scale.** A
  `ufixed64x2` parameter would just be a `u64`, with a doc comment saying "this
  is `round(real * 100)`." Simplest, zero new types — but it erases the scale
  from the type, lets a `ufixed64x2` and a `ufixed64x3` be passed
  interchangeably, and reads no differently from a plain `uint64` argument at the
  call site. Rejected: it throws away the one piece of information `ufixed` adds
  over `uint`.

- **(B) An external fixed-point crate** (e.g. `fixed`, `rust_decimal`). Gives
  real decimal arithmetic and parsing from `"1.50"`. But it forces a public
  dependency and a conversion-to-`BigUint` step on every encode, couples the
  crate's ABI surface to a third party's type for all widths up to 512 bits
  (which `fixed` does not cover — it tops out at 128 bits), and far overshoots
  what encoding needs (the wire format is just an integer). Rejected for the SDK
  layer; a caller who wants decimal arithmetic can compute the scaled integer
  with their crate of choice and wrap the result.

- **(C) A raw, unscaled-integer newtype `Ufixed<BITS, PRECISION>` over
  `BigUint`.** Chosen — see Decision.

## Decision

### D1 — `ufixedNxM` maps to `Ufixed<N, M>`, a generic newtype over the raw integer

Add a value newtype to `algonaut_abi::macro_support`:

```rust
pub struct Ufixed<const BITS: u16, const PRECISION: u16>(BigUint);
```

It wraps the **already-scaled, unscaled integer** `round(real * 10^M)` directly
(so a `ufixed64x2` of `1.50` is `Ufixed::<64, 2>::new(150u64)`), constructed via
`Ufixed::new(impl Into<BigUint>)` and read back with `into_raw`. `BITS` and
`PRECISION` are const generics so the type pins the full `ufixedNxM` identity and
the type-checker rejects mixing a `ufixed64x2` value into a `ufixed64x3` slot.

The newtype is **not** a fixed-point arithmetic type: it does no decimal parsing
or scaling. It is the typed envelope around the on-wire integer, mirroring how
`Address`/`AssetId`/`AppId` are typed envelopes around bytes/ids at the client
boundary. This keeps the SDK dependency-free and the encoding trivial, while a
caller who wants `"1.50"` semantics scales once at the boundary.

### D2 — It encodes to `AbiValue::Int`, sharing the `uintN` wire path

The `AbiArg<UFixed<BITS, PRECISION>>` impl for `Ufixed<BITS, PRECISION>` emits
`AbiValue::Int(self.0)`. No new `AbiValue` variant, no new encoder branch: a
`ufixed` argument flows through the exact `encode_int` path a `uintN` of the same
width uses, and `AbiType::encode` range-checks the integer against the bit width
as it does for any `BigUint`-sourced uint. This satisfies force (1) and reuses
force (2)'s machinery — the macro names the `UFixed<N, M>` marker for the slot
and the impl pins `Ufixed<N, M>` as its one accepted Rust representation.

### D3 — The macro wires both directions

`type_map.rs::rust_param_type` returns `::algonaut_abi::macro_support::Ufixed<N, M>`
for `SigType::UFixed { bit_size: N, precision: M }`; `abi_marker_type` returns the
matching `UFixed<N, M>` marker; and the scalar branch of `arg_encode_expr` routes
the value through `AbiArg::<UFixed<N, M>>::encode`. Because `rust_param_type` now
succeeds for `ufixed`, a `ufixed`-typed **named-struct field** (resolved through
the same function in `codegen/structs.rs`) is supported too, with no extra code.

## Consequences

- **The `ufixed` type gap from #342 closes.** Methods previously omitted solely
  for a `ufixed` argument are now generated, and `ufixed` named-struct fields are
  supported. The predecessor ADRs' "ufixed → unsupported" boundary is retired;
  their `UFixed<B, P>` marker gains the `AbiArg` impl they reserved it for.

- **No new dependency, no new `AbiValue` variant.** `ufixed` shares the `uintN`
  representation and encoder end to end. The runtime decode path is unchanged
  (it already produced `AbiValue::Int` for `ufixed` returns).

- **Scale lives in the type; arithmetic does not.** Callers pass the raw scaled
  integer (`Ufixed::new(150u64)` for `1.50` at `ufixed*x2`), and the const-generic
  precision prevents cross-precision mix-ups at compile time. A caller wanting
  decimal ergonomics scales once with their own decimal crate — a deliberate
  boundary, the same one the SDK draws for the rest of the value model.

- **Decoding ufixed *returns* to a typed `Ufixed` is not in scope.** This ADR
  covers the argument (encode) direction the macro generates; return values keep
  decoding to `AbiValue::Int`, as today. A typed-return overlay is future work,
  orthogonal to the argument mapping decided here.

- **A small new public surface.** `Ufixed<BITS, PRECISION>` (value) joins the
  existing `UFixed<BITS, PRECISION>` (marker) in `macro_support`. The two names
  are intentionally distinct (value vs. marker), matching the value/marker split
  the signature-macro ADR established for `Address`.
