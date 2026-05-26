---
id: method-call-builder
title: MethodCall fluent builder
abstract: Replace the 18-field `AddMethodCallParams` and `add_method_call(&mut params)` with a `MethodCall::new(app_id, method, sender, signer).args(...).on_complete(...).boxes(...).build(&params)?` fluent builder. The twelve-positional-arg helper in the ABI step-def collapses to a one-line builder. Seventh sub-ADR addressing decision item D6 of the ideal-type-safe-ergonomic-api index.
status: accepted
date: 2026-05-20
deciders: []
tags: [api, ergonomics, abi]
---

# MethodCall fluent builder

## Status

Accepted. Implements decision item **D6** of
[`ideal-type-safe-ergonomic-api`](ideal-type-safe-ergonomic-api.md).
Depends on **D7** ([`signer-trait`](signer-trait.md)) — the builder's
`signer` argument is `Arc<dyn Signer>`.

Amended by
[`abi-method-signature-macro`](abi-method-signature-macro.md): its **D4**
replaces this builder's `.args(...)` setter with `.invoke(...)` (fed by the
`abi_call!` macro or `Invocation::new`) and drops `method` from the
positional `builder(...)` arguments. The core decision here — replacing the
18-field params struct with a fluent builder — stands.

## Context

`AddMethodCallParams` is an 18-field struct: `app_id`, `method`,
`method_args`, `fee`, `sender`, `suggested_params`, `on_complete`,
`approval_program`, `clear_program`, `global_schema`, `local_schema`,
`extra_pages`, `note`, `lease`, `rekey_to`, `signer`, `boxes`. The
composer takes it by `&mut` and consumes it.

`tests/cucumber/step_defs/integration/abi.rs` funnels every method call through
one private helper with **twelve positional arguments**, most of them
`Option`:

```rust
add_method_call(w, account, oc, None, None, None, None, None, None, None, false, None).await;
```

Six distinct sites pass six different `None`-patterns into that helper.
This is the single least ergonomic corner of the SDK, and it's the
corner that ARC-4 application development depends on most.

## Decision

```rust
let call = MethodCall::new(AppId(id), method, sender, signer)
    .args(args)
    .on_complete(OnComplete::NoOp)
    .boxes(boxes)            // optional, omitted when unused
    .build(&params)?;
composer.add_method_call(call)?;
```

`MethodCall::new(app_id, method, sender, signer)` takes the four
genuinely required inputs positionally; everything else is an optional
setter that defaults to `None` (for `Option<…>` fields) or to a sensible
default. `.build(&params)` consumes the suggested-params reference and
returns the finalised `MethodCall` value the composer accepts.

### Why `sender` is positional, not derived from the signer

The `Signer` trait (D7) does not expose a single sender address —
`MultisigSigner` has a multisig address (not necessarily the sender of
any one transaction), and third-party HSM/KMS signers may not have a
unique address at all. Taking `sender` as a required positional
argument avoids leaking signer-implementation concerns into the call
site.

### Setter defaults

| Setter             | Default                              |
|--------------------|--------------------------------------|
| `args`             | empty                                |
| `fee`              | `params.min_fee` (resolved on build) |
| `on_complete`      | `OnComplete::NoOp`                   |
| `extra_pages`      | `0`                                  |
| `approval_program` | `None`                               |
| `clear_program`    | `None`                               |
| `global_schema`    | `None`                               |
| `local_schema`     | `None`                               |
| `note`             | `None`                               |
| `lease`            | `None`                               |
| `rekey_to`         | `None`                               |
| `boxes`            | `None`                               |

### `OnComplete` alias

`pub use ApplicationCallOnComplete as OnComplete;` lives next to the
enum in `algonaut_transaction::transaction`. The full name still
compiles everywhere it appears today; new code reads `OnComplete::NoOp`
instead of `ApplicationCallOnComplete::NoOp`.

### Composer

`composer.add_method_call(call: MethodCall)` takes the finished value.
The old `&mut AddMethodCallParams` signature is gone.

### Step-defs collapse

The 12-positional-arg helper retires. Each of the six callers becomes a
one-line builder chain with no `None`s:

```rust
// Was:
add_method_call(w, account, oc, None, None, None, None, None, None, None, false, None).await;

// Now:
let call = ctx.builder().build(&ctx.params);
```

Sites that previously passed a non-None argument add the corresponding
setter — `.boxes(boxes)`, `.note(note)`, `.approval_program(p)` — and
nothing else.

## Consequences

- **Compile-error breaking change** for every external caller of
  `AddMethodCallParams` or `add_method_call(&mut params)`. Pre-1.0;
  mechanical migration.
- **The step-defs become a regression test for ergonomics.** If a
  future PR makes ABI method calls noisier, the test code grows
  visibly; the builder shape is the canonical "the API still feels
  good" oracle.
- **The composer's public surface for ABI calls is now one method
  (`add_method_call`) plus one builder type (`MethodCall`).** No
  18-field struct. No `&mut` argument that needs to be threaded through.
- **Out of scope.** A typestate-encoded `MethodCall` (e.g.
  `MethodCallBuilder<NoArgs> → <WithArgs>` to enforce that `.args()`
  is called when the method expects arguments) is a non-goal of the
  index ADR. The current builder runtime-validates arg counts inside
  `build` — same as `AddMethodCallParams` did.
