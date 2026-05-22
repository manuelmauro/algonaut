---
id: kmd-client-hand-written
title: Keep the kmd client hand-written
abstract: kmd publishes only a Swagger 2.0 spec and exposes a small, stable v1 surface, so its client and models stay hand-written rather than joining the algod/indexer OpenAPI-3 regeneration pipeline.
status: accepted
date: 2026-05-22
deciders: []
tags: []
---

# Keep the kmd client hand-written

## Status

Accepted

## Context

`algonaut_algod` and `algonaut_indexer` began as
[openapi-generator](https://openapi-generator.tech) output and are now a
maintained fork driven by a reproducible regeneration pipeline (see
[openapi-client-regeneration](openapi-client-regeneration.md)). `algonaut_kmd`
is the third client crate, but unlike the other two it has always been
**hand-written**: the client lives in `algonaut_kmd/src/kmd/v1/mod.rs`
(~553 lines, 20 operations) and its request/response types are a single
hand-written `algonaut_model/src/kmd/v1/mod.rs` (~304 lines) — not per-type
generated files, no `configuration.rs`, no `git_push.sh`. The natural
question is why kmd is not generated like the others. Three forces:

**1. kmd publishes a different spec dialect, and the customization layer is
OAS3-shaped.** algod and indexer publish OpenAPI 3 specs — `make
fetch-openapi-specs` pulls `algod.oas3.yml` and `indexer.oas3.yml`. The key
management daemon publishes only
[`daemon/kmd/api/swagger.json`](https://github.com/algorand/go-algorand/blob/master/daemon/kmd/api/swagger.json),
whose top-level field is `"swagger": "2.0"` — **Swagger 2.0, not OAS3**.
openapi-generator `v6.6.0` *does* consume Swagger 2.0 directly (a one-off
generation against the spec produced all 22 operations — see the consistency
check below), so raw generation is feasible. What does **not** transfer is the
customization layer that makes the algod/indexer output a near-lossless
maintained fork: `preprocess.py` and the mustache templates are keyed on the
OAS3 `x-algorand-format` vendor extensions, which kmd's 2.0 spec does not
carry. Stock generation gives un-customized output (raw names, no domain
types); reaching algod/indexer parity would mean a kmd-specific config plus a
new override path.

**2. The "generated" clients are really a hand-maintained fork.** Per
[openapi-client-regeneration](openapi-client-regeneration.md), regenerating
algod fresh produces 210 `i32`/`i64` occurrences and 0 unsigned, while the
committed crate carries 171 unsigned and 0 signed — every integer was
hand-flipped — and 36 of 60 algod models reference algonaut domain types the
generator never emits. The regen exists as a **drift-detection tool**, not as
live code generation. So "generate the kmd client" would buy far less than it
sounds like: the output would still need heavy hand-customization to take the
domain types the current client already takes (`Address`,
`MasterDerivationKey`, `MultisigSignature` from `algonaut_core` /
`algonaut_crypto`).

**3. kmd is small and stable.** kmd's v1 surface is a fraction of algod's
(60 → 81 model types, 87 → 109 operations across one upstream cycle) and
barely changes. The scale and churn that justify a reproducible regen for
algod/indexer are simply absent for kmd.

A consistency check confirmed both the coverage gap and the ergonomic
trade-off. Generating a throwaway rust client from upstream
`daemon/kmd/api/swagger.json` (fetched 2026-05-22) with openapi-generator
`v6.6.0` produced **22 functional operations** (excluding the `GET
/swagger.json` meta-endpoint); the hand-written client implements **20**, with
no operations the spec lacks. The two not implemented are the TEAL-program /
LogicSig signing endpoints:

| Upstream operation | Status |
| --- | --- |
| `POST /v1/program/sign` (SignProgram) | not implemented |
| `POST /v1/multisig/signprogram` (SignMultisigProgram) | not implemented |

This is the one cost of hand-maintenance worth naming: a generated client
would have surfaced both endpoints automatically (tracked in issue #327).

The same generation illustrates the ergonomic cost in the other direction. The
stock output names operations straight off the upstream `operationId`s — so it
faithfully reproduces the upstream typo `ListMultisg` as `list_multisg`, and
emits `init_wallet_handle_token` / `get_version` / `list_keys_in_wallet` where
the hand-written client reads `init_wallet_handle` / `versions` / `list_keys`
and corrects the typo. The hand-written client is the more idiomatic surface.

Note that [client-feature-gates](client-feature-gates.md) loosely groups
`algonaut_kmd` with the "generated crates" when describing the feature gating;
structurally it sits alongside them, but it is in fact hand-written. This ADR
records that distinction explicitly.

## Decision

Keep the kmd client and its models **hand-written**. Do not onboard kmd into
`make generate-clients` or the `openapi/` regeneration pipeline.

New upstream kmd surface — including the two missing program-signing
operations should the SDK need them — is added by hand to
`algonaut_kmd/src/kmd/v1/mod.rs` and `algonaut_model/src/kmd/v1/mod.rs`,
taking algonaut domain types at the boundary like the existing methods.

## Consequences

- The kmd client keeps its ergonomic, domain-typed API with no generator
  scaffolding, no `swagger2openapi` conversion step, and no third
  config/template path to maintain.
- Upstream drift for kmd is **not** auto-detected the way algod/indexer drift
  is — tracking new kmd endpoints and fields is a manual responsibility.
- `POST /v1/program/sign` and `POST /v1/multisig/signprogram` remain
  unimplemented; a consumer needing kmd-side LogicSig signing must add them by
  hand (or sign programs client-side).
- Revisit this decision if kmd's surface grows materially or if upstream
  begins publishing an OAS3 spec for kmd — at that point the cost/benefit that
  favored algod/indexer generation could shift.
