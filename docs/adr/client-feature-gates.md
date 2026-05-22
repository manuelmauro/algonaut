---
id: client-feature-gates
title: Cargo feature gates for the client modules
abstract: Put the algod, indexer, and kmd client modules behind additive Cargo features (algod, indexer, kmd), offline core always on, default = the three clients plus a TLS backend so the gating is non-breaking; default-features=false enables slim and wasm32 builds that drop reqwest. The raw openapi_* re-exports are not gated but removed, as the closing act of the hand-named-types migration (north-star D3 / hide-generated-types).
status: proposed
date: 2026-05-22
deciders: []
tags: [build, features, wasm, modularity]
---

# Cargo feature gates for the client modules

## Status

Proposed

## Context

The umbrella `algonaut` crate today pulls in every workspace member
unconditionally. `lib.rs` re-exports the offline crates (`abi`, `core`,
`crypto`, `model`, `transaction`), re-exports the three generated network
clients under raw names (`pub use algonaut_algod as openapi_algod`, plus
`openapi_indexer`, `openapi_kmd`), and exposes the hand-written high-level
modules `algod`, `indexer`, `kmd`, `atomic`, `dryrun`, `simulate`. All of it
compiles for every consumer, whatever they actually use.

### 1. The only features today gate one crate's TLS, inconsistently

The root `Cargo.toml` declares exactly three features:

```toml
[features]
default = ["native"]
native  = ["algonaut_kmd/native"]
rustls  = ["algonaut_kmd/rustls"]
```

`native`/`rustls` forward only to `algonaut_kmd`. But `algonaut_algod` and
`algonaut_indexer` declare their `reqwest` dependency as
`reqwest = { workspace = true, features = ["json", "multipart", "query"] }`,
and the workspace `reqwest` carries default features — so `native-tls` is
*always* compiled into the algod and indexer clients regardless of which
feature the consumer picked. Selecting `rustls` does not give you a
rustls-only build; it gives you rustls for kmd and native-tls everywhere
else. The one feature axis the crate already has is silently broken for two
of its three clients.

### 2. Offline users still pay for three HTTP clients

A consumer who only builds and signs transactions offline — a CLI that emits
a signed blob, a hardware-wallet companion, a test fixture generator — has no
use for `algod`, `indexer`, or `kmd`, yet links all three `reqwest`-based
clients and their transitive TLS stacks. There is no way to ask for "just the
transaction and signing types."

This bites hardest on `wasm32`. The 0.5 line was specifically reworked so
`wasm32` builds need no C toolchain (`ring` → `ed25519-dalek`), and `lib.rs`
already carries a `cfg(target_arch = "wasm32")` block. The natural wasm
consumer — build a transaction in the browser, hand it to a wallet to
sign — wants the offline core and nothing that drags in a native HTTP/TLS
stack. Today they cannot express that.

### 3. The raw generated clients are still part of the public surface

`pub use algonaut_algod as openapi_algod` (and the indexer/kmd equivalents)
put machine-generated type names on the public API.
[`ideal-type-safe-ergonomic-api`](ideal-type-safe-ergonomic-api.md) item
**D3** and the first cut in
[`hide-generated-types`](hide-generated-types.md) already commit to hiding
these: each client method migrates to a hand-named type in
`algonaut_model::client_types`, and "the `pub use algonaut_algod as
openapi_algod` re-exports in `lib.rs` are removed" once coverage is complete.

That coverage is still early. **56 public client methods** — 35 on algod, 21
on indexer — return a generated type *by name* today
(`RawTransaction200Response`, `PendingTransactionResponse`,
`GetBlockLogs200Response`, `BlockResponse`, `models::Box`, …), against only
**3** wrapped so far (`SuggestedParams`, `NodeStatus`, `Supply`). So the
re-exports are still load-bearing — our own unit-test `World` names ~14
responses through `algonaut::openapi_*` — and the question this ADR settles
is what to *do* with them. The decision (D3 below) is to **remove** them by
finishing the hand-named migration, not to preserve them behind a feature:
gating a generated-type escape hatch the project has already decided to
delete would only entrench it.

### 4. What actually depends on what

Mapping the modules to the generated crates decides which gates are even
possible:

| Root item                          | Needs generated crate        |
|------------------------------------|------------------------------|
| `core`, `crypto`, `encoding`, `abi`, `abi_sig`, `transaction`, `model` | none — pure offline |
| `error`, `util`                    | none                         |
| `algod` module                     | `algonaut_algod`             |
| `indexer` module                   | `algonaut_indexer`           |
| `kmd` module                       | `algonaut_kmd`               |
| `atomic`, `simulate`, `dryrun`     | `algonaut_algod::models::*`  |
| `openapi_algod` / `_indexer` / `_kmd` re-exports | the matching crate |

Two facts make the split clean. `algonaut_model` does **not** depend on any
client crate, so the hand-named `client_types` (`SuggestedParams`,
`NodeStatus`, …) and the entire offline foundation can stay always-on.
`atomic`, `simulate`, and `dryrun`, however, all import
`algonaut_algod::models` directly (e.g. `PendingTransactionResponse`,
`SimulateRequestTransactionGroup`, `DryrunRequest`) — they are coupled to the
algod crate today and so must ride whatever gate `algonaut_algod` rides.

## Decision

Introduce a small set of **additive** Cargo features on the root crate. The
offline core stays unconditional; each network client and the raw re-export
layer become opt-out toggles. Crucially, `default` keeps **today's full
surface**, so this change compiles every existing consumer unchanged — the
gates are a capability for people who want *less*, not a removal of anything.

### D1 — The feature set

```toml
[features]
default = ["algod", "indexer", "kmd", "native"]

# High-level network clients. Each pulls its generated crate and unlocks
# the matching hand-written module(s).
algod   = ["dep:algonaut_algod"]                 # algod + atomic + simulate + dryrun
indexer = ["dep:algonaut_indexer"]
kmd     = ["dep:algonaut_kmd"]

# TLS backend, forwarded to *every* enabled client (see D4).
native = ["algonaut_kmd?/native", "algonaut_algod?/native", "algonaut_indexer?/native"]
rustls = ["algonaut_kmd?/rustls", "algonaut_algod?/rustls", "algonaut_indexer?/rustls"]
```

The generated crates become optional dependencies (`optional = true`), pulled
in only by their feature via `dep:`. The `feature?/…` weak-dependency syntax
forwards TLS selection to a client only when that client is also enabled, so
`--no-default-features --features "algod,rustls"` is a coherent rustls-only
algod build.

These are the names a consumer writes. Mapping the examples from the request:
`algod` and `indexer` are literal feature names. The raw-client re-export
(`raw_openapi_clients` in the request — the clients are openapi-generator
output, see [`openapi-client-regeneration`](openapi-client-regeneration.md))
is deliberately **not** a feature here; it is handled by removal, see D3. A
per-workspace-crate `crate`-style gate is considered and rejected in D3.

### D2 — The offline core is always on, gated modules follow their crate

`abi`, `core`, `crypto`, `encoding`, `abi_sig`, `transaction`, `model`,
`error`, and `util` carry no feature gate — they are the floor every build
gets. `#[cfg(feature = "...")]` guards the rest in `lib.rs`:

```rust
#[cfg(feature = "algod")]
pub mod algod;
#[cfg(feature = "indexer")]
pub mod indexer;
#[cfg(feature = "kmd")]
pub mod kmd;

// atomic / simulate / dryrun import algonaut_algod::models today,
// so they ride the algod gate until that coupling is removed (see D3).
#[cfg(feature = "algod")]
pub mod atomic;
#[cfg(feature = "algod")]
pub mod simulate;
#[cfg(feature = "algod")]
pub mod dryrun;
```

### D3 — Remove the raw re-exports by finishing the hand-named migration

The `openapi_*` re-exports are **not** turned into a kept feature; they are
**removed**, once the hand-named migration that makes them unnecessary is
complete. Both halves — wrapping the remaining responses and deleting the
re-exports — are owned by [`hide-generated-types`](hide-generated-types.md) /
north-star **D3**, not by a feature in this ADR.

The reasoning is in the numbers from Context §3: with 56 methods still
returning generated types and only 3 wrapped, neither gating the re-exports
behind a `raw_openapi_clients` feature nor deleting them outright *today*
would hide a generated type. It would only make those 56 return types
**unnameable through `algonaut`** — a caller who must name an un-migrated
response would have to add their own `algonaut_algod` dependency and import
`algonaut_algod::models::…`, re-creating the two-paths problem D3 set out to
kill (north-star item 3), relocated into every user's `Cargo.toml`. A
default-off feature would have the same effect for the default build, just
opt-in-able. So the only move that actually advances the goal is to finish
wrapping, *then* delete.

The sequence:

1. **Finish D3 coverage** (owned by `hide-generated-types`): wrap the
   remaining ~53 responses in hand-named `algonaut_model` types, so no public
   client method returns a generated type.
2. **Delete the three `pub use algonaut_algod as openapi_*` lines.** The
   generated crates stay as optional, *internal-only* dependencies behind the
   `algod`/`indexer`/`kmd` gate; nothing re-exports them.
3. The unit-test `World` (which names ~14 responses through
   `algonaut::openapi_*`) and any remaining direct `algonaut_algod::models`
   uses in `tests/` migrate to the hand-named types in the same effort.

**Transition.** Until step 2 lands, the re-exports remain in the (default)
surface but must ride their client's gate so a slim build does not reference
an uncompiled crate:

```rust
#[cfg(feature = "algod")]
pub use algonaut_algod as openapi_algod;
#[cfg(feature = "indexer")]
pub use algonaut_indexer as openapi_indexer;
#[cfg(feature = "kmd")]
pub use algonaut_kmd as openapi_kmd;
```

They are deleted, not demoted to an opt-in feature — there is no
`raw_openapi_clients`.

Three alternatives were weighed and rejected:

- **Gate-and-keep behind `raw_openapi_clients`** (default-on or default-off).
  Rejected: it preserves a generated-type escape hatch the project has decided
  to eliminate, and the default-off variant still leaves 56 methods whose
  return types cannot be named without the feature. Removal via finishing D3
  is the chosen end state, not a permanent gate.
- **A feature per workspace crate** (the literal `crate` example — e.g. a
  `crypto` or `transaction` toggle). Rejected: the offline crates are a tight,
  cheap, no-network foundation that everything else needs; gating them buys no
  meaningful build savings and multiplies the feature-combination matrix CI
  must cover. The savings live entirely in the three `reqwest` clients.
- **A separate gate for `atomic`/`simulate`/`dryrun`.** Rejected for now:
  these modules import `algonaut_algod::models` directly, so they cannot
  compile without the algod crate regardless. Decoupling them (consuming the
  hand-named `algonaut_model` types instead of `algonaut_algod::models`, per
  D3) is the prerequisite; once that lands, an `atomic`-without-`algod` gate
  becomes possible and can get its own follow-up.

### D4 — Fix the TLS leak while we are here

Give `algonaut_algod` and `algonaut_indexer` the same `native`/`rustls`
features `algonaut_kmd` already has, declaring their `reqwest` with
`default-features = false` so the backend is genuinely chosen by the feature
rather than always-on:

```toml
# in algonaut_algod / algonaut_indexer
reqwest = { workspace = true, default-features = false, features = ["json", "multipart", "query"] }

[features]
default = ["native"]
native  = ["reqwest/native-tls"]
rustls  = ["reqwest/rustls"]
```

After this, the root `native`/`rustls` features forward to all three clients
(D1), and `rustls` produces a genuinely native-tls-free build.

## Consequences

- **The gating is non-breaking by construction.** A consumer who keeps
  `default` (or omits the `algonaut` features entirely) keeps the identical
  module surface they have today — same modules, same `openapi_*` re-exports
  during the transition, same TLS backend (modulo the fix below). The only
  consumers who must act for the *gates* are those who opt out with
  `default-features = false`, and they opt in to exactly the features they
  name. The breaking change in this area is the eventual *removal* of the
  `openapi_*` re-exports (D3), which lands with that migration, not with the
  gates.
- **Slim and wasm builds become expressible.** `algonaut = { version = "...",
  default-features = false }` yields the offline core only — transactions,
  signing, ABI, the domain model — with no `reqwest` and no TLS stack. A
  browser/wasm signer can build and sign without dragging in a native HTTP
  client. This is the headline win.
- **The broken TLS axis is fixed.** `rustls` finally means rustls everywhere,
  not "rustls for kmd, native-tls for the rest." This is a behavioural change
  for anyone who selected `rustls` and unknowingly still linked native-tls via
  algod/indexer; on most platforms it removes a system OpenSSL dependency.
- **The raw re-exports are removed, not gated.** This ADR introduces no
  escape-hatch feature; the `openapi_*` re-exports are deleted as the closing
  act of north-star D3 / [`hide-generated-types`](hide-generated-types.md),
  after the remaining ~53 client responses are wrapped in hand-named types.
  Finishing that migration is therefore a prerequisite for the final surface.
  The client gates (D1/D2/D4) can land first and independently; the re-export
  removal lands with D3.
- **CI must cover the feature matrix.** At minimum: `--no-default-features`
  (offline core compiles standalone), each client feature in isolation,
  `--no-default-features --features "algod,rustls"` (the rustls path), and
  `--all-features`. Without these, a `#[cfg]` that references a gated type
  from always-on code, or a feature that fails to build alone, regresses
  silently. A `cargo hack --feature-powerset --depth 2` check on the root
  crate is the cheap insurance.
- **`#[cfg]` discipline is now load-bearing.** Re-exports, the `atomic` /
  `simulate` / `dryrun` modules, and any always-on code that touches a gated
  type must carry matching `cfg` guards. Examples and integration tests that
  hit the network already implicitly need `algod`/`indexer`/`kmd`; those
  targets should declare `required-features` so a slim build skips them
  cleanly instead of failing to compile.
- **Cost.** Three new feature stanzas, optional-dep plumbing, and the cfg
  guards are mechanical and one-time. The ongoing tax is the CI matrix and the
  habit of gating new client-touching code — modest, and the kind of thing
  `cargo hack` enforces automatically.
- **Non-goals.** This ADR does not gate the offline workspace crates
  individually, does not split `atomic`/`simulate`/`dryrun` from `algod`
  (blocked on D3 decoupling), and does not change the default build's
  behaviour beyond the TLS-backend fix in D4. It introduces the client gates
  and the TLS axis; it introduces no escape-hatch feature for the generated
  clients — their re-exports are removed by completing
  [`hide-generated-types`](hide-generated-types.md) (north-star D3), which
  this ADR treats as a prerequisite for the final surface rather than
  re-specifying.
