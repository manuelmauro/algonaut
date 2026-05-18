# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `algonaut_abi::sourcemap::SourceMap`: a V3 TEAL source-map decoder with `pc_to_line` / `line_to_pcs` / `last_pc_for_line` / `source_line_to_pc` accessors. `Algod::teal_compile_with_sourcemap` returns a parsed map alongside the bytes
- The `@compile.sourcemap` cucumber scenario now runs
- ADR collection at `docs/adr/` managed by [`arkouda`](https://github.com/manuelmauro/arkouda); seeded with `cucumber-test-suite-coverage-strategy`, `simulaterequest-model-needs-power-pack-fields`, `atomictransactioncomposer-simulate-convenience`, `teal-source-map-decoder`, `dryrun-request-builder`, and `cucumber-unit-test-scaffolding`
- `tests/features_runner.rs` now enumerates every `.feature` file in the algorand-sdk-testing suite (13 integration + 17 unit), gating un-implemented features behind explicit ADR references
- Drafted upstream tickets in `docs/cucumber-pending-issues.md` ready for `gh issue create`
- `algonaut_core::StateProofPk` (64-byte BLS public key) plus `algonaut_encoding::U8_64Visitor`
- `state_proof_key: Option<StateProofPk>` field on `algonaut_transaction::transaction::KeyRegistration`, serialised as the `sprfkey` msgpack field
- `@send.keyregtxn` scenarios (online, offline, nonparticipation) are now live in the cucumber runner

### Changed

- The runner's "v1 only" comment for `algod`/`assets` is replaced with an accurate matrix — both features actually target v2 endpoints and only need step-def coverage
- **Breaking:** `algonaut_transaction::RegisterKey::online` now takes a `StateProofPk` argument between `selection_pk` and `vote_first`. v34 consensus rejects online registrations without it

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
