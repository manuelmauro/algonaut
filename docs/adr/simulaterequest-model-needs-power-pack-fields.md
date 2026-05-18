---
id: simulaterequest-model-needs-power-pack-fields
title: SimulateRequest model needs power-pack fields
abstract: Extend the algod SimulateRequest model with allow-more-logging, extra-opcode-budget, allow-unnamed-resources, and exec-trace-config so simulate.feature can drive the endpoint.
status: proposed
date: 2026-05-18
deciders: []
tags: []
---

# SimulateRequest model needs power-pack fields

## Status

Proposed

## Context

`tests/features/integration/simulate.feature` exercises the algod
`/v2/transactions/simulate` endpoint with several "power-pack" toggles:

| Step phrase                                                                | Underlying field                       |
|----------------------------------------------------------------------------|----------------------------------------|
| `I allow more logs on that simulate request.`                              | `SimulateRequest.allow-more-logging`   |
| `I allow N more budget on that simulate request.`                          | `SimulateRequest.extra-opcode-budget`  |
| `I allow exec trace options "stack,scratch" on that simulate request.`     | `SimulateRequest.exec-trace-config`    |
| `the simulation should report a failure at group ... with message ...`     | response inspection                    |
| `Nth unit in the "approval" trace at txn-groups path "0" should ...`       | exec-trace inspection                  |
| `I check the simulation result has power packs ...`                        | round-trip request echo                |

The current model at
`algonaut_algod/src/models/simulate_request.rs` only declares
`txn_groups`:

```rust
pub struct SimulateRequest {
    #[serde(rename = "txn-groups")]
    pub txn_groups: Vec<SimulateRequestTransactionGroup>,
}
```

The algod OpenAPI spec at HEAD includes additional fields:

- `allow-more-logging: bool`
- `allow-empty-signatures: bool`
- `allow-unnamed-resources: bool`
- `extra-opcode-budget: u64`
- `exec-trace-config: SimulateTraceConfig` (`enable`, `stack-change`,
  `scratch-change`, `state-change`)
- `round: u64` and `fix-signers: bool` in newer revisions.

The response side (`SimulateTransactionGroupResult`,
`SimulateTransactionResult`) is similarly incomplete: it lacks
`exec-trace`, `txn-results.app-budget-consumed`, and the per-unit trace
payload that `simulate.feature` introspects.

## Decision

1. **Regenerate the algod OpenAPI models against the current
   `algod.oas3.json`** (commit-pinned for determinism), or extend
   `simulate_request.rs` and the simulate response models by hand if a
   full regeneration is too disruptive.
2. **Expose ergonomic setters** on a wrapper newtype
   (`simulate_request_builder.rs`) so step-defs can write
   `SimulateRequestBuilder::new(txn_groups).allow_more_logging(true)`
   rather than mutating raw fields.
3. **Add the exec-trace model types** (`SimulateTraceConfig`,
   `SimulationTransactionExecTrace`, `SimulationOpcodeTraceUnit`,
   `ApplicationStateOperation`, etc.) so step-defs can inspect stack,
   scratch, and state changes.
4. **Update `AtomicTransactionComposer`** once ADR
   `atomictransactioncomposer-simulate-convenience` lands so simulate
   calls go through the high-level API and benefit from the same
   builder.

## Consequences

- `simulate.feature` and the simulate-flavoured `c2c` scenarios become
  runnable.
- The autogeneration pipeline gets re-validated; if drift exists in
  other endpoints, those follow-up gaps surface in the same pass.
- The existing field name (`txn-groups`) and JSON shape are preserved
  so existing callers do not break — only additive fields.
