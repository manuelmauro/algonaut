# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
[0.1.1]: https://github.com/manuelmauro/algorand-rs/releases/tag/v0.1.1
