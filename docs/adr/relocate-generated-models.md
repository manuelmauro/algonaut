---
id: relocate-generated-models
title: Relocate generated models into algonaut_model
abstract: Author the algod/indexer OpenAPI model structs in algonaut_model (as algod/indexer submodules) instead of the client crates, so the public API exposes clean algonaut_model paths and the openapi_* re-exports can be removed — without hand-re-typing the ~142-type generated graph. The 39 *200Response envelopes are renamed at generation (inline-schema name mapping) and fields domain-typed via the existing type-overrides, so no *200Response name leaks either. The client crates keep the API/transport code and depend on algonaut_model for models (same dependency direction, no cycle). Fully supersedes hide-generated-types — rename + type-overrides reproduce what its three hand-wrappers did, from generation, so the wrappers retire.
status: accepted
date: 2026-05-22
deciders: []
tags: [api, openapi, codegen, crate-structure]
---

# Relocate generated models into algonaut_model

## Status

Accepted. Reframes how
[`ideal-type-safe-ergonomic-api`](ideal-type-safe-ergonomic-api.md) item
**D3** is achieved, and supersedes
[`hide-generated-types`](hide-generated-types.md) (whose three hand-wrappers
fold back into generation — see Decision). Builds on the generation pipeline
from [`openapi-client-regeneration`](openapi-client-regeneration.md).

## Context

D3 of the north-star and the first cut in `hide-generated-types` commit to a
clear end state: **no `*200Response` name and no `openapi_*` path appears in
user code or rustdoc**, and the `pub use algonaut_algod as openapi_algod`
re-exports in `lib.rs` are removed. `hide-generated-types` pursued this by
hand-naming each response into a wrapper type in
`algonaut_model::client_types`, converting at the client edge. Its first cut
shipped three wrappers — `SuggestedParams`, `NodeStatus`, `Supply`.

Carrying that strategy to completion does not scale.

### 1. Removing the re-exports means hiding ~142 nested types, not ~40 flat ones

About 40 client methods still return a generated type, but the cost is the
**transitive closure** of those types, not the 40 envelopes:

- The ugly `*200Response` envelopes are mostly shallow —
  `RawTransaction200Response` is just `{ tx_id }`.
- The shared models are deep. `Account` has 27 fields and pulls in
  `ApplicationLocalState`, `ApplicationStateSchema`, `AssetHolding`,
  `Application`, `Asset`, `AccountParticipation`. `SimulateTransaction200Response`
  pulls in an exec-trace tree (`…GroupResult → …Result → …ExecTrace →
  …OpcodeTraceUnit`). `Block` is large.

`algonaut_algod` and `algonaut_indexer` carry **71 model files each (142
total)**. Hand-re-typing the closure to remove the re-exports is thousands of
lines of code that *duplicates the generated layer by hand* — and that
duplicate must then be kept in sync on every regen, which fights directly
against the reproducible-regeneration goal of
[`openapi-client-regeneration`](openapi-client-regeneration.md).

### 2. The re-exports cannot simply be moved

The reason a generated type is only reachable via `openapi_algod` is that it
is **authored in the client crate**. Two seemingly cheaper fixes both fail:

- **Re-export the generated types from `algonaut_model`.** Impossible:
  `algonaut_algod` already depends on `algonaut_model` (for domain types), so
  `algonaut_model` re-exporting from `algonaut_algod` is a dependency cycle.
- **Re-home them at the umbrella crate** (`pub use algonaut_algod::models::Account
  as algonaut::model::Account`). Doesn't hide anything: `Account`'s fields are
  typed `algonaut_algod::models::ApplicationLocalState`, so the `openapi` path
  still appears in rustdoc unless every nested type is re-typed too.

So the leak is structural: it follows from *where the models are authored*.

### 3. Most of the wrapper work would be pure renaming

The three shipped wrappers earn their keep by introducing **domain types** the
generated layer cannot express — `SuggestedParams.last_round: Round`,
`Supply.total_money: MicroAlgos`. But for the bulk of the 142 types, a
hand-written wrapper would be a field-for-field copy with the same types — a
rename with a maintenance tax and no semantic gain.

## Decision

Stop hiding the generated models behind a hand-written layer. Instead, change
**where the generated models are authored**: emit the model structs into
`algonaut_model`, and keep only the operation/transport code in the client
crates.

### D1 — Models are authored in `algonaut_model`

The OpenAPI **model** structs are generated into `algonaut_model` under
per-spec submodules:

```
algonaut_model::algod      // models generated from the algod spec
algonaut_model::indexer    // models generated from the indexer spec
```

Per-spec submodules, **not** one flattened namespace: algod and indexer each
define their own `Account`, `Block`, `Transaction`, `Asset`, … with different
fields, so flattening would collide. The submodule also documents which spec a
type came from at the use site (`model::algod::Account` vs
`model::indexer::Account`).

`algonaut_algod` / `algonaut_indexer` keep their **apis** (the
operation functions, `Configuration`, the HTTP/`reqwest` plumbing) and depend
on `algonaut_model` for the model types — the *same* dependency direction that
exists today (`client → model`), so there is **no cycle**. The clients shrink
to transport layers; `algonaut_model` becomes the home for both the domain
types and the wire models, which is what its name already implies.

### D2 — The public API speaks `algonaut_model`, and the re-exports are removed

Client methods return `algonaut_model` paths (e.g.
`algonaut::model::algod::Account`). With no generated type reachable only via
the client crate, the three `pub use algonaut_* as openapi_*` re-exports in
`lib.rs` are deleted — closing the D3 line item. No `openapi_*` path appears
in user code or rustdoc.

### D3 — Resolve the two cycle-risk references

`algonaut_model` must not gain a dependency on `algonaut_transaction` (which
already depends on `algonaut_model`). Exactly **two of the 142 model files**
reference `algonaut_transaction`, and both have an existing `algonaut_model`
equivalent to point at:

| Model file | Today references | Redirect to |
|---|---|---|
| `transaction_params_200_response.rs` | `algonaut_transaction::builder::TransactionParams` | `algonaut_model::client_types::TransactionParams` (already lives there) |
| `simulate_request_transaction_group.rs` | `algonaut_transaction::SignedTransaction` | `algonaut_model::transaction::ApiSignedTransaction` (already exists) |

These redirections are encoded in the generator's `type-overrides.json` /
import mappings (the mechanism `openapi-client-regeneration` already uses), so
they survive regeneration.

### D4 — Generation pipeline: split models from apis

`make generate-clients` is extended so the rust generator's **model** output
lands in `algonaut_model/src/{algod,indexer}/` and the **api** output stays in
the client crate, with the api template importing
`algonaut_model::{algod,indexer}::…` instead of `crate::models::…`. This is a
template + config change layered on the existing `openapi/` setup (custom
`model.mustache`, `preprocess.py`, per-client configs); the pinned specs and
the `u64`/domain-type substitutions are unchanged.

### D5 — Rename the response envelopes, and retire the hand-wrappers

Relocation hides the `openapi_*` *path*, but the synthesized envelope *names*
would still surface as `algonaut::model::algod::RawTransaction200Response` —
and the north-star (item 3) bars `*200Response` *names* from user code and
rustdoc, not just the paths. So the **39 inline-response envelopes** (25 algod
+ 14 indexer) are renamed at generation to intentional names:
`RawTransaction200Response` → `SubmitResponse`, `GetBlockHash200Response` →
`BlockHash`, `TealCompile200Response` → `CompiledTeal`,
`TransactionParams200Response` → `SuggestedParams`, and so on.

The mechanism is the generator's **inline-schema name mapping**
(`inlineSchemaNameMappings` in the per-client config — the option built for
renaming the synthesized `<op>200Response` names, available on the pinned
v6.6.0); if a case escapes it, `preprocess.py` hoists the inline 200-response
schema into a named component instead. Either way it is a declarative table
under `openapi/`, reproducible across regens, with a fail-loud check (à la
`type-overrides.json`) when a mapped operation disappears upstream. (Renaming
inline *response* schemas does not require the 7.x `modelNameMappings` option.)

This changes the calculus for `hide-generated-types`. A generated model that
is **renamed** (here) and has its fields **domain-typed** (via the existing
`type-overrides.json` — e.g. `SuggestedParams.last_round → Round`,
`Supply.total_money → MicroAlgos`) is *exactly* what the three hand-wrappers
produce, minus the hand-maintenance. So the wrappers are **fully retired**,
not merely reduced: the three structs in `algonaut_model::client_types` are
removed, their domain types move to `type-overrides.json`, and any ergonomics
they carried (e.g. `impl TransactionParams for SuggestedParams`) become
ordinary `impl` blocks in `algonaut_model` on the generated type.

## Consequences

- **D3 completes without a parallel hand-maintained layer.** The generated
  models stay generated — they just live in `algonaut_model`. Regeneration
  remains the single source of truth, preserving the
  [`openapi-client-regeneration`](openapi-client-regeneration.md) workflow
  instead of fighting it.
- **Clean public paths; re-exports gone.** `algonaut::model::algod::Account`
  replaces `algonaut::openapi_algod::models::Account`; the `openapi_*`
  re-exports are deleted, satisfying D3's path-hiding goal in full.
- **`algonaut_model` grows; the client crates shrink** to API/transport
  layers. This matches the crates' names and removes the awkward "the model
  crate doesn't contain the wire models" situation.
- **Breaking — but a path change, not a re-type.** Public type *paths* move
  (`algonaut_algod::models::X` → `algonaut_model::algod::X` /
  `algonaut::model::…`). Mechanical for callers; pre-1.0. Notably the type
  *shapes* are unchanged, so unlike the wrapper approach there is no risk of a
  hand-copied field drifting from the wire model.
- **The cost moves into the generation pipeline.** The real work is the
  template/config change to split models from apis and rewrite import paths,
  plus the two cycle-risk redirections — a focused, one-time pipeline effort
  rather than an open-ended hand-typing slog. It needs a careful regen diff
  (the tool `openapi-client-regeneration` exists to provide).
- **Fully supersedes `hide-generated-types`.** Envelope renaming (D5) plus the
  existing domain-type overrides reproduce, from generation, what its three
  hand-wrappers did — so the wrappers retire and the whole wrap-every-response
  strategy is replaced. `hide-generated-types` is marked superseded-by this ADR.

### Open questions / non-goals

- **Envelope names.** *That* the 39 envelopes are renamed is decided (D5);
  the specific clean names are a small design task. A handful are non-obvious —
  the paginated indexer `search_*`/`lookup_*` list responses, and the algod
  `account_*_information` family — and want names that read well at call sites.
- **Sequencing.** Whether to relocate algod and indexer in one change or two.
- **Does `algonaut_algod`/`algonaut_indexer` remain a separate crate at all,**
  or eventually fold its thin api layer elsewhere — explicitly out of scope
  here.
