---
id: atomictransactioncomposer-simulate-convenience
title: AtomicTransactionComposer simulate convenience
abstract: Add a simulate method on AtomicTransactionComposer that wraps algod.simulate_txns, mirroring the execute path.
status: accepted
date: 2026-05-18
deciders: []
tags: []
---

# AtomicTransactionComposer simulate convenience

## Status

Accepted

## Context

`AtomicTransactionComposer` (in `src/atomic_transaction_composer/mod.rs`)
exposes a streamlined `build_group()` / `gather_signatures()` /
`execute()` flow. Reference SDKs (java, go, py) also expose a `simulate`
method on the composer so callers can run the same group against
`/v2/transactions/simulate` and parse the response into a typed
`SimulateResult`. The Rust API has no such method today — callers must
drop down to `algod.simulate_txns()` with hand-built msgpack payloads,
which the integration step-defs are not set up to do.

`tests/cucumber/features/integration/simulate.feature` includes the step:

> `I simulate the current transaction group with the composer`

and several scenarios that introspect the structured result. Without a
composer-level simulate method these scenarios cannot be wired
ergonomically; the alternative is duplicating composer state into a
side-channel simulate path inside the step-def code.

This ADR depends on `simulaterequest-model-needs-power-pack-fields`:
the simulate API surface is only complete once the underlying request
and response models cover the power-pack fields.

## Decision

Add the following surface to `AtomicTransactionComposer`:

```rust
pub async fn simulate(
    &mut self,
    algod: &Algod,
) -> Result<AtcSimulateResult, Error>;

pub async fn simulate_with(
    &mut self,
    algod: &Algod,
    request: SimulateRequest,
) -> Result<AtcSimulateResult, Error>;
```

- `simulate()` builds a default `SimulateRequest` (matching `execute()`
  semantics, but without state-changing side effects).
- `simulate_with()` accepts a pre-built request so callers can opt into
  exec-trace, extra budget, allow-more-logging, etc.
- `AtcSimulateResult` mirrors `ExecuteResult` but carries
  `SimulateTransactionGroupResult` plus parsed ABI return values per
  method call.
- Composer status transitions: `SIGNED → SIMULATED` (new variant), keeping
  `COMMITTED` reserved for `execute()`.

## Consequences

- The simulate scenarios in `simulate.feature` and `c2c.feature` become
  expressible with the same fluent API as execute.
- A new `SIMULATED` status variant requires updates to existing status
  assertion step-defs — additive change.
- Documentation and examples gain a simulate example alongside the
  execute one, lowering the bar for users testing inner-txn flows.
