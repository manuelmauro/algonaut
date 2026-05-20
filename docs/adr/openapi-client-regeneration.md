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

Custom templates and a preprocessing step under `openapi/`, wired through
`make generate-clients`:

- `model.mustache` / `reqwest/api.mustache` — every integer (model field,
  `Vec` element, operation parameter) emits `u64`, since Algorand types its
  integers as 64-bit unsigned.
- `model.mustache` — `format: byte` fields emit `algonaut_encoding::Bytes`
  instead of `String`.
- `openapi/type-overrides.json` — a per-field table for the domain types the
  spec cannot express (`HashDigest`, `Vec<SignedTransaction>`, ...).
  `openapi/preprocess.py` reads it and injects `x-rust-type` / `x-rust-serde`
  / `x-rust-imports` vendor extensions into the spec, which the template
  consumes; mustache cannot branch on an `x-algorand-format` *value*, so the
  decision is made in the preprocessor instead. preprocess.py fails loudly
  if a table entry stops matching the spec.
- `make generate-clients` runs `rustfmt` (edition 2024, matching the
  workspace) over the output, so the review diff reflects semantic drift
  rather than formatting.

With these, **77 of the 121 common model files regenerate byte-identical**
to the committed crates (33/57 algod, 44/64 indexer) — measured after
formatting, ignoring the generated-header comment.

The remaining ~44 files carry genuinely new upstream models and fields — the
drift the tool exists to surface — plus a few bespoke per-file hand-edits (a
renamed field, an extra `skip_serializing_if`) not worth encoding in the
table. The regen now serves drift detection with a tightly review-able diff;
the last bespoke edits stay documented hand-edits.

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
- The Phase 2 templates and `type-overrides.json` cover every integer, the
  `Bytes` fields, and the domain-typed fields; only genuinely new upstream
  content and a few bespoke per-file edits remain in the regen diff.
- `type-overrides.json` is a maintenance surface: when upstream adds a
  domain-typed field the regen emits the stock type, the diff reveals it,
  and a table entry is added.
- The clients are still behind upstream after Phase 1 — closing that gap is
  deliberately deferred to Phase 3 so the API additions get a focused review.
