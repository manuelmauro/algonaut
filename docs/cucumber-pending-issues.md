# Pending GitHub issues

This file is a staging area for issues we want to file against this
repository. Each entry maps 1:1 to an ADR under `docs/adr/`. A
maintainer reviews the drafts, files them with `gh issue create`, and
then strikes the entry through (or removes it) — this file is not a
permanent log, just a hand-off buffer.

The titles are deliberately concise; the body templates are copy-paste
ready (HEREDOC them into `gh issue create --body "$(cat <<'EOF' ... EOF)"`).

---

## ~~1. Extend `SimulateRequest` (and friends) with power-pack fields~~

Landed — see ADR
[`simulaterequest-model-needs-power-pack-fields`](adr/simulaterequest-model-needs-power-pack-fields.md)
(status: accepted). The `simulate.feature` cucumber scenarios still
need `AtomicTransactionComposer::simulate` (ADR
`atomictransactioncomposer-simulate-convenience`) and a chunk of
step-defs before they can run.

---

## 2. Add `AtomicTransactionComposer::simulate`

**Labels:** `area/atomic-composer`, `kind/feature`, `cucumber-blocker`
**ADR:** [`atomictransactioncomposer-simulate-convenience`](atomictransactioncomposer-simulate-convenience.md)

```markdown
`AtomicTransactionComposer` exposes `build_group`, `gather_signatures`,
`submit`, and `execute`, but not `simulate`. Reference SDKs (java, go,
py) all expose a composer-level simulate.

`tests/features/integration/simulate.feature` has the step
`I simulate the current transaction group with the composer`, and
`c2c.feature` scenarios benefit from the same surface for inspecting
inner-txn trees without committing.

Proposed surface (see ADR for detail):

```rust
pub async fn simulate(&mut self, algod: &Algod)
    -> Result<AtcSimulateResult, Error>;

pub async fn simulate_with(&mut self, algod: &Algod, request: SimulateRequest)
    -> Result<AtcSimulateResult, Error>;
```

### Acceptance
- [ ] `AtcSimulateResult` mirrors `ExecuteResult` plus
      `SimulateTransactionGroupResult` and parsed ABI returns.
- [ ] A new `Simulated` composer-status variant is added.
- [ ] At least one example exercises the new method.
- [ ] Depends on the power-pack ticket above.
```

---

## ~~3. Add a TEAL source-map decoder~~

Landed — see ADR
[`teal-source-map-decoder`](adr/teal-source-map-decoder.md)
(status: accepted). The integration `@compile.sourcemap` scenario is
live; the unit `sourcemap.feature` scenarios remain gated on
`cucumber-unit-test-scaffolding`.

---

## ~~4. Add a `DryrunRequestBuilder`~~

Landed — see ADR
[`dryrun-request-builder`](adr/dryrun-request-builder.md)
(status: accepted). The integration scenarios in `dryrun.feature` and
`dryrun_testing.feature` are now live; the unit
`dryrun_trace.feature` still waits on `cucumber-unit-test-scaffolding`.

---

## ~~5. Add `state_proof_key` to `KeyRegistration`~~

Landed — see ADR
[`keyregistration-v2-state-proof-key`](adr/keyregistration-v2-state-proof-key.md)
(status: accepted).

---

## 6. Scaffold a cucumber unit-test World

**Labels:** `area/tests`, `kind/infra`, `cucumber-blocker`
**ADR:** [`cucumber-unit-test-scaffolding`](adr/cucumber-unit-test-scaffolding.md)

```markdown
The 17 features under `tests/features/unit/` don't need a live algod
or kmd. Today our cucumber `World` is built for the integration suite
and is the wrong container for unit-only state.

Proposed work:

- Add `tests/step_defs/unit/` mirroring the integration tree, with a
  `UnitWorld` containing only fixture/parser state.
- Drive the unit features from `tests/features_runner.rs` with a
  second `World::cucumber()` builder.
- Use `@unit` / `@integration` tags so CI jobs can filter
  declaratively.

### Acceptance
- [ ] `cargo test --test features_runner --` runs both worlds in
      sequence on a clean checkout (no harness needed for `@unit`).
- [ ] At least one unit feature is exercised end-to-end as a smoke
      test (`feetest.feature` is a good candidate — pure maths, no IO).
```

---

## 7. (No issue) Expand step-def coverage for already-supported features

The remaining work is mechanical step-def writing for features where
the SDK already covers the underlying capability. These do **not**
warrant individual issues — they will land as ordinary PRs against
ADR-0001's coverage tracker. The list:

- `algod.feature`
- `assets.feature`
- `auction.feature`
- `kmd.feature`
- `rekey.feature`
- `send.feature`
- `compile.feature` (everything except the mapping-enabled scenario)

Each PR should flip the corresponding `gate` in
`tests/features_runner.rs` from `Some(...)` to `None`.
