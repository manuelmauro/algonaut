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

## ~~2. Add `AtomicTransactionComposer::simulate`~~

Landed — see ADR
[`atomictransactioncomposer-simulate-convenience`](adr/atomictransactioncomposer-simulate-convenience.md)
(status: accepted).

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

## ~~6. Scaffold a cucumber unit-test World~~

Landed — see ADR
[`cucumber-unit-test-scaffolding`](adr/cucumber-unit-test-scaffolding.md)
(status: accepted). The scaffold is in place plus a smoke-test pass on
the address / mnemonic / microalgos round-trip scenarios of
`offline.feature`. The remaining 16 unit features keep their stub
entries in the runner; each is now mechanical step-def work and can
land as ordinary follow-up PRs.

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
