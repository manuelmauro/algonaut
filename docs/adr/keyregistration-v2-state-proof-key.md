---
id: keyregistration-v2-state-proof-key
title: KeyRegistration v2 state-proof key
abstract: Add state_proof_key to KeyRegistration so online key-registration transactions match the v2 consensus surface used by send.feature.
status: accepted
date: 2026-05-18
deciders: []
tags: []
---

# KeyRegistration v2 state-proof key

## Status

Accepted

## Context

`tests/cucumber/features/integration/send.feature` includes a
`@send.keyregtxn` scenario that exercises **V2** key registration with
the `online`, `offline`, and `nonparticipation` variants. Other SDKs
build the online variant from a fixed test fixture that includes a
state-proof public key (`sprfkey`, a 64-byte BLS public key):

```python
transaction.KeyregOnlineTxn(
    sender, params,
    votekey="9PYsmlBevatTWVRcfDLqzpyaURRTbjPo+oTrFkXgKHE=",
    selkey="UkpZx9Vfo0Q1+/8mE2KCDdQ72Y0AKEK9DnFvL3yWZ2c=",
    sprfkey="WaA5UWiVDzD6QY/ZxNi0Pc4xL4FxQa3kjlrZmkSMcEUjGFQqRGo3CSNZ9D8GAr+5e7TgQHM2RfsdJ4yLpcfkRA==",
    votefst=0, votelst=30001, votekd=10000, nonpart=False,
)
```

Our `algonaut_transaction::transaction::KeyRegistration` model omits
the state-proof key and the corresponding builder
`RegisterKey::online` does not accept one. Since the v34 consensus
upgrade, online registration requires `sprfkey`; without it the algod
harness rejects the transaction. Offline and nonparticipation variants
are fine — they don't carry vote material.

## Decision

1. Extend `algonaut_transaction::transaction::KeyRegistration` with
   `state_proof_key: Option<StateProofPk>` where `StateProofPk` is a
   newtype around `[u8; 64]` (matching the protocol's BLS key length).
2. Update the msgpack serialization to write `sprfkey` (no field when
   `None` — protocol allows for offline/nonparticipation).
3. Extend `RegisterKey::online` to accept the state-proof key:
   ```rust
   pub fn online(
       sender: Address,
       vote_pk: VotePk,
       selection_pk: VrfPk,
       state_proof_key: StateProofPk,
       vote_first: Round,
       vote_last: Round,
       vote_key_dilution: u64,
   ) -> Self;
   ```
4. Provide a small helper to base64-decode the algorand-sdk-testing
   fixture keys for tests.

## Consequences

- The `@send.keyregtxn` scenario unblocks fully — online / offline /
  nonparticipation all become reachable.
- `RegisterKey::online`'s signature changes (additive parameter), so
  this is a breaking change for any caller using online registration
  today. The change is small enough to justify directly given the
  protocol requires the field.
- Downstream tooling that builds participation keys for real nodes
  benefits from a typed `StateProofPk` rather than the existing
  ad-hoc plumbing.
