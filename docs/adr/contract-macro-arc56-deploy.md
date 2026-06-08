---
id: contract-macro-arc56-deploy
title: 'ARC-56 contract! deploy: byteCode programs and richer create'
abstract: Extend the contract! macro's deploy to build programs from precompiled byteCode (not just compiled TEAL source) and to honor an ABI create method's typed constructor arguments, the declared create OnComplete action, foreign references, and automatically-sized extra program pages.
status: accepted
date: 2026-05-27
deciders: []
tags: [api, abi, macros, codegen, arc56, deploy, ergonomics]
---

# ARC-56 contract! deploy: byteCode programs and richer create

## Status

Accepted and implemented on `feat/contract-deploy-enhancements`.

Extends [`contract-macro-arc56-app-spec`](contract-macro-arc56-app-spec.md),
whose Phase 5 introduced a `deploy` that compiled the contract's TEAL `source`
through algod and submitted a single app-create — bare, or carrying a no-arg
ABI create method's 4-byte selector. This ADR fills the two `deploy` gaps that
ADR tracked into #345:

- the **`byteCode` path** — a spec that ships only precompiled programs (no
  `source`) now gets a `deploy`;
- a **richer create** — `deploy` now honors an ABI create method's typed
  constructor arguments, the declared create OnComplete, foreign references, and
  the extra program pages a large contract needs.

## Context

The Phase-5 `deploy` was deliberately minimal:

- It required TEAL `source`. AlgoKit/puya specs frequently ship `byteCode`
  (base64 compiled programs) and no `source`, so those contracts got no
  `deploy` at all — the macro silently dropped the constructor.
- It issued only a *bare* NoOp create, or — for a create method with **no**
  arguments — passed that method's selector as the lone app argument. A create
  method that takes constructor arguments (the common shape, e.g.
  `createApplication(address,uint64)`), declares a non-NoOp create OnComplete,
  or references foreign accounts/assets/apps could not be expressed. Large
  contracts that need extra program pages could not be created either.

Two facts in the existing code make the richer create cheap to build on rather
than to rebuild:

- `algonaut_core::CompiledTeal` is a transparent `CompiledTeal(pub Vec<u8>)`
  over the raw program bytes, and `Algod::teal_compile` returns exactly that.
  So "use precompiled `byteCode`" is just `CompiledTeal(base64_decode(bytecode))`
  — the same value `teal_compile` would have produced, without the round-trip.
- `MethodCall` already models **application creation** (`app_id == AppId(0)`):
  its builder carries the approval/clear programs, the state schema, extra
  pages, the OnComplete, and box references, and its encoder turns ABI value,
  reference, and transaction arguments into the create transaction's app
  arguments and foreign arrays. The argument-spec machinery that the per-method
  builders use (`methods::method_arg_specs`) already maps each ABI argument to a
  typed Rust parameter and its encode expression.

So the create-method arguments, references, OnComplete, and pages are all
reachable by routing the create through a `MethodCall` instead of the bare
`CreateApplication` builder, reusing the exact argument codegen the rest of the
client already uses.

## Decision

### D1 — Program acquisition: prefer `source`, fall back to `byteCode`

`generate_deploy` resolves the approval/clear programs in this order:

1. **`source`** (TEAL text) — base64-decoded at macro-expansion time, carried as
   strings, template-substituted, and compiled through algod at deploy time
   (unchanged from Phase 5).
2. **`byteCode`** — base64-decoded at macro-expansion time and emitted as a
   `CompiledTeal(vec![..])` literal, used directly at deploy time with **no**
   `teal_compile` call.
3. Neither present (or malformed base64) → no `deploy` (graceful per the parent
   ADR's D7).

`source` wins when both are present: it is what the prior path compiled, and it
is the only form that can host template variables.

### D2 — `byteCode` is incompatible with template variables

TEAL template variables substitute `TMPL_<name>` tokens into program *text*; a
precompiled program no longer carries them. A `byteCode`-only spec that also
declares `templateVariables` therefore gets **no** `deploy`, rather than one
that would silently ignore the templates. (A `source` spec keeps the existing
per-template `u64` `deploy` parameters.)

### D3 — Create through the ABI method when one is declared

`generate_deploy` picks the first method whose declared `actions.create`
contains a recognized OnComplete (preferring `NoOp`) and whose arguments the
macro can model. When found, `deploy` routes the create through a
`MethodCall::builder(AppId(0), ..)`:

- the method's **constructor arguments** become **typed `deploy` parameters**,
  appended after the fixed `algod`/`sender`/`signer`/`params` and the
  template-variable parameters — reusing `method_arg_specs`, so a create
  argument is typed and encoded identically to the same argument on a regular
  method call (scalars, named structs, arrays, byte arrays, references,
  transactions, and literal defaults all behave the same);
- the chosen **create OnComplete** sets the call's on-complete (so a create
  declared `OptIn`/`UpdateApplication`/etc. is expressible, not just `NoOp`);
- **foreign references** (reference-typed create arguments) populate the create
  transaction's foreign arrays via the method-call encoder.

With no usable create method, `deploy` falls back to a **bare** create through
`CreateApplication` (no app arguments), as before.

### D4 — Extra program pages are sized automatically

The create allocates extra program pages from the *compiled* program length
rather than exposing a parameter: one 2048-byte page is free, and each further
slab needs an extra page, so `extra_pages = (approval_len + clear_len - 1) /
2048` (0 when the programs fit one page). This keeps the `deploy` signature
stable while letting large contracts deploy. It applies to both the
method-create and bare-create paths and to both program sources.

### D5 — The `deploy` parameter order is fixed and additive

```text
deploy(algod, sender, signer, params [, <template vars> ] [, <create args> ])
```

The fixed prefix never changes; template-variable parameters (alphabetical by
name, `source` path only) come next; create-method constructor parameters come
last, in method-argument order. A spec with neither extra group keeps the
four-argument form, so existing call sites are unaffected.

### Scope boundary — deferred

Kept out of this change deliberately, to be added behind the same stable
`deploy` entry point later:

- **Create-time box references and arbitrary extra foreign references** beyond
  the create method's own reference arguments. The method-call path populates
  foreign arrays only from reference-typed *arguments*; surfacing additional
  boxes/refs would need a new `deploy` parameter (a signature break) or a deploy
  *builder*. The create method's reference arguments — the common case — are
  already honored.
- **Non-NoOp create on the bare path.** `CreateApplication::build` hardcodes the
  NoOp OnComplete, so a *bare* (no-ABI-method) create can only be NoOp. A
  non-NoOp create must be declared on an ABI create method, which routes through
  the `MethodCall` path where the OnComplete is honored.
- **A deploy *builder*.** A fluent `Contract::deploy_builder()` would let callers
  override fees, notes, leases, boxes, and refs without growing the positional
  signature. Worth doing once the positional form's limits bite; not needed for
  the cases above.

### Alternatives considered

- **Always compile `byteCode` back through algod.** Rejected: pointless work and
  a needless node round-trip; `CompiledTeal` is just the bytes, so the macro can
  hand them straight to the create transaction.
- **Substitute template variables into `byteCode`.** Rejected: a compiled
  program has no `TMPL_<name>` tokens to substitute; the substitution model only
  makes sense on TEAL text (D2).
- **Build the rich create on `CreateApplication` and hand-encode the selector +
  args + foreign arrays.** Rejected: it would duplicate the ABI encoder and the
  foreign-array population that `MethodCall` already does correctly, and would
  not handle reference/transaction arguments. Routing through `MethodCall` with
  `app_id == 0` reuses the proven path.
- **Add `boxes` / `extra_pages` / `refs` as `deploy` parameters now.** Rejected
  for this change: it breaks the stable positional signature for cases that are
  uncommon at create time, and auto-sizing pages (D4) covers the one that isn't.
  A deploy builder is the better long-term home (deferred above).

## Consequences

- **`byteCode`-only specs deploy.** AlgoKit/puya specs that ship compiled
  programs and no `source` now generate a working `deploy`, with no node compile
  round-trip.
- **Constructor arguments are typed and checked.** A create method's arguments
  appear as typed `deploy` parameters, encoded by the same machinery as regular
  method arguments, so the create call is as type-safe as any other.
- **The `deploy` signature stays additive.** Existing four-argument and
  template-variable call sites are unchanged; new parameters only appear when the
  spec declares create arguments. Auto-sized pages keep large contracts
  deployable without a new parameter.
- **One create path, two program sources.** `source` and `byteCode` differ only
  in how the `CompiledTeal` programs are obtained; the create logic (bare vs.
  method, OnComplete, schema, pages) is shared, so the two sources cannot drift.
- **Remaining deploy ergonomics are funnelled to a future builder.** Create-time
  boxes, arbitrary extra refs, and per-transaction overrides are explicitly
  deferred to a deploy builder rather than bolted onto the positional function.
