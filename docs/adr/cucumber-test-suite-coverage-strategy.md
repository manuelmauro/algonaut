---
id: cucumber-test-suite-coverage-strategy
title: Cucumber test-suite coverage strategy
abstract: Enumerate every algorand-sdk-testing feature in the runner; gate unsupported ones via a filter backed by an ADR plus tracking issue.
status: accepted
date: 2026-05-18
deciders: []
tags: []
---

# Cucumber test-suite coverage strategy

## Status

Accepted

## Context

The Algorand cross-SDK acceptance suite lives at
[`algorand/algorand-sdk-testing`](https://github.com/algorand/algorand-sdk-testing)
and is the reference Gherkin contract every SDK is expected to honour. The
suite contains:

- **13 integration features** (live algod + kmd + indexer harness):
  `abi`, `algod`, `applications`, `assets`, `auction`, `c2c`, `compile`,
  `dryrun`, `dryrun_testing`, `kmd`, `rekey`, `send`, `simulate`.
- **17 unit features** (in-process, no harness):
  `abijson`, `algodclient_paths`, `atomic_transaction_composer`,
  `client-no-headers`, `dryrun_trace`, `feetest`, `offline`,
  `program_sanity_check`, `rekey`, `responses`, `sourcemap`, `tealsign`,
  `transactions`, `v2algodclient_paths`, `v2algodclient_responses`,
  `v2indexerclient_paths`, `v2indexerclient_responses`.

Across both folders there are roughly **325 unique integration step
phrases** and **242 unique unit step phrases**. The Rust SDK currently
wires only three integration features into `tests/cucumber/main.rs`
(`applications`, `abi`, `c2c`) and ships no unit-feature support at all.
The comment in `tests/cucumber/main.rs` claiming `algod`/`assets` were "v1
only" is outdated — both features target the v2 endpoints algonaut
already implements.

## Decision

1. **Treat the Gherkin suite as a coverage target, not a black-box
   import.** Each feature lands behind a deliberate decision: either a
   step-def module that exercises real SDK behaviour, or an explicit
   skip backed by an ADR and a tracking issue.
2. **Distinguish "missing step definition" from "missing SDK
   capability".** The former is mechanical work and does not warrant an
   ADR; the latter gets its own ADR plus GitHub issue and the affected
   scenarios are skipped via cucumber filters in the runner.
3. **Adopt a runner layout that enumerates every `.feature` file** under
   `tests/cucumber/features/integration` and `tests/cucumber/features/unit`. Features
   without a corresponding step-def module are listed but excluded by a
   feature-set filter; the filter is the single source of truth for
   "what runs in CI".
4. **Track gaps in `docs/adr` (via `arkouda`) and in
   `docs/cucumber-pending-issues.md`.** The pending-issues file is a
   staging area; entries are converted into GitHub issues by a
   maintainer (not automatically) before being marked filed.

## Consequences

- Contributors can grep `tests/cucumber/main.rs` and see, at a glance,
  which features are live and which are stubbed.
- The bar for "the cucumber suite passes" is well-defined: every enabled
  feature must run green; every disabled feature must point at an ADR.
- Future ADRs cover specific SDK gaps the suite surfaces (simulate
  power packs, ATC simulate, TEAL source maps, dryrun builders, unit
  test scaffolding).
- Expanding coverage becomes incremental and reviewable rather than an
  all-or-nothing port.
