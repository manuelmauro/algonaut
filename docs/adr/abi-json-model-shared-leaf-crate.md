---
id: abi-json-model-shared-leaf-crate
title: ABI JSON model in a shared leaf crate
abstract: 'Extract the ARC-4 ABI JSON data model into a dependency-light leaf crate, algonaut_abi_model, shared by the runtime algonaut_abi (which keeps its richer types and delegates serde via #[serde(from/into)]) and the contract! proc-macro in algonaut_abi_macros. Single-sources the wire format and removes the macro''s duplicate structs, at the cost of one more published crate.'
status: accepted
date: 2026-05-25
deciders: []
tags: [abi, macros, codegen, crate-layout, serde]
---

# ABI JSON model in a shared leaf crate

## Status

Accepted. Implemented on the `feat/contract-macro` branch (PR #341): new crate
`algonaut_abi_model`; `algonaut_abi`'s `AbiContract` / `AbiInterface` /
`AbiMethod` / `AbiMethodArg` / `AbiReturn` / `AbiContractNetworkInfo` keep their
fields and behaviour but carry `#[serde(from = "model::…", into = "model::…")]`
plus `From` conversions; `algonaut_abi_macros` depends on the model and dropped
its duplicate structs and its direct `serde` (derive) dependency.

Amends decision **D5** of
[`contract-macro-from-abi-json`](contract-macro-from-abi-json.md): the macro no
longer defines its own minimal `ContractJson`/`MethodJson`/… structs.

## Context

The `contract!` macro
([`contract-macro-from-abi-json`](contract-macro-from-abi-json.md)) needs to
read an ARC-4 ABI/app-spec JSON file at compile time. `algonaut_abi` already
defines a full serde model of that exact JSON — `AbiContract`, `AbiMethod`,
`AbiMethodArg`, `AbiReturn`, `AbiContractNetworkInfo` — with the matching
`#[serde(rename)]`s (`"desc"`, `"type"`, `"appID"`, …).

The macro could not reuse those types directly: `algonaut_abi` re-exports the
macros from `algonaut_abi_macros`, so the macro crate depending back on
`algonaut_abi` is a dependency cycle Cargo rejects. D5 worked around this by
having the macro define its own minimal duplicate structs. That left two
hand-maintained definitions of the same wire format, free to drift.

Two further facts shape the fix:

- `algonaut_abi`'s types are **not** pure DTOs. `AbiContractNetworkInfo.app_id`
  is a typed `AppId` (`algonaut_core`); `AbiMethodArg`/`AbiReturn` carry a
  lazily-parsed `Option<AbiType>` cache; `AbiMethod` computes selectors via
  `sha2`. Pulling any of that into a proc-macro's build graph would be wrong
  and heavy. The macro only needs the raw strings.
- The codebase already has the pattern for exactly this split:
  [`abi-method-signature-macro`](abi-method-signature-macro.md) introduced
  `algonaut_abi_sig`, a zero-I/O leaf crate holding the signature/type grammar
  shared by the runtime parser and the macros. The JSON shape is the analogous
  shared concern, with one extra dependency the grammar crate deliberately
  avoids: serde.

We considered three options (discussed in full before deciding):

1. **Move the model into `algonaut_abi_sig`.** Zero new crates, but forces
   `serde` into a crate whose charter is "pure grammar, no I/O, no heavy deps,"
   and conflates "parse a signature string" with "deserialize a contract file."
2. **Keep the duplicate, add a drift-guard test.** Lowest effort, but leaves
   two definitions of the wire format.
3. **A new shared leaf crate for the model.** Cleanest separation — each leaf
   has one reason to change and its own dependency profile — at the cost of one
   more published crate (see
   [`publish-entire-workspace`](publish-entire-workspace.md)).

## Decision

Take option 3. Add `algonaut_abi_model`: a dependency-light leaf crate (serde
derive only, no `serde_json`, no I/O, no `algonaut_core`) holding the canonical
serde structs for the ARC-4 ABI JSON — `AbiContract`, `AbiInterface`,
`AbiMethod`, `AbiMethodArg`, `AbiReturn`, `AbiContractNetworkInfo` — plus the
pure helpers `AbiMethod::get_signature()` and `genesis_to_network()`. `appID`
is modelled as a plain `u64`; the model carries no resolved types and no caches.

This crate is the **single source of truth for the JSON shape**, sitting below
both consumers so it breaks no cycle:

- **`algonaut_abi_macros`** deserializes ABI files straight into the model and
  generates code from it. Its own duplicate structs and its `serde`-derive
  dependency are removed (`serde_json` stays, to parse).
- **`algonaut_abi`** keeps its richer runtime types unchanged in fields and
  public API, but delegates the wire format to the model via
  `#[serde(from = "model::X", into = "model::X")]` and a set of total `From`
  conversions. The runtime types add what the model deliberately omits: the
  typed `AppId`, the lazily-parsed `AbiType` cache (reset to `None` on
  conversion, filled on demand exactly as before), and selector hashing.

The field-name mapping (`rename`, `skip_serializing_if`, `default`) now lives in
exactly one place. The existing exact-JSON round-trip tests in
`algonaut_abi` (`abi_json_tests.rs`) are the regression guard that the delegated
format is byte-identical; they pass unchanged.

## Consequences

- **No drift.** The macro and the runtime cannot disagree about the JSON shape:
  there is one definition, and `algonaut_abi`'s round-trip tests pin its bytes.
- **Proc-macro build stays lean.** The macro pulls in `algonaut_abi_model` +
  `serde_json`, not `algonaut_core`, `AbiType`, or `sha2`.
- **One more published crate.** `algonaut_abi_model` is depended on by both
  `algonaut_abi` and the (always-published) proc-macro crate, so it must be
  published too. This is the unavoidable cost of option 3; the broader rationale
  for accepting it lives in
  [`publish-entire-workspace`](publish-entire-workspace.md).
- **Slightly more lenient runtime decoding.** Routing through the model means a
  method object with no `returns` now decodes to `void` (the model defaults it)
  rather than erroring. This is a strict superset of prior behaviour and
  spec-reasonable; no existing test relied on the stricter form.
- **D5 of the contract-macro ADR is amended, not reversed.** The macro and its
  user-facing behaviour are unchanged; only the internal source of the parsed
  structs moved.
