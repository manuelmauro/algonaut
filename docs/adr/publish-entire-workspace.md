---
id: publish-entire-workspace
title: Publish the entire workspace to crates.io
abstract: 'Keep publishing every workspace crate to crates.io. Cargo features gate optional client crates rather than replacing them; the published-crate count cannot be reduced by features. Fewer published crates is only achievable by collapsing crates into feature-gated modules, and is bounded by a hard floor: the proc-macro crate algonaut_abi_macros plus its library dependencies must always be published separately.'
status: accepted
date: 2026-05-25
deciders: []
tags: [crate-layout, release, publishing, features]
---

# Publish the entire workspace to crates.io

## Status

Accepted. Records existing practice and settles a recurring question; no code
change. Prompted by adding `algonaut_abi_model`
([`abi-json-model-shared-leaf-crate`](abi-json-model-shared-leaf-crate.md)),
which raised "do we really need to publish yet another crate?"

## Context

The workspace ships as ~12 crates, and the umbrella `algonaut` crate depends on
the rest via `{ path = "…", version = "0.8.0" }`. A natural question keeps
recurring: now that we have Cargo features
([`client-feature-gates`](client-feature-gates.md)), can we publish fewer
crates — or stop publishing the sub-crates and depend on them another way?

The answer is constrained by Cargo, not preference:

- **Features don't replace crates; here they gate them.** The optional clients
  are wired as `algod = ["dep:algonaut_algod"]`, `indexer`, `kmd`, with
  `feature?/tls` forwarding. Those features are *defined in terms of* separate
  crates, so they make the multi-crate split more load-bearing, not less.
- **A published crate cannot have path-only dependencies.** `cargo publish`
  requires every normal/build dependency — including optional `dep:` ones — to
  carry a `version` resolvable on the registry. So as long as `algonaut`
  depends on the sub-crates by version, all of them must be published. There is
  no `publish = false` route that keeps `algonaut` installable, and no
  "depend on the local crate instead of the version" trick: the `version` is
  the coordinate of the *published* copy, and the `path` is already used for
  in-tree builds.
- **`cargo yank` is not "unpublish."** It only stops a version from being
  selected for *new* resolution; the code stays downloadable and existing
  lockfiles keep using it. Yanking healthy versions to "tidy up" breaks
  downstream reproducible builds and is an anti-pattern.

The only mechanism that actually reduces the published-crate count is to make
the crates stop being separate crates — collapse them into the umbrella as
feature-gated modules. That has a hard floor: `algonaut_abi_macros` is
`proc-macro = true` and can never be folded into a normal crate, and a published
proc-macro crate needs its library dependencies (`algonaut_abi_sig`,
`algonaut_abi_model`) published too. The macro also emits absolute paths
(`::algonaut_core::…`), which a collapse would force rewriting through
`::algonaut::…` re-exports. A maximal collapse therefore still publishes ~4
crates, bought with a large, breaking refactor that also removes the ability to
depend on a single piece (e.g. `algonaut_core`) standalone.

## Decision

Keep publishing the entire workspace, one version in lockstep. Treat the
multi-crate layout as the intended structure: compile-enforced module
boundaries and standalone reuse of the lower crates. Adding a focused leaf crate
when a concern is genuinely shared by two or more crates (as with
`algonaut_abi_model` and `algonaut_abi_sig`) is acceptable and expected; the
extra published artifact is the price of the cleaner layering, not a reason to
avoid it.

Do **not** attempt to cut published crates via features, path-only
dependencies, or yanking — none of them can. If reducing published artifacts
ever becomes a real goal, the only valid path is a deliberate, breaking
consolidation release that collapses crates into modules, accepting the
proc-macro floor above; that would warrant its own ADR.

If the felt pain is release *friction* rather than crate *count*, the lever is
release automation (`release-plz` / `cargo-release` / `cargo workspaces`) that
publishes the graph in dependency order from one command — not a change to the
architecture.

## Consequences

- **Predictable, settled policy.** New shared leaf crates are evaluated on
  cohesion and dependency-isolation merits; "but it's another published crate"
  is not a blocker, because the count is structural, not discretionary.
- **Lockstep versioning continues.** Every crate moves to the same version each
  release; automation is the answer to the ceremony, not consolidation.
- **Granular consumption preserved.** Downstreams may depend on `algonaut_core`,
  `algonaut_transaction`, etc. directly.
- **A consolidation escape hatch exists but is costly.** It is documented here
  so the trade-off (one breaking refactor, a ~4-crate floor, loss of standalone
  reuse) doesn't have to be rediscovered next time the question comes up.
