# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Add Github CI actions
- Add tests for `kmd` client
- Add more integration tests

### Changed

- Refactor `kmd` client

## [0.1.1] - 2021-02-19

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

### Removed

- Remove cucumber test suite

[unreleased]: https://github.com/manuelmauro/algorand-rs/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/manuelmauro/algorand-rs/releases/tag/v0.1.1
