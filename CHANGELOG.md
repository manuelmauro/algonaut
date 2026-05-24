# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- KMD program-signing endpoints: `Kmd::sign_program` and `Kmd::sign_program_multisig` for signing TEAL programs / LogicSigs with keys managed by kmd. New request/response types in `algonaut_model::kmd::v1`
- Convenience re-exports at the crate root: `use algonaut::{Algod, Indexer, Kmd, SourceMap, PendingSubmission}` instead of the versioned paths (`algonaut::algod::v2::Algod`, etc.). The versioned paths remain available
- Protocol domain separators constants module (`algonaut_core::domain_separator`) with named constants (`MESSAGE_PREFIX`, `TX_PREFIX`, `MULTISIG_PREFIX`, `PROGRAM_PREFIX`, `TEAL_SIGN_PREFIX`, `BIDDATA_PREFIX`) replacing magic literals
- Open `algonaut_transaction::signer::Signer` trait so third-party signers (HSMs, remote KMS, hardware wallets, etc.) can plug into the `AtomicTransactionComposer`. Built-in impls are provided for `Account`, `ContractAccount`, and a new `MultisigSigner` bundle (`address: MultisigAddress`, `accounts: Vec<Account>`). Re-exported as `algonaut::transaction::{Signer, MultisigSigner}`
- `TransactionWithSigner::new(tx, signer)` and `TransactionWithSigner::unsigned(tx)` convenience constructors
- Compile-time checked ABI method calls modeled on `format!`: `abi_call!("add(uint64,uint64)uint64", 2u64, 3u64)` treats the ARC-4 signature literal as a format string whose argument types are the specifiers — validating the signature against the canonical grammar, checking argument arity, and type-checking each argument via a per-type `AbiArg<T>` bound, all at compile time with `format!`-quality spans. `abi_method!("…")` is the signature-only base. Two new crates back this: `algonaut_abi_sig` (the pure signature/type grammar, shared so the macros and the runtime `from_signature` cannot disagree) and the `algonaut_abi_macros` proc-macro crate (re-exported as `algonaut_abi::abi_call!` / `algonaut::abi::abi_call!`). The `AbiArg<T>` trait, marker types, and `MethodInvocation` live in `algonaut_abi::macro_support`. The first cut checks value arguments with a canonical Rust representation (`uintN`, `byte`, `bool`, `address`, `string`, `byte[]`); transaction/reference/`ufixed` arguments use the dynamic path. See `docs/adr/abi-method-signature-macro.md`

### Changed

- **Breaking:** the `algod`, `indexer`, and `kmd` client modules are now gated behind Cargo features of the same name (`algod`, `indexer`, `kmd`). With `default-features = false`, none of the clients are compiled; opt in explicitly. The default TLS backend is now `rustls` for all clients (`native-tls` available via feature). See `docs/adr/client-feature-gates.md`
- **Breaking:** structured error variants replace generic string errors across the leaf crates. `CoreError::General(String)` is replaced with `Base64Decode`, `InvalidArraySize`, `InvalidTransactionType`. `AbiError::Msg(String)` is replaced with `TypeParse`, `Encode`, `Decode`, `MethodSignature`, `ValueOutOfRange`. `AlgodError`/`IndexerError` now expose source-chaining variants (`Reqwest`, `Decode`, `Msgpack`, `Io`, `ResponseError`) so callers can inspect underlying causes via `source()`. See `docs/adr/structured-leaf-errors.md`
- **Breaking:** the `algonaut::atomic_transaction_composer` module is renamed to `algonaut::atomic`. Update imports: `use algonaut::atomic_transaction_composer::{…}` → `use algonaut::atomic::{…}`. No type names, signatures, or behaviour change — the directory was named after the `AtomicTransactionComposer` type the typestate refactor deleted. The module is also reorganized internally into `group`, `method_call`, `encode`, `outcome`, and `signing` submodules (private; the flat public re-exports are unchanged). See `docs/adr/atomic-module-layout.md`
- **Breaking:** the closed `algonaut::atomic_transaction_composer::transaction_signer::TransactionSigner` enum is replaced by the open `Signer` trait. `TransactionWithSigner.signer` is now `Option<Arc<dyn Signer>>` (`None` mirrors the old `TransactionSigner::Empty` simulate slot — the composer fills it with an all-zero placeholder signature). `AddMethodCallParams.signer` is now `Arc<dyn Signer>`. Migrate constructions: `TransactionSigner::BasicAccount(acc)` → `Arc::new(acc) as Arc<dyn Signer>`; `TransactionSigner::ContractAccount(ca)` → `Arc::new(ca)`; `TransactionSigner::MultisigAccount { address, accounts }` → `Arc::new(MultisigSigner { address, accounts })`; `TransactionSigner::Empty` → drop, use `TransactionWithSigner::unsigned(tx)` or `signer: None`
- **Breaking:** the `MethodCall` builder takes the method and its arguments together via `.invoke(...)` instead of the separate `.args(...)` setter, and `MethodCall::builder` no longer takes the method positionally. Migrate `MethodCall::builder(app_id, method, sender, signer).args([2u64, 3u64])` → `MethodCall::builder(app_id, sender, signer).invoke(abi_call!("add(uint64,uint64)uint64", 2u64, 3u64))` for compile-time-literal signatures, or `.invoke(Invocation::new(method, [2u64, 3u64]))` when the method comes from `AbiMethod::from_signature` at run time. See `docs/adr/abi-method-signature-macro.md`
- **Breaking:** transaction naming is normalized across the handwritten API — the `Transaction` noun is dropped where the receiver or argument type already carries it, and otherwise spelled out in full (never `tx` / `txn` / `txid` in a public name). Type renames: `TxId` → `TransactionId`, `TxGroup` → `TransactionGroup` (module `algonaut_transaction::tx_group` → `group`), `TxnHeader` → `TransactionHeader`. `Algod` methods: `send_txn` → `send`, `send_txns` → `send_transactions`, `send_txn_async` → `send_async`, `send_txns_async` → `send_transactions_async`, `send_raw_txn` → `send_raw`, `send_raw_txn_async` → `send_raw_async`, `pending_txn` → `pending_transaction`, `pending_txns` → `pending_transactions`, `address_pending_txns` → `address_pending_transactions`, `block_txids` → `block_transaction_ids`, `txn_proof` → `transaction_proof`, `txn_group_state_delta` → `transaction_group_state_delta`, `txn_group_state_deltas_for_round` → `transaction_group_state_deltas_for_round`, `submit_txns` → `submit_transactions`, `simulate_txns` → `simulate`, `txn_params` → `suggested_params`. `Account::sign_transaction` and `Kmd::sign_transaction` → `sign`. The transaction-id concept is spelled `transaction_id` everywhere — `txid` / `tx_id` parameters and fields retire, including `PendingSubmission::tx_id()` → `transaction_id()` and the `tx_id` / `tx_ids` fields on the atomic `ExecuteOutcome` / `SimulateOutcome` / `AbiMethodResult`. Amends the `TxId` / `TxGroup` names introduced by the identifier-newtypes ADR. Rename-only: no behaviour or wire-format change. See `docs/adr/transaction-naming-convention.md`
- **Breaking:** the generated OpenAPI **models** move out of the client crates into `algonaut_model`, under per-spec submodules `algonaut_model::algod` and `algonaut_model::indexer` (re-exported as `algonaut::model::algod` / `algonaut::model::indexer`). `algonaut_algod` / `algonaut_indexer` keep only their `apis` (operation functions, `Configuration`, transport) and depend on `algonaut_model` for the model types. The hand-written wire models in `algonaut_algod::ext` move to `algonaut_model::algod::ext`. Migrate type paths: `algonaut_algod::models::X` → `algonaut_model::algod::X` (or `algonaut::model::algod::X`); `algonaut_indexer::models::X` → `algonaut_model::indexer::X`; `algonaut_algod::ext::block::BlockResponse` → `algonaut_model::algod::ext::block::BlockResponse`. Type *shapes* are unchanged. See `docs/adr/relocate-generated-models.md`
- **Breaking:** the `algonaut::openapi_algod`, `algonaut::openapi_indexer`, and `algonaut::openapi_kmd` re-exports are removed. Reach models through `algonaut::model::{algod,indexer}` and raw operations through `algonaut::algod::api` / `algonaut::indexer::api` (re-exported from the client crates, so no extra `Cargo.toml` dependency is needed). No `openapi_*` path appears in user code or rustdoc anymore
- **Breaking:** the 39 synthesized `<op>200Response`/`<op>400Response` response envelopes are renamed to intentional names in `algonaut_model::{algod,indexer}`. Headline algod renames: `RawTransaction200Response` → `SubmitResponse`, `GetBlockHash200Response` → `BlockHash`, `TealCompile200Response` → `CompiledTeal`, `TransactionParams200Response` → `SuggestedParams`, `GetStatus200Response` → `NodeStatus`, `GetSupply200Response` → `Supply`, `SimulateTransaction200Response` → `SimulateTransactionResponse`. Indexer list responses follow a `<Subject>Response` scheme (e.g. `LookupAccountById200Response` → `AccountResponse`, `SearchForAccounts400Response` → `ErrorResponse`). The full rename table is the `inlineSchemaNameMappings` block in `openapi/config-{algod,indexer}.yaml`
- **Breaking:** the hand-written response wrappers in `algonaut_model::client_types` (`SuggestedParams`, `NodeStatus`, `Supply`) are retired — the renamed, domain-typed generated models in `algonaut_model::algod` reproduce them, so `Algod::suggested_params` / `status` / `supply` (and `status_after_block`) return those directly. `SuggestedParams` keeps its domain types (`last_round: Round`, `min_fee`/`fee`: `MicroAlgos`) but the field formerly named `fee_per_byte` is now `fee`; `NodeStatus` is now algod's full node-status struct (a superset of the old curated wrapper). The `TransactionParams` trait stays in `algonaut_model::client_types` (still re-exported from `algonaut_transaction::builder`). This fully supersedes the `hide-generated-types` ADR
- **Breaking:** `SimulateRequestTransactionGroup::txns` is now `Vec<algonaut_model::transaction::ApiSignedTransaction>` (was `Vec<algonaut_transaction::SignedTransaction>`), so `algonaut_model` need not depend on `algonaut_transaction`. Convert with `ApiSignedTransaction::try_from(signed_txn)?` when building a group by hand; the `AtomicTransactionComposer` simulate path does this for you
- **Breaking:** `algonaut_model::algod::DryrunSource.source` is now `String` (the TEAL source *text* algod's dryrun API expects per its OpenAPI spec) instead of `algonaut_encoding::Bytes`. Sending it base64-encoded made algod treat the source as program bytes and fail with `unknown opcode`. Accordingly `DryrunRequestBuilder::add_text_source` now takes `impl Into<String>` (was `impl Into<Vec<u8>>`), and the broken `add_compiled_source` is removed — it put compiled bytes into the text `source` field, which never worked; dryrun an already-compiled program via the transaction's LogicSig or the application's approval/clear programs instead

### Fixed

- Optional address fields in `AssetParams` (`clawback`, `freeze`, `manager`, `reserve`) now correctly deserialize the Algorand zero address (`AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA`) as `None` instead of `Some(Address::default())`. The indexer returns the zero address for unset fields; the new `deserialize_optional_zero_address` helper in `algonaut_encoding` handles this (#142)
- ABI method-call return values now decode. `AtomicTransactionComposer::execute` (and the simulate return-value path) base64-decoded the confirmed transaction's last log a second time, but `PendingTransactionResponse.logs` already holds decoded bytes (`algonaut_encoding::Bytes`), so any method returning a value failed with `Base64DecodeError`. The redundant decode is removed
- `Algod::teal_dryrun` no longer fails to decode algod's response. algod returns `null` (not `[]`) for `DryrunResponse.txns` when the dryrun reports a top-level error — and for some nested arrays (`DryrunTxnResult.disassembly`, `DryrunState.stack`) — which a plain `Vec<T>` field could not deserialize. These now decode `null`/missing as empty via the new `algonaut_encoding::deserialize_null_default` helper

## [0.7.0] - 2026-05-20

### Added

- `MicroAlgos::checked_add`, `MicroAlgos::checked_sub`, and `MicroAlgos::checked_mul` — overflow- and underflow-safe arithmetic returning `Option<MicroAlgos>`, so callers no longer need to reach into the inner `u64` (#152)
- Block-introspection endpoints on `Algod`: `block_txids` (`GET /v2/blocks/{round}/txids`), `block_logs` (`GET /v2/blocks/{round}/logs`), `block_timestamp_offset` (`GET /v2/devmode/blocks/offset`), and `set_block_timestamp_offset` (`POST /v2/devmode/blocks/offset/{offset}`). New `algonaut_algod` models `GetBlockTxids200Response`, `GetBlockLogs200Response`, `AppCallLogs`, and `GetBlockTimeStampOffset200Response`
- Account-resource pagination endpoints on `Algod`: `account_apps` (`GET /v2/accounts/{address}/applications`) and `account_assets` (`GET /v2/accounts/{address}/assets`), each accepting `limit`/`next` pagination arguments. New `algonaut_algod` models `AccountApplicationsInformation200Response`, `AccountAssetsInformation200Response`, `AccountApplicationResource`, and `AccountAssetHolding`
- Ledger state-delta endpoints on `Algod`: `txn_group_state_delta` (`GET /v2/deltas/txn/group/{id}`) and `txn_group_state_deltas_for_round` (`GET /v2/deltas/{round}/txn/group`). New `algonaut_algod` models `GetTransactionGroupLedgerStateDeltasForRound200Response` and `LedgerStateDeltaForTransactionGroup`
- Node-administration endpoints on `Algod`: `generate_participation_keys` (`POST /v2/participation/generate/{address}`), `config` (`GET /debug/settings/config`), `debug_settings_prof` (`GET /debug/settings/pprof`), and `set_debug_settings_prof` (`PUT /debug/settings/pprof`). New `algonaut_algod` model `DebugSettingsProf`
- Asynchronous transaction-broadcast endpoint on `Algod`: `send_raw_txn_async`, `send_txn_async`, and `send_txns_async` (`POST /v2/transactions/async`), which submit transactions to the network without ahead-of-time checks and without waiting for a transaction ID
- `Indexer::search_for_block_headers` exposing the indexer's `GET /v2/block-headers` endpoint, which returns block headers in ascending round order. Backed by the new `algonaut_indexer::apis::search_api::search_for_block_headers` operation and `SearchForBlockHeaders200Response` model. Adds heartbeat-transaction support — new `TransactionHeartbeat` and `HbProofFields` models, a `Transaction::heartbeat_transaction` field, and a `TxType::Hb` variant — plus access-list / resource-reference support on `TransactionApplication` (`access`, `box-references`, `reject-version` fields) backed by the new `ResourceRef`, `BoxReference`, `HoldingRef`, and `LocalsRef` models
- `v2algodclient_paths.feature` (38 example rows) and `v2indexerclient_paths.feature` (109 example rows) cucumber unit tests are now live. They exercise the request paths the algod / indexer v2 clients emit against a new in-process recording mock HTTP server (`tests/step_defs/unit/mock_server.rs`), with no Algorand harness required. The path assertion compares the query string as an unordered set of parameters (RFC 3986), so a difference in query-parameter ordering is not a failure. The `features_runner` matrix gains an `excluded_scenarios` field for name-qualified per-scenario exclusion; the only remaining exclusions are documented capability gaps in the generated `algonaut_algod` / `algonaut_indexer` operations — a missing `header-only` (`get_block`), `online-only` (`search_for_accounts`), or `group-id` (`search_for_transactions`) query parameter
- `v2algodclient_responses.feature` and `v2indexerclient_responses.feature` cucumber unit tests are now live. They serve canned HTTP response bodies (the upstream fixtures under `tests/features/resources/`) from a new in-process `ResponseMockServer` (`tests/step_defs/unit/mock_server.rs`), drive a high-level `Algod` / `Indexer` call, and assert on the parsed response — no Algorand harness required. The new step-def module is `tests/step_defs/unit/v2_client_responses.rs`. `v2algodclient_responses.feature` is fully live (14/14 scenarios, none excluded)
- msgpack response-body support in `algonaut_algod`. The generated `get_block`, `pending_transaction_information`, `get_pending_transactions`, and `get_pending_transactions_by_address` operations now content-negotiate the response decoder: a body served with `Content-Type: application/msgpack` is decoded with `rmp_serde`, otherwise JSON is used as before (`algonaut_algod::apis::decode_response_body`). `Error` gains a `Msgpack` variant. The hand-written `ext::block` / `ext::transaction` models gain a `WireBytes` byte type and lenient string helpers so they decode from either wire format
- A heartbeat (`hb`) variant on the `algonaut_algod` `ext::transaction::Transaction` enum, mirroring the indexer's heartbeat shape — a `HeartbeatFields` struct (`address`, `key_dilution`, `proof`, `seed`, `vote_id`) and a `HeartbeatProof` struct — plus `Transaction::heartbeat`, `Transaction::heartbeat_address`, and `Transaction::sender` accessors. This brings the algod `Get Block` heartbeat scenarios (JSON and msgpack) live
- `make generate-clients` / `make fetch-openapi-specs` and an `openapi/` directory (pinned algod & indexer specs + generator configs) make regenerating the OpenAPI clients reproducible and diff-able; see `docs/adr/openapi-client-regeneration.md`

### Changed

- **Breaking:** `Algod::account_apps` gains an `include: Option<&[String]>` argument, surfacing the `include` query parameter (e.g. `["params"]`) of `GET /v2/accounts/{address}/applications`, which the wrapper previously hard-coded to `None`
- **Breaking:** `Algod::pending_txns` and `Algod::address_pending_txns` each gain a `format: Option<&str>` argument, surfacing the `format` query parameter (`"json"` / `"msgpack"`) of the pending-transactions endpoints, which the wrappers previously hard-coded to `None`
- **Breaking:** `AssetParams.decimals` is now `u32` instead of `u64` in the `algonaut_algod` and `algonaut_indexer` clients, matching Algorand's documentation (the value is bounded to 0..=19). The hand-written `algonaut_transaction`/`algonaut_model` `AssetParams` types already used `u32` (#140)
- **Breaking:** introduced the `algonaut_core::{AppId, AssetId, TxId}` newtypes and adopted them across the hand-written crates for full type safety. `AppId`/`AssetId` wrap `u64` and `TxId` wraps `String`; all three serialize transparently (wire-identical to the bare value). Application and asset ids — and transaction ids — in the transaction builders (`UpdateAsset`, `DestroyAsset`, `TransferAsset`, `AcceptAsset`, `ClawbackAsset`, `FreezeAsset`, `CreateApplication`, `UpdateApplication`, `CallApplication`, `OptInApplication`, `ClearApplication`, `CloseApplication`, `DeleteApplication`), the `Transaction`/`SignedTransaction` types and `Transaction::id`, the `algonaut_model` API transaction models, the `algonaut_abi::AbiContractNetworkInfo` type, and the `AtomicTransactionComposer` (`AddMethodCallParams::app_id`, `AbiMethodResult::tx_id`) now use these newtypes instead of bare `u64`/`String`. Wrap literals with `AppId(..)`/`AssetId(..)`/`TxId(..)` or use `.into()`; unwrap at generated-crate boundaries with `id.0` / `u64::from(id)` / `txid.as_str()` (#160)
- **Breaking:** `algonaut_algod`'s `PendingTransactionResponse.txn` and `GetPendingTransactionsByAddress200Response.top_transactions` are now `ext::transaction::TransactionHeader` instead of `serde_json::Value` — the transaction is typed, and decodes from msgpack responses (`serde_json::Value` cannot hold msgpack `bin`)

### Fixed

- `algonaut_indexer` no longer fails to deserialize valid responses that omit a field the generated models marked mandatory. `Account`'s `total-*` count fields and the `genesis-hash` / `metadata-hash` / `transactions-root-sha256` fields on `Transaction`, `Block`, and `AssetParams` gained `#[serde(default)]`, so an absent value decodes (counts to `0`, optional hashes to `None`) instead of erroring. This brings `v2indexerclient_responses.feature` fully live (13/13 scenarios, none excluded)

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
