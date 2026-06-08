---
id: contract-macro-arc56-app-spec
title: Generate contract clients from ARC-56 app specs
abstract: 'Re-scope the contract! macro from "reads an ARC-4 ABI JSON" to "reads an ARC-56 Extended App Description," treating the ARC-4 contract JSON it understands today as the degenerate subset. ARC-56 is a strict superset that the official Algorand SDKs (algokit-utils, algokit-client-generator) already standardise on, and it carries exactly the metadata the macro is missing: named structs (which unblock tuple args/returns), readonly markers, default argument values, declared state, ARC-28 events, and create/call action sets. Extend the shared algonaut_abi_model leaf crate additively so ARC-4 files keep round-tripping unchanged, then layer the new code generation in phases.'
status: accepted
date: 2026-05-25
deciders: []
tags: [api, abi, macros, codegen, arc56, ergonomics]
---

# Generate contract clients from ARC-56 app specs

## Status

Accepted and largely implemented on the `feat/contract-macro-arc56` branch:

- **Phase 1 (D2):** parse the full ARC-56 schema in `algonaut_abi_model`;
  ARC-4 files round-trip unchanged. *(unit + round-trip tested)*
- **Phase 2 (D3):** generate named-struct types from `structs` and accept them
  as typed `contract!` method arguments — closing the tuple-argument gap from
  #342. *(integration tested)*
- **Phase 3 (D5, literal):** an argument with a `literal` default value is
  decoded and supplied automatically, omitted from the signature. *(tested)*
- **Phase 3 (D4, read-only):** every method builder gains a `simulate` read
  path. *(compile-verified against the real simulate API)*
- **Phase 4 (events):** an ARC-28 event decoder (`decode_events`) over a
  transaction's logs. *(unit tested with synthetic logs)*
- **Phase 4 (state):** `global_<key>` read accessors that fetch and decode
  declared global state. *(compile-verified against the algod app-info API)*
- **Phase 5 (lifecycle):** declared `call` actions become builder setters
  (`opt_in`/`close_out`/`update`/`delete`) that set the on-complete. *(tested)*
- **Phase 5 (deploy):** a `deploy` constructor compiles the contract's TEAL
  `source` through algod, submits an app-create with the declared schema, and
  returns a client bound to the new app id. This required exposing the created
  app id on the atomic group: `ExecuteOutcome::created_app_id`. *(compile-verified
  against the real compile/create/execute APIs)* — the `byteCode`-only program
  source and a richer create (typed constructor args, declared create
  OnComplete, foreign references, auto-sized extra pages) followed in
  [`contract-macro-arc56-deploy`](contract-macro-arc56-deploy.md).

A full worked example of the generated surface lives in
`examples/arc56_client.rs`.

Runtime-coupled features that touch a node are verified by **compilation**
against the real APIs; node-backed behaviour tests belong in the integration
suite.

The remaining gaps — node-backed integration tests for the runtime paths,
sourced (non-literal) default values, local/box/map state accessors, typed
return/value decoding, a richer `deploy`, and more — are tracked in #345.
Sourced default values and the local/box/map state accessors are addressed in
[`contract-macro-state-accessors`](contract-macro-state-accessors.md).

Extends [`contract-macro-from-abi-json`](contract-macro-from-abi-json.md):
its **D4** ("type mapping for initial implementation") declared an honest scope
boundary — scalar value types only, everything else a compile error. This ADR
keeps that boundary as the *fallback* but moves the goalposts: the macro's input
format becomes ARC-56, and most of what D4 left unsupported (tuples via named
structs, read-only call semantics, default arguments) becomes expressible.

Builds directly on [`abi-json-model-shared-leaf-crate`](abi-json-model-shared-leaf-crate.md):
the `algonaut_abi_model` crate introduced there is the single place this work
edits to teach the whole stack the new fields.

## Context

[`contract-macro-from-abi-json`](contract-macro-from-abi-json.md) added
`contract!("path/to/contract.json")`, which reads an **ARC-4** contract
description and generates a typed client. The shape it understands lives in
`algonaut_abi_model/src/lib.rs`: `name`, `desc`, `networks`, and `methods`
(each `arg`/`return` carrying only an ABI type *string*). `type_map.rs` maps the
scalar value types and `byte[]`, and returns an error for tuples, arrays,
`ufixed`, and transaction/reference arguments; methods using those are omitted
from the generated client (see D7), tracked in #342.

That format is the floor of what a contract can describe, not the ceiling.

### ARC-56 is the format the ecosystem actually emits

ARC-4's contract JSON predates typed-client generation; it carries only enough
metadata to *call* a method, not enough to *model* the contract. Two later ARCs
filled the gap, and ARC-56 ("Extended App Description") consolidates them:

- It is a **strict superset** of the ARC-4 contract shape — same `name`,
  `methods`, and `networks` (with the identical `{ "appID": <u64> }` value, so
  our network constructors already work against it).
- It folds in the metadata that two separate ARC-32 files used to carry, in a
  *single* document, while remaining backward-compatible with ARC-4 consumers.

Crucially, ARC-56 is **what the official SDKs consume today**. `algokit-utils`
(TypeScript and Python) defaults to ARC-56 and auto-converts older ARC-32 specs;
`algokit-client-generator` generates typed clients by reading an ARC-56 (or
ARC-32) app spec. A user who compiles a contract with current AlgoKit gets a
`*.arc56.json`. Pointed at that file, `contract!` today parses `name`/`methods`/
`networks` and silently ignores everything that makes ARC-56 worth having.

### What ARC-56 adds, and what each addition unblocks

| ARC-56 field | What it lets the macro generate | Status quo it replaces |
|---|---|---|
| `structs` map + `arg.struct` / `returns.struct` | Named Rust structs for tuple args/returns | `type_map.rs` errors on every tuple (#342) |
| `methods[].readonly` (ARC-22) | A simulate-based read path (no fee, no submit) | Every method builds a submitted txn |
| `methods[].args[].defaultValue` | Args that don't need to be passed | All args are required positionals |
| `state` (schema + `keys`/`maps`, with type encodings) | Typed accessors that decode stored values | No state surface at all |
| `events` (ARC-28) | Typed decoders over method-call logs | No event surface at all |
| `bareActions` + `methods[].actions` | create / opt-in / update / delete / close-out builders | Only NoOp calls are expressible |
| `source` / `byteCode` / `compilerInfo` | Deploy from the spec (AppFactory pattern) | No deploy story |

A subtle but load-bearing fact makes this additive rather than a rewrite: in
ARC-56 the method-argument `type` is **still the canonical ARC-4 ABI string**
(a tuple argument's `type` is `"(uint64,address)"`); `struct` is a *parallel
naming overlay*, not a replacement. So `AbiMethod::get_signature()` stays correct
unchanged, signature validation through `algonaut_abi_sig` is unaffected, and the
struct name is purely a codegen concern.

### Deprecation posture: keep ARC-4, ARC-32 is the format slated to go

We mirror the official SDKs on what is, and is not, deprecated — and ARC-4 is
**not**. ARC-56's own Backwards Compatibility section guarantees its schema
"should be compatible with all ARC-4 clients," and its Rationale describes ARC-56
as taking "the existing JSON description of a contract as described in ARC-4 and
[adding] more fields." ARC-4 remains the foundational ABI standard the SDKs build
on; `algokit-utils` keeps ARC-4 backward-compatibility rather than removing it. So
the macro keeps reading ARC-4 contract JSON — we do not deprecate it.

The format actually on the deprecation path is **ARC-32**, the two-file
predecessor ARC-56 was designed to consolidate (an ARC-32 file embeds the ARC-4
JSON *and* adds a second file — redundant information). We do not target it
directly: supporting it would mean parsing a second, redundant format for no
capability ARC-56 doesn't already give us. ARC-32 users convert once via AlgoKit,
exactly as the SDKs treat it — a convert-on-ingest legacy format, with ARC-56 the
default. The official client generators, notably, never accepted bare ARC-4
contract JSON as input at all (only ARC-32 or ARC-56), so "deprecating ARC-4" is
not even a question their tooling faces.

## Decision

### D1 — The macro's input format is ARC-56; ARC-4 is its degenerate subset

`contract!("path/to/contract.json")` keeps its signature and its compile-time
file-dependency behaviour. What changes is the contract it reads: the macro now
understands the full ARC-56 document and emits richer code when the richer fields
are present. An ARC-4 file (no `structs`/`state`/`bareActions`/`arcs`) is just an
ARC-56 document with those fields empty, and produces exactly today's output. No
version flag, no second entry point, no `contract_arc56!`: detection is by field
presence, and degradation is graceful.

### D2 — Extend `algonaut_abi_model` additively, in one place

Per [`abi-json-model-shared-leaf-crate`](abi-json-model-shared-leaf-crate.md),
`algonaut_abi_model` is the single source of truth for the wire format, shared by
the runtime `algonaut_abi` (via `#[serde(from/into)]`) and the macro. All ARC-56
additions land **there**, as `#[serde(default)]` / `Option` fields so that:

- ARC-4 files continue to deserialize (missing fields → empty/`None`);
- `algonaut_abi`'s exact-JSON round-trip tests (`abi_json_tests.rs`) still pass —
  serialising an ARC-4-shaped value omits the new fields via
  `skip_serializing_if`, keeping bytes identical;
- the macro and runtime cannot disagree about ARC-56 the way the pre-leaf-crate
  duplicate structs could have.

New model types (names indicative): `AbiContract` gains `arcs`, `structs`,
`state`, `bare_actions`, `events`, `source`, `byte_code`, `compiler_info`;
`AbiMethodArg` gains `struct_`, `default_value`; `AbiReturn` gains `struct_`;
`AbiMethod` gains `readonly`, `actions`, `events`. The model stays a pure DTO —
no resolved types, no `AppId`, no client — consistent with its charter. This
follows the project's migration-consistency principle: the type change is applied
at the shared definition and flows to every consumer and its tests, not patched
per-call-site.

### D3 — Named structs from `structs` (the headline capability)

When an arg or return carries `struct: "Name"`, the macro generates a Rust struct
from that entry in the top-level `structs` map and uses it as the parameter /
return type, with ABI tuple encode/decode derived from the field order. This is
what turns "tuple → unsupported" into a first-class typed surface and closes
the bulk of #342. Nested structs (a field whose `type` is itself a struct name or
an inline `StructField[]`) are generated recursively. The generated type name is
the PascalCase struct name, scoped to avoid collisions with the client struct.

### D4 — `readonly` methods get a simulate-based read path

ARC-22 read-only methods make no ledger writes. For a method marked
`readonly: true`, the generated builder exposes a read entry point that executes
via *simulate* and decodes the return value, rather than (or in addition to)
building a submittable `MethodCall`. This reuses the existing simulate ergonomics
from
[`atomictransactioncomposer-simulate-convenience`](atomictransactioncomposer-simulate-convenience.md);
no fee, no signature, no state change. Non-readonly methods are unaffected.

### D5 — Default argument values: inline literals now, sourced defaults later

An `arg.defaultValue` means the caller may omit it. ARC-56 defines five sources:
`literal`, `global`, `local`, `box`, `method`.

- **`literal`** is a compile-time constant and is implemented in the first ARC-56
  phase: the arg drops out of the generated method signature and the constant is
  encoded for the caller.
- **`global` / `local` / `box` / `method`** require a runtime read (a storage
  lookup or a read-only method call) to resolve the value at call time. These are
  deferred to a later phase because they need an `algod` handle inside the
  builder; until then such args remain required positionals (today's behaviour).

### D6 — State, events, and lifecycle are designed here, delivered later

These are in-scope for the ARC-56 direction but explicitly phased after structs,
because each needs a runtime capability the current value-only codegen does not:

- **State** (`state.keys` / `state.maps`): generate typed getters that read app
  global/local/box storage and decode per the declared `keyType`/`valueType`
  (ABI type, AVM type, or struct). Needs a client reader on the generated struct.
- **Events** (`events`, ARC-28): generate typed event structs and a decoder over
  a method call's logs (4-byte event selector prefix + ARC-4 tuple body).
- **Lifecycle** (`bareActions` + `methods[].actions`): generate
  create / opt-in / update / delete / close-out builders driven by the declared
  action sets. Create converges with deploy via `source` / `byteCode` /
  `compilerInfo` (an AppFactory-style story), which is the last and largest phase.

### D7 — The scope boundary omits methods rather than failing the build

Anything the macro cannot yet generate (a `ufixed` arg, a compound array, a
transaction/reference arg, an unsupported `defaultValue` source) leaves that
method out of the generated client; the omission and its reason are recorded in
the client struct's doc comment. A struct whose name is not a valid Rust
identifier is likewise skipped. (Named *and* inline-nested structs are now
generated — see the struct mapping above.)

This supersedes the predecessor ADR's "unsupported → `compile_error!`" stance.
That floor let a single unmodelled method sink an entire spec, so `contract!`
could not target real-world contracts (Reti, NFD, AlgoKit's `Arc56Test`, …),
which invariably carry at least one such method. Omitting yields a usable
*partial* client instead. Parse-level errors (a malformed spec, a bad path)
still fail loudly; ARC-56 introduces no silent *value* fallbacks.

### Scope and phasing

1. **Parse ARC-56** — D2 only. Extend the model, add an `*.arc56.json` fixture,
   round-trip tests, no codegen behaviour change. ARC-4 path untouched.
2. **Named structs** — D3. Highest value; closes most of #342.
3. **Read-only + literal defaults** — D4, D5 (literal).
4. **State + events** — D6 (state, events).
5. **Lifecycle + deploy** — D6 (actions, then `source`/`byteCode`).

Phase 1 is a safe, reviewable change on its own; later phases each stand alone.

### Alternatives considered

- **A separate `contract_arc56!` macro.** Rejected: two entry points for one
  concept, and ARC-4 being a subset means one macro can serve both. Detection by
  field presence (D1) is simpler for users and for us.
- **Support ARC-32 directly.** Rejected: it is the superseded two-file format;
  parsing it adds a redundant model for no capability the SDKs don't already get
  by converting to ARC-56 on ingest.
- **Parse the spec at runtime instead of compile time.** Rejected for the same
  reason the predecessor ADR rejected it: it forfeits the compile-time type
  safety that is the whole point of the macro.
- **Generate everything (state, events, deploy) in one change.** Rejected:
  incremental delivery, matching how the predecessor ADR scoped scalars first.
  Each phase has a different runtime dependency and deserves its own review.

## Consequences

- **ARC-56 files become first-class, ARC-4 keeps working.** The same macro reads
  both; the only difference is how much typed surface it can generate.
- **The biggest type-coverage gap closes.** Named structs make tuple args and
  returns usable, retiring most of #342 without users dropping to the dynamic
  path.
- **One shared model carries the format.** All new fields live in
  `algonaut_abi_model`; the macro and the runtime stay in lockstep by
  construction, and the round-trip tests pin the bytes.
- **The generated surface grows over phases.** Read-only reads, default args,
  state accessors, events, and lifecycle each arrive behind the same `contract!`
  entry point, so user code adopting an ARC-56 spec gains capability without API
  churn.
- **Some generated methods will need an `algod` handle.** State reads, sourced
  default values, and event decoding require I/O; the generated struct will grow
  a way to supply a client. This is a deliberate, later-phase API addition, not a
  Phase 1 concern.
- **Unsupported methods are omitted, not `compile_error!`d.** What the macro
  can't yet model is left out of the client and listed in its doc comment, so a
  real-world spec yields a usable partial client; parse-level errors still fail
  loudly. (This reverses the predecessor ADR's `compile_error!` floor — see D7.)
- **The predecessor ADR's D4 is re-scoped, not reversed.** Scalar value types
  still map as before; ARC-56 widens what "supported" means rather than changing
  any existing mapping.
