---
id: abi-method-signature-macro
title: Compile-time checked ABI method calls, modeled on format!
abstract: Model ABI method calls on the format!/println! family. abi_call!("add(uint64,uint64)uint64", 2u64, 3u64) treats the ARC-4 signature literal as a format string whose argument types are the specifiers — validating the signature against the canonical grammar, checking argument arity, and checking each argument's type via a per-type AbiArg trait bound, all at compile time with format!-quality spans. abi_method!("…") is the signature-only base. from_signature stays for dynamically sourced signatures.
status: superseded
superseded_by: "remove-abi-call-macros"
date: 2026-05-22
deciders: []
tags: [api, abi, macros, type-safety, ergonomics]
---

# Compile-time checked ABI method calls, modeled on format!

## Status

Superseded by [remove-abi-call-macros](remove-abi-call-macros.md): the
`abi_call!` / `abi_method!` macros were removed once the `contract!` macro
subsumed their use case. Originally accepted and implemented: the grammar lives in the new `algonaut_abi_sig`
crate; `abi_call!`/`abi_method!` in the new `algonaut_abi_macros` proc-macro
crate (re-exported as `algonaut_abi::abi_call!`); the `AbiArg<T>` trait,
marker types, and `MethodInvocation` in `algonaut_abi::macro_support`;
`from_signature` and `AbiType::from_str` reimplemented on `algonaut_abi_sig`.
The `MethodCall` builder's `.args(...)` is replaced by
`.invoke(...)` (D4), fed either by `abi_call!` or, for runtime-sourced
signatures, by `Invocation::new`. The first cut checks value arguments with
a canonical Rust representation (D5); transaction/reference/`ufixed`
arguments are routed through the dynamic `Invocation::new` path, a
documented gap rather than a silent one.

## Context

Every ARC-4 application call in algonaut begins by turning a method
signature into an `AbiMethod`, then feeding arguments to it. The canonical
shape, from `examples/app_call.rs`:

```rust
let method = AbiMethod::from_signature("add(uint64,uint64)uint64")?;
let call = MethodCall::builder(AppId(5), method, alice.address(), signer)
    .args([2u64, 3u64])
    .build(&params);
```

`from_signature(method_str: &str) -> Result<AbiMethod, AbiError>`
(`algonaut_abi/src/abi_interactions.rs`) is a hand-rolled string parser that
balances parens to split the argument list and parses each type through
`AbiType`'s `FromStr` impl. `args` takes `impl IntoIterator<Item = impl
Into<AbiArgValue>>` (see
[idiomatic-rust-in-algonaut-atomic](idiomatic-rust-in-algonaut-atomic.md)).

Both halves of that snippet — the signature *and* the arguments — are, at
almost every call site we own, written by hand and fixed at compile time.
Neither is checked then.

### 1. Signature typos are a runtime failure

`"add(unt64,uint64)uint64"` — `unt64` for `uint64` — compiles cleanly and
fails only when the line executes: `?` propagates an `AbiError::TypeParse`
in the examples, `.unwrap()` panics in the step-defs
(`tests/cucumber/step_defs/integration/abi.rs`). A misspelled type, an unbalanced
paren, an out-of-range bit size (`uint65`), or a missing return type all sit
latent until the path runs.

### 2. `?` / `.unwrap()` ceremony for a constant the author controls

`from_signature` returns `Result`, so every call site handles an error for
an input that cannot vary. The branch is unreachable for a well-formed
literal — pure noise the type system is forced to thread through.

### 3. The arguments are never checked against the signature

Nothing connects the two `uint64` slots in the parsed signature to the two
`u64` values passed to `args`. Each element is erased through
`Into<AbiArgValue>`, so **arity** (passing one argument, or three) and
**element type** (a `String` where `uint64` is expected) are both unchecked;
the mismatch surfaces only at encode/execute time. `.args([2u64, 3u64])` is
even an *array* — homogeneous — which only works because the elements are
type-erased; heterogeneous arguments have no honest array form today.

### 4. This is the problem `format!` already solves

A format string is a literal spec, parsed and checked at compile time:
`format!("{} + {}", a)` is a compile error (arity), and `format!("{}", v)`
where `v: !Display` is a compile error (per-specifier trait bound), both
with spans on the exact placeholder or argument. An ARC-4 signature is
structurally the same thing: `add(uint64,uint64)uint64` is a spec whose
argument *types* are the specifiers, and the call arguments are the values
that fill them. We are hand-rolling, at runtime, a check the `format!`
family does at compile time. [ideal-type-safe-ergonomic-api](ideal-type-safe-ergonomic-api.md)
points the same way — stop "relying on stringly-typed escape hatches" — and
explicitly files this kind of "larger bet" as deserving its own ADR.

### Constraints the mechanism has to respect

- **`macro_rules!` cannot see inside a string literal** (it is one token),
  so grammar validation needs a *procedural* macro — exactly what
  `format_args!` is.
- **A proc-macro sees tokens, not resolved types.** Like `format!`, it
  cannot assert `2u64: u64` directly; it must *emit code that pins the type*
  (a trait bound) and let the type-checker reject mismatches, spanned to the
  argument.
- **`from_signature` is not `const` and `AbiMethod` is not a `const`
  value.** Only the *validation* and the *argument typing* move to compile
  time; the `AbiMethod` is still built at run time.

## Decision

Model ABI method calls on the `format!` / `println!` family. Two macros, the
same relationship as `format_args!` and `format!`.

### D1 — `abi_call!`: the signature literal is a format string

```rust
let invocation = abi_call!("add(uint64,uint64)uint64", 2u64, 3u64);
```

`abi_call!` is a `#[proc_macro]` in a new `algonaut_abi_macros` crate. At
expansion it does what `format_args!` does, with the ARC-4 signature in the
role of the format string and the argument *types* in the role of the
specifiers:

1. **Parse + validate the signature** against the canonical grammar.
   Malformed → `compile_error!` spanned to the literal, reusing `AbiError`'s
   `reason` text:

   ```text
   error: invalid ABI type "unt64": unknown type name
     --> examples/app_call.rs:35:30
      |
   35 |     let call = abi_call!("add(unt64,uint64)uint64", 2u64, 3u64);
      |                          ^^^^^^^^^^^^^^^^^^^^^^^^^^
   ```

2. **Check arity** — the count of value-bearing specifiers vs. trailing
   arguments — with a `format!`-style message:

   ```text
   error: this ABI signature takes 2 arguments but 3 were supplied
   ```

3. **Check each argument's type** by emitting, per position `i` with parsed
   type `Tᵢ`, a type-pinned conversion through a per-type trait — the direct
   analog of `format!` selecting `Display` for `{}` and `Debug` for `{:?}`:

   ```rust
   // "this Rust type may stand in for ABI type T"; reuses the existing
   // From<…> for AbiValue conversions in abi_type.rs
   pub trait AbiArg<T> { fn encode(self) -> AbiArgValue; }

   impl AbiArg<Uint<64>>  for u64    { /* … */ }
   impl AbiArg<Uint<64>>  for u32    { /* widening allowed */ }
   impl AbiArg<AbiString> for &str   { /* … */ }
   impl AbiArg<Bytes>     for Vec<u8>{ /* … */ }
   // no `impl AbiArg<Uint<64>> for &str` → a mismatch fails to compile
   ```

   `Uint<const BITS: u16>`, `Bool`, `AbiAddress`, `Bytes`, `AbiString`, and
   the recursive `Tuple<…>`, `DynArray<T>`, `StaticArray<T, N>`,
   `UFixed<B, P>` are zero-sized marker types the macro synthesizes from the
   parsed signature. The macro emits `AbiArg::<Tᵢ>::encode(argᵢ)` with the
   token span of `argᵢ`, so a wrong type reads as `the trait bound
   &str: AbiArg<Uint<64>> is not satisfied`, pointed at that argument.

The expansion produces a checked **`MethodInvocation`** value — the
`AbiMethod` plus its already-encoded `Vec<AbiArgValue>` — with no `Result`
and no tuple/array gymnastics: the arguments are plain trailing macro
arguments, exactly as in `format!("…", a, b)`.

### D2 — `abi_method!`: the signature-only base

```rust
let method: AbiMethod = abi_method!("add(uint64,uint64)uint64");
```

The `format_args!`-style building block: validate the literal, expand to an
infallible `AbiMethod`, no arguments bound. Used when the method is needed
as a value (passed around, or arguments supplied dynamically). `abi_call!`
is `abi_method!` plus the argument check.

### D3 — One grammar, shared by a small crate

The grammar must not fork. If `algonaut_abi_macros` reused the parser by
depending on `algonaut_abi`, and `algonaut_abi` re-exported the macros, the
crates would cycle. Extract the pure signature/type grammar — minimal deps,
no I/O — into a small `algonaut_abi_sig` crate that both depend on:

```
algonaut_abi_sig      // pure grammar: parse + validate
   ├── algonaut_abi          (from_signature delegates here; defines markers + AbiArg)
   └── algonaut_abi_macros   (abi_call!/abi_method! validate + emit here)
```

`algonaut_abi` re-exports the macros, so the path stays
`algonaut_abi::abi_call!` (and `algonaut::abi::abi_call!`). `from_signature`
is reimplemented on top of `algonaut_abi_sig`, so the macros and the runtime
parser are provably the same grammar.

### D4 — The builder takes the checked invocation; `.args()` retires

We are free to revisit prior shape (no pre-1.0 backward-compat
constraint), so the [method-call-builder](method-call-builder.md)'s separate
`.args(...)` setter goes away: arguments only make sense relative to a
method, and now they arrive together, already checked, from the macro. The
fluent builder keeps the *call-context* setters:

```rust
let call = MethodCall::builder(AppId(5), alice.address(), signer)
    .invoke(abi_call!("add(uint64,uint64)uint64", 2u64, 3u64))
    .on_complete(OnComplete::NoOp)      // optional context stays fluent
    .build(&params);
```

No generic `MethodCallBuilder<Args>`, no phantom typed handle, no tuple — the
checking lives entirely in the macro, the builder stays monomorphic. (The
exact `builder(...)`/`.invoke(...)` spelling is itself revisitable; what this
ADR fixes is *where* the check happens.)

### D5 — Specifier kinds; honest scope

Like `format!`'s `{}` vs `{:?}`, ARC-4 has specifier *kinds*, and the
positional `format!` model maps them cleanly — each signature slot consumes
exactly one trailing argument, of a kind dictated by that slot:

- **Value args** (`uint64`, `string`, `byte[]`, tuples/arrays of these,
  `address`) → checked via `AbiArg<T>`, fully for types with a canonical
  Rust representation (`uint8…64` → `u8…u64`, `uint128` → `u128`, larger →
  `BigUint`, `bool`, `address`, `string`, `byte[]`).
- **Transaction args** (`pay`, `axfer`, `txn`, …) → consume a transaction,
  routed to the builder's transaction channel rather than encoded.
- **Reference args** (`account`, `application`, `asset`) → consume an
  `Address` / `AppId` / `AssetId`, routed to the foreign arrays.

Two honest boundaries: `ufixed` (no native Rust type) and any ABI type with
no `AbiArg<T>` impl fall back to an explicit `AbiArgValue`, unchecked — a
documented gap, not a silent one. Wiring the transaction/reference kinds
through the macro is the natural extension; the core lands the value-arg
check first.

### Alternatives considered

- **Typed handle + generic builder** (an earlier draft): `abi_method!`
  yields `TypedMethod<(Uint<64>, …), Ret>` and `MethodCallBuilder<Args>`
  checks a tuple via an `AbiArgs<Sig>` trait. It works, but it pushes a
  phantom type and a generic parameter into the public builder, forces
  `.args((a, b))` tuples, and splits the typed/untyped argument forms. The
  `format!` model gets the same checking with none of that surface — the
  macro holds both the spec and the values, so nothing generic needs to
  escape into the builder. Rejected in favor of D1.
- **A `const fn` validator / `macro_rules!`** — rejected per Constraints
  (no spans / cannot read the literal).
- **`build.rs` codegen from an app-spec** — useful for spec-shipping
  contracts but not for an inline literal; orthogonal, could coexist.

`from_signature` is *kept* (not for backward compatibility): a signature
that arrives at run time — from app-spec JSON or user input — cannot be
macro-checked, and needs the runtime parser. The macros cover the
compile-time-literal case; `from_signature` covers the rest. Both share
`algonaut_abi_sig`.

## Consequences

- **A familiar mental model.** "The signature is a format string; the
  arguments fill it" is something every Rust programmer already understands
  from `format!`. The macro's diagnostics (arity message, per-argument
  trait-bound error, spanned to the offending token) match what they expect
  from that family.
- **Both halves of a hand-written call are checked by `cargo build`.** A
  misspelled type, an unbalanced paren, the wrong argument count, or a
  `String` where `uint64` is expected become compile errors.
  `examples/app_call.rs`, `examples/atomic.rs`, and the `abi.rs` step-defs
  drop their `?`/`.unwrap()` and their unchecked `args`.
- **The design shrinks versus the typed-handle draft.** No phantom
  `TypedMethod`, no generic `MethodCallBuilder<Args>`, no `AbiArgs<Sig>`
  tuple trait, no array→tuple migration. The builder stays monomorphic; all
  checking is in the macro. This is the main payoff of the `format!` framing.
- **`method-call-builder` is amended, not superseded.** Its core decision
  (replace the 18-field params struct with a fluent builder) stands; only
  the `.args(...)` setter is replaced by `.invoke(abi_call!(…))`. That ADR
  should gain a back-reference to this one.
- **Two new workspace crates.** `algonaut_abi_macros` (`syn`/`quote`/
  `proc-macro2`) and the small `algonaut_abi_sig` grammar crate — higher
  crate count and proc-macro build cost, the price of one source of truth
  across the macros and `from_signature`.
- **A new public trait surface.** `AbiArg<T>` and the marker types are
  public and extensible; each accepted Rust representation per ABI type is
  an impl to write and keep in sync with the `AbiValue` conversions.
- **The check is partial by design** (D5): `ufixed` and unmapped types fall
  back to an explicit `AbiArgValue`. Readers should not assume *every*
  argument is checked — only those with a canonical Rust representation.
- **The dynamic path is untouched.** App-spec-loaded contracts and
  runtime-sourced signatures keep using `from_signature` and supply
  arguments as `AbiArgValue`; the compile-time check is opt-in via the
  macros.
- **Optional later optimization.** Since the macro already parses the
  signature, it can bake the fully-constructed `AbiMethod` and the 4-byte
  selector (`Sha512_256(signature)[..4]`, today computed by `get_selector`)
  as constants, skipping the runtime re-parse. Deferred; not needed for the
  checking win.
- **A natural sibling.** `AbiType::from_str` is parsed the same way; an
  `abi_type!("(uint64,byte[])")` macro falls out of the same
  `algonaut_abi_sig` foundation.
</content>
