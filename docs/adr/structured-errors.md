---
id: structured-errors
title: Structured error variants
abstract: Promote the `Error::Msg("...")` sites that callers actually branch on into typed variants — `EmptyTransactionGroup`, `ComposerStatusInvalid`, `ComposerGroupFull`, `MissingReturnLog`, `MissingSourcemap`. Reserve `Msg` / `Internal` for the genuinely unstructured cases. Third sub-ADR addressing decision item D8 of the ideal-type-safe-ergonomic-api index.
status: proposed
date: 2026-05-20
deciders: []
tags: [api, errors, type-safety]
---

# Structured error variants

## Status

Proposed. Implements decision item **D8** of
[`ideal-type-safe-ergonomic-api`](ideal-type-safe-ergonomic-api.md).

## Context

`Error` has two catch-all variants — `Msg(String)` and `Internal(String)` —
and ~27 construction sites inside the workspace use them for failure modes
that callers genuinely care about. `build_group()` reports an empty group
as `Error::Msg("attempting to build group with zero transactions")`; the
ABI step-def asserts on it by **substring match**
(`tests/step_defs/integration/abi.rs:540`). A message reword silently
breaks the test.

The atomic-transaction-composer has the same pattern in five other places:
- `"status must be BUILDING in order to add transactions"` (twice)
- `"status is already committed"`
- `"reached max group size: 16"` (twice)
- `"App call transaction did not log a return value"` (twice exactly,
  once with a `(2)` suffix)

And `algod.teal_compile_with_sourcemap` reports a missing source-map as
`Error::Msg("algod did not return a sourcemap")`.

## Decision

`Error` gains five variants for the failure modes callers actually branch on:

```rust
#[error("transaction group is empty")]
EmptyTransactionGroup,

#[error("composer status invalid: {0}")]
ComposerStatusInvalid(String),     // operation + expected status, free-form

#[error("composer group full (max {max} transactions)")]
ComposerGroupFull { max: usize },

#[error("app call transaction did not log a return value")]
MissingReturnLog,

#[error("algod did not return a sourcemap")]
MissingSourcemap,
```

`Msg(String)` and `Internal(String)` stay for the genuinely unstructured
cases (BASE64 decode errors, arg-type mismatches with rich `{:?}` debug
formatting, the "should not happen" SDK-internal paths).

The step-def that previously substring-matched on the empty-group
message now matches the variant:

```rust
// Was:
match build_res {
    Err(Error::Msg(m)) if m == "attempting to build group with zero transactions" => {}
    _ => panic!(...),
}

// Now:
match build_res {
    Err(Error::EmptyTransactionGroup) => {}
    _ => panic!(...),
}
```

### Why `ComposerStatusInvalid(String)` over `ComposerStatusInvalid { expected, actual }`?

The three sites that hit this variant all carry slightly different
metadata — `submit` knows "after a previous submit/execute", `execute`
knows "after the composer is committed", `add_method_call` knows "must
be Building". A single `expected: ComposerStatus` enum field would
either lose that nuance or force a `Vec<ComposerStatus>` /
`Option<ComposerStatus>` shape that's harder to read. The `String` body
captures the contextual hint as a one-shot diagnostic; tests should
match on the **variant**, not on its body.

## Consequences

- **Compile-error breaking change.** Any caller that pattern-matched on
  `Error::Msg(...)` for one of the now-typed failure modes will fail to
  compile until they switch to the variant. The exact-message
  substring assertion in `tests/step_defs/integration/abi.rs` is the
  one such caller in-tree; it's migrated in this PR.
- **`Msg` and `Internal` aren't gone — they're reserved.** The
  remaining ~22 sites use `Msg` for arg-type mismatches, BASE64
  decode failures, and similar cases where the error body carries
  data callers can't reasonably branch on. Those messages are still
  free to reword without breaking anything.
- **Test-side hygiene improves.** Substring-matching on English prose
  is the failure mode this ADR sets out to remove; the migrated
  step-def is the template for any future test that wants to assert
  on an error.
- **Out of scope.** The audit also surfaced ~14 abi-arg-validation
  `Error::Msg(...)` sites (`"Invalid value type: {arg_value:?} for arg
  type: {arg_type:?}"`, "incorrect number of arguments were provided",
  etc.). Those are heavily formatted with `{:?}` debug values and no
  caller branches on them today. They stay as `Msg` for now; a
  follow-up sub-ADR can promote them to a typed `AbiArgError` family if
  the need arises.
