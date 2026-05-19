# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `MicroAlgos::checked_add`, `MicroAlgos::checked_sub`, and `MicroAlgos::checked_mul` — overflow- and underflow-safe arithmetic returning `Option<MicroAlgos>`, so callers no longer need to reach into the inner `u64` (#152)
- Block-introspection endpoints on `Algod`: `block_txids` (`GET /v2/blocks/{round}/txids`), `block_logs` (`GET /v2/blocks/{round}/logs`), `block_timestamp_offset` (`GET /v2/devmode/blocks/offset`), and `set_block_timestamp_offset` (`POST /v2/devmode/blocks/offset/{offset}`). New `algonaut_algod` models `GetBlockTxids200Response`, `GetBlockLogs200Response`, `AppCallLogs`, and `GetBlockTimeStampOffset200Response`
- Account-resource pagination endpoints on `Algod`: `account_apps` (`GET /v2/accounts/{address}/applications`) and `account_assets` (`GET /v2/accounts/{address}/assets`), each accepting `limit`/`next` pagination arguments. New `algonaut_algod` models `AccountApplicationsInformation200Response`, `AccountAssetsInformation200Response`, `AccountApplicationResource`, and `AccountAssetHolding`
- Ledger state-delta endpoints on `Algod`: `txn_group_state_delta` (`GET /v2/deltas/txn/group/{id}`) and `txn_group_state_deltas_for_round` (`GET /v2/deltas/{round}/txn/group`). New `algonaut_algod` models `GetTransactionGroupLedgerStateDeltasForRound200Response` and `LedgerStateDeltaForTransactionGroup`
- Node-administration endpoints on `Algod`: `generate_participation_keys` (`POST /v2/participation/generate/{address}`), `config` (`GET /debug/settings/config`), `debug_settings_prof` (`GET /debug/settings/pprof`), and `set_debug_settings_prof` (`PUT /debug/settings/pprof`). New `algonaut_algod` model `DebugSettingsProf`

### Changed

- **Breaking:** `AssetParams.decimals` is now `u32` instead of `u64` in the `algonaut_algod` and `algonaut_indexer` clients, matching Algorand's documentation (the value is bounded to 0..=19). The hand-written `algonaut_transaction`/`algonaut_model` `AssetParams` types already used `u32` (#140)

## [0.6.0] - 2026-05-18

### Added

- `algonaut::dryrun::DryrunRequestBuilder` for assembling a `DryrunRequest` from signed transactions + compiled or source-text TEAL programs, plus `algonaut::dryrun::result::{first_status, app_call_status, logic_sig_status, overall_status}` helpers
- `dryrun.feature` and `dryrun_testing.feature` cucumber scenarios are now live
- `tests/step_defs/unit/` scaffolding: `UnitWorld` plus a separate `cucumber()` builder so the 17 unit features can run without a live harness. First unit feature (`offline.feature`, address/mnemonic/microalgos round-trip scenarios) is now live; the remaining unit scenarios are unblocked and can land as mechanical follow-ups
- `algonaut_abi::sourcemap::SourceMap`: a V3 TEAL source-map decoder with `pc_to_line` / `line_to_pcs` / `last_pc_for_line` / `source_line_to_pc` accessors. `Algod::teal_compile_with_sourcemap` returns a parsed map alongside the bytes
- The `@compile.sourcemap` cucumber scenario now runs
- Simulate-endpoint power-pack fields on `algonaut_algod::models::SimulateRequest`: `allow_more_logging`, `allow_empty_signatures`, `allow_unnamed_resources`, `extra_opcode_budget`, `exec_trace_config`, `round`, `fix_signers`. New types `SimulateTraceConfig`, `SimulateEvalOverrides`, `AvmValue`, `ScratchChange`, `ApplicationStateOperation`, `SimulationOpcodeTraceUnit`, `SimulationTransactionExecTrace`. Response models (`SimulateTransactionResult`, `SimulateTransaction200Response`) gain matching `exec-trace`, `eval-overrides`, `fixed-signer`, `initial-states` fields
- `algonaut::simulate::SimulateRequestBuilder` plus `SimulateTraceConfigBuilder` — fluent setters that fold the nested `Option<Box<...>>` fields into a chainable API
- `algonaut::atomic_transaction_composer::AtomicTransactionComposer::{simulate, simulate_with}` for running the composed group against algod's `/v2/transactions/simulate` endpoint. New `AtcSimulateResult` carries the typed `SimulateTransaction200Response` alongside the parsed ABI returns
- `AtomicTransactionComposerStatus::Simulated` variant (placed between `Signed` and `Submitted` so subsequent `submit()`/`execute()` calls still work after a simulate)
- `TransactionSigner::Empty` variant — emits the all-zero placeholder signature so the simulator can report unsigned transactions as `signedtxn has no sig`
- `simulate.feature` is wired up: payment + ATC group simulate scenarios, power-pack toggles, and trace assertions. Two scenarios are gated (`simulate.exec_trace_with_stack_scratch`, `simulate.exec_trace_with_state_change_and_hash`) pending an upstream fix and the `create-and-optin` on-complete combo on `CreateApplication` (#273)
- Format-aware `Serialize`/`Deserialize` impls on `Address`, `VotePk`, `VrfPk`, `StateProofPk`, `Signature`, `HashDigest`, and `Ed25519PublicKey` — JSON now renders canonical base32/base64 strings while msgpack stays byte-identical. Unblocks the indexer client and other JSON-only endpoints (#272, closes #271)
- ADR collection at `docs/adr/` managed by [`arkouda`](https://github.com/manuelmauro/arkouda); seeded with `cucumber-test-suite-coverage-strategy`, `simulaterequest-model-needs-power-pack-fields`, `atomictransactioncomposer-simulate-convenience`, `teal-source-map-decoder`, `dryrun-request-builder`, `cucumber-unit-test-scaffolding`, and `domain-types-serialize-for-both-json-and-msgpack`
- `tests/features_runner.rs` now enumerates every `.feature` file in the algorand-sdk-testing suite (13 integration + 17 unit), gating un-implemented features behind explicit ADR references
- Drafted upstream tickets in `docs/cucumber-pending-issues.md` ready for `gh issue create`
- `algonaut_core::StateProofPk` (64-byte BLS public key) plus `algonaut_encoding::U8_64Visitor`
- `state_proof_key: Option<StateProofPk>` field on `algonaut_transaction::transaction::KeyRegistration`, serialised as the `sprfkey` msgpack field
- `@send.keyregtxn` scenarios (online, offline, nonparticipation) are now live in the cucumber runner

### Changed

- **Breaking:** `SimulateRequestTransactionGroup.txns` switches from `Vec<String>` (the OpenAPI placeholder) to `Vec<SignedTransaction>` — algod expects nested `SignedTxn` objects inline (#273)
- **Breaking:** `algonaut_transaction::RegisterKey::online` now takes a `StateProofPk` argument between `selection_pk` and `vote_first`. v34 consensus rejects online registrations without it
- `SimulateTransaction200Response.would_succeed` is now `#[serde(default)]`; current sandbox builds omit it on early-error responses
- `AtomicTransactionComposer::simulate_with` no longer base64-encodes the signed group — it passes the `SignedTransaction`s straight to the request
- The dryrun msgpack workaround introduced as a stopgap is removed; the generated JSON client serialises the body correctly now that domain types branch on `is_human_readable()`
- The runner's "v1 only" comment for `algod`/`assets` is replaced with an accurate matrix — both features actually target v2 endpoints and only need step-def coverage

## [0.5.0] - 2026-05-18

### Added

- Hoist every external dependency version into the root `[workspace.dependencies]` table; subcrates inherit via `{ workspace = true }` (#244)
- Add `lefthook.yml` running `make ci` on pre-commit and enforcing Conventional Commits on commit-msg (#242)
- Expand the `Makefile` with `setup`, `fmt[-check]`, `clippy`, `check[-release]`, `check-wasm`, `build[-release]`, `test[-release]`, `ci`, `doc`, and `help` targets while preserving the existing integration/harness/docker targets (#242)
- GitHub Actions runs the cucumber integration tests against an algorand sandbox harness, replacing the retired CircleCI job; promoted to a required check on `main` (#247, #248)

### Changed

- Bump the Rust edition of every workspace crate to 2024 — raises the MSRV to 1.85 (#246)
- Bump dependencies to the latest compatible versions: `thiserror` 2, `derive_more` 2, `reqwest` 0.13, `sha2` 0.11, `cucumber` 0.23, `serde_with` 3, `env_logger` 0.11, `gloo-timers` 0.4, `indexmap` 2, and various minor bumps (#242)
- Replace `ring` with `ed25519-dalek` 2 for Ed25519 sign/verify, removing the C-compiler dependency for `wasm32-unknown-unknown` builds (#245)
- `Account` internal field `key_pair: Ed25519KeyPair` → `signing_key: ed25519_dalek::SigningKey` (private)

### Fixed

- `cargo check --target wasm32-unknown-unknown` now builds on stock toolchains (no `brew install llvm` required on macOS) (#245)
- Pre-existing clippy warnings (empty doc comments, doc-list indentation, deprecated `impl ToString`, `push`-in-loop, etc.) across the workspace (#242)

### Pinned

- `rand` stays on 0.8 and `getrandom` on 0.2 because `num-bigint` 0.4.6's `RandBigInt` trait is built against rand 0.8

## [0.4.2] - 2022-10-06

- Add state proof transaction type and fields

## [0.4.1]

- Add "." to version numbers

## [0.4.0]

TODO

## [0.3.0] - 2021-07-30

### Added

- Add examples: logic sig (contract account, delegated sig, delegated
  multisig), key registration, atomic swap, asset transfer, asset opt-in, asset
  clawback, app create, app opt-in, app call, app update, app delete, app close out, app
  clear state
- Port official Java SDK's Unit tests for account, client, address, logic sig
- Add extra_pages application call parameter
- Add builders for all application call types
- Add verification functions to logic and multisig signatures
- Set fee to max(fee, min_fee) in builders (from API's suggested params)
- Add convenience to initialize transaction builders with suggested transaction params
- Support asset removal
- Support transaction URL scheme (payment prompts)
- Add convenience to parse Address strings
- Add convenience to submit signed transaction structs to algod
- Support WASM
- Support transactions groups
- Support logic signatures: contract account, delegated and multi signature
- Add abstraction layer for user interface

### Changed

- Fix deserialization of pending transactions for application calls using local state
- Fix application calls
- Improve user interface to compile TEAL
- Make genesis id optional
- Rewrite transaction builders to represent better use cases and verify mandatory fields at compile time
- Move transaction sender field to transaction types, to allow to document/name differently
- Fix asset opt-in transaction
- Fix asset transfer transaction
- Replace KMD signing with direct signing in non KMD specific tests and examples
- Fix key registration transaction
- Migrate to async API
- Improve transaction debug representation
- Improve indexer queries ergonomics
- Fix direct account signing
- Separate domain and API transaction representation
- Fix indexer queries
- Display error messages returned by clients in error

## [0.2.0] - 2021-04-23

### Added

- Support all kind of Algorand transactions
- Add transaction builders
- Add example for the creation of a new ASA asset
- Add builder for payment, key registration, and asset creation transactions
- Add `indexer` v2 API support
- Add `algod` v2 API support
- Add structs for all transaction types
- Add Github CI actions
- Add tests for `kmd` client
- Add tests for `indexer` v2 client
- Add tests for `algod` v2 client
- Add more integration tests

### Changed

- Implement `FromStr` for `Address` in place of `Address::from_str` method
- Rename project from `algorand-rs` to `algonaut`
- Refactor project in multiple crates according to [Algorand's SDK common schema](https://github.com/algorand/algorand-sdk-testing#sdk-overview)
- Refactor `kmd` client

### Removed

- `BaseTransaction` and constructors from `Transaction`

## [0.1.1] - 2021-02-19 (`algorand-rs`)

### Added

- Add some sandbox integration tests
- Use `dotenv` for address and token env variable pointing at the sandbox
- Add client builder for: algod, kmd, and indexer
- Use `thiserror` crate for error management
- Add algorand's indexer client (incomplete)
- Add algod v2 client (incomplete)
- Add `reqwest` http client to clients' structs
- Forked [rust-algorand-sdk](https://github.com/mraof/rust-algorand-sdk)

### Changed

- Change modules structure

### Removed

- Remove APIV1Request trait
- Remove (temporarily) cucumber test suite

[unreleased]: https://github.com/manuelmauro/algonaut/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/manuelmauro/algonaut/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/manuelmauro/algonaut/releases/tag/v0.1.1
