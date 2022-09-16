# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2022-09-16

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
