---
id: openapi-client-regeneration
title: Reproducible OpenAPI client regeneration
abstract: Make algod/indexer client regeneration reproducible and diff-able, then drive it toward near-lossless so upstream drift can be ported deliberately.
status: accepted
date: 2026-05-19
deciders: []
tags: []
---

# Reproducible OpenAPI client regeneration

## Status

Accepted

## Context

`algonaut_algod` and `algonaut_indexer` started life as
[openapi-generator](https://openapi-generator.tech) output (commits #213,
#214, generator `6.6.0-SNAPSHOT`). They have since been heavily customized by
hand and are best understood as a **maintained fork** of generated code:

- Regenerating the algod client fresh produces **210** `i32`/`i64`
  occurrences in the model files and **0** unsigned; the committed crate has
  **0** signed and **171** unsigned. Every integer was hand-flipped to the
  unsigned width Algorand actually uses.
- **36 of 60** algod model files reference algonaut domain types
  (`HashDigest`, `Bytes`, `Address`, `SignedTransaction`, the simulate
  power-pack) that the generator never emits.
- Further hand-edits landed over time: the metadata-hash deserialization fix
  (#237), block serde improvements (#233, #234), the simulate power-pack
  (#261, #273).

A naive regeneration would revert all of that.

At the same time the clients have **drifted behind upstream**. Regenerated
against the current specs:

| | algod | indexer |
| --- | --- | --- |
| model types | 60 → 81 (+24 new) | 64 → 71 (+7) |
| operations | 87 → 109 (+22) | 20 → 21 (+1) |

New algod surface includes block logs / txids, ledger state deltas, account
application/asset resources, genesis, and heartbeat-related models, plus new
fields on existing models (`Account.incentive_eligible`, `last_heartbeat`,
`last_proposed`).

Regeneration was also **unreproducible**: no committed spec, no generator
config, no pinned generator version, no script. The knowledge of *how* to
regenerate was lost.

Two facts make a better setup tractable:

1. Most divergence is **mechanical** — a global `integer → u64` rule is
   correct for the overwhelming majority of Algorand's numeric fields, and the
   spec carries `x-algorand-format` vendor extensions (`Address`,
   `SignedTransaction`, `TEALProgram`, `uint64`) on 27 properties, so the
   domain-type substitutions are derivable from the spec.
2. openapi-generator supports `typeMappings`, `importMappings`, and custom
   mustache templates that can express most of this as configuration.

The `decimals` field is a good illustration: the algod spec declares it
`format: uint64`, the indexer spec gives it no format at all, the generator
emits `i32` for both, and the crate carried `u64`. Issue #140 settled on
`u32`. None of the four agree — exactly the kind of drift this setup surfaces.

## Decision

Treat the clients as a customized fork and make regeneration a **repeatable,
review-able** process, staged in three phases.

**Phase 1 — reproducible scaffolding (this change).**

- Pin the upstream specs under `openapi/specs/` and add `make
  fetch-openapi-specs` to refresh them.
- Commit per-client generator configs (`openapi/config-{algod,indexer}.yaml`)
  carrying `packageName`, `library`, and `typeMappings` (the formatted
  integers — `format: uint64`/`int64` — now regenerate as `u64`).
- Pin the generator version (`v6.6.0`) and add `make generate-clients`
  (Docker-based; no local Java needed).
- Regenerated output lands in `openapi/generated/` (git-ignored) for
  diffing — it never overwrites the crates.

**Phase 2 — drive the regen toward near-lossless.**

*Phase 2a (done).* Custom `model.mustache` and `reqwest/api.mustache`
templates under `openapi/templates/` force every integer — scalar, array
element, and operation parameter — to `u64`, since Algorand types its
integers as 64-bit unsigned. With the integer dimension removed, **86 of the
121 common model files regenerate byte-identical** to the committed crates
(41/57 algod, 45/64 indexer).

*Phase 2b (next).* The bulk of the remaining residual is the domain-type
substitution — `format: byte` / `x-algorand-format` fields that the crates
hand-type as `HashDigest`, `Bytes`, `Address`, `SignedTransaction`. Mustache
cannot branch on a vendor extension's *value*, so this needs either a custom
generator class or a post-generation patch step driven by the spec's
`x-algorand-format`. The remaining hand-written extensions then move into
`ext/` modules so the generated files carry no bespoke logic.

Goal: `make generate-clients` produces a diff small enough to review by eye.

**Phase 3 — adopt upstream changes.**

- With a near-lossless regen, port the +24 algod / +7 indexer model types and
  the +22 / +1 operations in a dedicated, reviewed change.

## Consequences

- Regeneration becomes a reproducible drift-detection tool: `make
  generate-clients` then a `git diff --no-index` against the crates.
- Phase 1 is non-destructive — the crates are untouched — so it carries no
  risk and unblocks the later phases.
- The committed specs add ~460 KB to the repo; that is the pinning mechanism
  and the price of reproducibility.
- The Phase 2a templates cover every integer; the domain-type substitutions
  (`HashDigest`, `Bytes`, `Address`, `SignedTransaction`) are the main
  remaining residual and are addressed in Phase 2b.
- The clients are still behind upstream after Phase 1 — closing that gap is
  deliberately deferred to Phase 3 so the API additions get a focused review.
