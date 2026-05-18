---
id: cucumber-unit-test-scaffolding
title: Cucumber unit-test scaffolding
abstract: Split the integration cucumber World from a new unit World so the 17 unit features can run without a live harness.
status: accepted
date: 2026-05-18
deciders: []
tags: []
---

# Cucumber unit-test scaffolding

## Status

Accepted

## Context

The existing `tests/step_defs/integration/world.rs` is built around
live clients (`Option<Algod>`, `Option<Kmd>`, `Option<Indexer>`) and
holds dozens of fields specific to integration scenarios (suggested
params, transient account, atomic composer state, ABI return cache).
That shape works for the harness-backed features but is the wrong
container for the 17 unit features under
`tests/features/unit/*.feature`, which exercise:

- ABI JSON parsing (`abijson.feature`)
- Client URL/header construction
  (`algodclient_paths.feature`, `client-no-headers.feature`,
  `v2algodclient_paths.feature`, `v2indexerclient_paths.feature`)
- Fixture-driven response deserialization
  (`responses.feature`, `v2algodclient_responses.feature`,
  `v2indexerclient_responses.feature`)
- Offline transaction signing, fee maths, TEAL signing, rekey
  semantics, dryrun trace parsing, source-map parsing, program sanity,
  atomic composer unit invariants.

None of these need a running algod / kmd; many run faster if they
*can't* accidentally call one.

## Decision

1. Add `tests/step_defs/unit/` mirroring the integration tree:
   - `world.rs` with a `UnitWorld` containing only the fixture / parser
     state each unit feature needs (raw JSON inputs, parsed structs,
     constructed transactions, etc.).
   - One step-def module per unit feature (`abijson.rs`, `offline.rs`,
     `tealsign.rs`, …).
2. Drive the unit features from `tests/features_runner.rs` with a
   second `World::cucumber()` builder so cross-talk between the two
   worlds is impossible.
3. Use cucumber tags (`@unit`, `@integration`) to keep filters in the
   runner declarative — the harness CI job filters to `@integration`,
   the unit job to `@unit`. Both run on every PR.

## Consequences

- Unit features run on every PR without needing the algorand harness,
  giving fast feedback on serde / SDK-internal regressions.
- The integration runner stays focused and the integration World stops
  accreting fields for unit-only state.
- Splits the maintenance work into two clean piles: harness-bound and
  in-process. New step-defs land in whichever pile fits.
