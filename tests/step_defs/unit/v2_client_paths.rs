//! Step definitions for the `v2algodclient_paths` and
//! `v2indexerclient_paths` unit features.
//!
//! These are pure path-assertion unit tests: the SDK is pointed at a local
//! recording [`MockServer`], a client call is made, and the request *path*
//! (including query string) is compared against the feature's expectation.
//! The call's `Result` is intentionally discarded — the mock answers with a
//! body the typed models will not deserialize, and that is fine here.

use crate::step_defs::unit::mock_server::MockServer;
use crate::step_defs::unit::world::UnitWorld;
use algonaut::algod::v2::Algod;
use algonaut::indexer::v2::Indexer;
use algonaut_core::{Address, AppId, AssetId, TxId};
use cucumber::{given, then, when};

/// A token (the `Configuration` requires one; its value is irrelevant to the
/// path under test).
const TOKEN: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

/// Treat empty `Scenario Outline` cells as "argument omitted".
fn opt_str(s: &str) -> Option<&str> {
    if s.is_empty() { None } else { Some(s) }
}

/// Treat `0` (and empty) as "argument omitted" for paginating / filtering
/// numeric query parameters.
fn opt_u64(n: u64) -> Option<u64> {
    if n == 0 { None } else { Some(n) }
}

/// Parse a comma-separated list cell into an owned `Vec<String>`, or `None`
/// when the cell is empty.
fn opt_csv(s: &str) -> Option<Vec<String>> {
    if s.is_empty() {
        None
    } else {
        Some(s.split(',').map(|p| p.to_string()).collect())
    }
}

/// Parse a feature-table cell into an [`Address`]. The cells are populated
/// with well-formed Algorand addresses; treat parse failure as a test-data
/// bug rather than a fall-through.
fn parse_address(s: &str) -> Address {
    s.parse()
        .unwrap_or_else(|e| panic!("invalid address `{s}`: {e}"))
}

/// Treat empty cells as "argument omitted"; otherwise parse the cell as an
/// [`Address`].
fn opt_address(s: &str) -> Option<Address> {
    if s.is_empty() {
        None
    } else {
        Some(parse_address(s))
    }
}

/// Treat `0` (and empty) as "argument omitted" for application IDs.
fn opt_app_id(n: u64) -> Option<AppId> {
    if n == 0 { None } else { Some(AppId(n)) }
}

/// Treat `0` (and empty) as "argument omitted" for asset IDs.
fn opt_asset_id(n: u64) -> Option<AssetId> {
    if n == 0 { None } else { Some(AssetId(n)) }
}

/// Treat empty cells as "argument omitted"; otherwise wrap the cell as a
/// [`TxId`].
fn opt_tx_id(s: &str) -> Option<TxId> {
    if s.is_empty() {
        None
    } else {
        Some(TxId(s.to_string()))
    }
}

// --- Background ------------------------------------------------------------

#[given(expr = "mock server recording request paths")]
async fn mock_server_recording(w: &mut UnitWorld) {
    let server = MockServer::start().await;
    w.algod = Some(Algod::new(&server.base_url, TOKEN).expect("algod client"));
    w.indexer = Some(Indexer::new(&server.base_url, TOKEN).expect("indexer client"));
    w.mock_server = Some(server);
}

fn algod(w: &UnitWorld) -> Algod {
    w.algod.clone().expect("algod client not initialised")
}

fn indexer(w: &UnitWorld) -> &Indexer {
    w.indexer.as_ref().expect("indexer client not initialised")
}

/// Build an algod-openapi [`Configuration`] pointed at the mock server, used
/// for the handful of endpoints whose extra query parameters the high-level
/// [`Algod`] wrapper does not expose.
fn algod_config(w: &UnitWorld) -> algonaut::openapi_algod::apis::configuration::Configuration {
    let base = w
        .mock_server
        .as_ref()
        .expect("mock server not started")
        .base_url
        .clone();
    algonaut::openapi_algod::apis::configuration::Configuration {
        base_path: base,
        user_agent: Some("algonaut".to_owned()),
        client: reqwest::Client::new(),
        basic_auth: None,
        oauth_access_token: None,
        bearer_access_token: None,
        api_key: Some(algonaut::openapi_algod::apis::configuration::ApiKey {
            prefix: None,
            key: TOKEN.to_owned(),
        }),
    }
}

// --- Then ------------------------------------------------------------------

/// Split a request path into its `(path, query)` halves at the first `?`.
/// A path with no `?` yields an empty query string.
fn split_path(p: &str) -> (&str, &str) {
    match p.split_once('?') {
        Some((path, query)) => (path, query),
        None => (p, ""),
    }
}

/// Parse a query string into its sorted list of `&`-separated `key=value`
/// pairs. The pairs are compared raw (still percent-encoded) — both the SDK
/// and the upstream fixtures encode the same way, so the only difference is
/// ordering, which RFC 3986 leaves semantically insignificant.
fn sorted_query_pairs(query: &str) -> Vec<&str> {
    if query.is_empty() {
        return Vec::new();
    }
    let mut pairs: Vec<&str> = query.split('&').collect();
    pairs.sort_unstable();
    pairs
}

/// Assert that `actual` and `expected` refer to the same request path,
/// treating the query string as an unordered set of parameters.
fn assert_path_eq(actual: &str, expected: &str) {
    let (actual_path, actual_query) = split_path(actual);
    let (expected_path, expected_query) = split_path(expected);
    assert_eq!(
        actual_path, expected_path,
        "request path mismatch: got `{actual}`, expected `{expected}`"
    );
    assert_eq!(
        sorted_query_pairs(actual_query),
        sorted_query_pairs(expected_query),
        "request query-parameter mismatch (order-insensitive): \
         got `{actual}`, expected `{expected}`"
    );
}

#[then(regex = r#"^expect the path used to be "([^"]*)"$"#)]
async fn expect_path(w: &mut UnitWorld, expected: String) {
    let req = w
        .mock_server
        .as_ref()
        .expect("mock server not started")
        .last_request()
        .await;
    assert_path_eq(&req.path, &expected);
}

#[then(regex = r#"^expect the request to be "([^"]*)" "([^"]*)"$"#)]
async fn expect_request(w: &mut UnitWorld, method: String, expected: String) {
    let req = w
        .mock_server
        .as_ref()
        .expect("mock server not started")
        .last_request()
        .await;
    assert_eq!(
        req.method.to_lowercase(),
        method.to_lowercase(),
        "request method mismatch"
    );
    assert_path_eq(&req.path, &expected);
}

// === Algod v2 paths ========================================================

#[when(
    regex = r#"^we make a Pending Transaction Information against txid "([^"]*)" with format "([^"]*)"$"#
)]
async fn algod_pending_txn_information(w: &mut UnitWorld, txid: String, format: String) {
    // `Algod::pending_txn` hard-codes `format=None`; call the generated
    // endpoint directly so the `?format=` query parameter is emitted.
    let conf = algod_config(w);
    let _ = algonaut::openapi_algod::apis::public_api::pending_transaction_information(
        &conf,
        &txid,
        opt_str(&format),
    )
    .await;
}

#[when(
    regex = r#"^we make a Pending Transaction Information with max (\d+) and format "([^"]*)"$"#
)]
async fn algod_pending_txn_information2(w: &mut UnitWorld, max: u64, format: String) {
    let _ = algod(w).pending_txns(opt_u64(max), opt_str(&format)).await;
}

#[when(
    regex = r#"^we make a Pending Transactions By Address call against account "([^"]*)" and max (\d+) and format "([^"]*)"$"#
)]
async fn algod_pending_txns_by_address(
    w: &mut UnitWorld,
    account: String,
    max: u64,
    format: String,
) {
    let _ = algod(w)
        .address_pending_txns(&parse_address(&account), opt_u64(max), opt_str(&format))
        .await;
}

#[when(regex = r"^we make a Status after Block call with round (\d+)$")]
async fn algod_status_after_block(w: &mut UnitWorld, round: u64) {
    let _ = algod(w).status_after_block(round).await;
}

#[when(regex = r#"^we make an Account Information call against account "([^"]*)"$"#)]
async fn algod_account_information(w: &mut UnitWorld, account: String) {
    let _ = algod(w).account(&parse_address(&account)).await;
}

#[when(regex = r#"^we make a Get Block call against block number (\d+) with format "([^"]*)"$"#)]
async fn algod_get_block(w: &mut UnitWorld, round: u64, format: String) {
    // `Algod::block` hard-codes `format=None`; call the generated endpoint
    // directly so the `?format=` query parameter under test is emitted.
    let conf = algod_config(w);
    let _ =
        algonaut::openapi_algod::apis::public_api::get_block(&conf, round, opt_str(&format)).await;
}

#[when(regex = r#"^we make a GetAssetByID call for assetID (\d+)$"#)]
async fn algod_get_asset_by_id(w: &mut UnitWorld, asset_id: u64) {
    let _ = algod(w).asset(AssetId(asset_id)).await;
}

#[when(regex = r#"^we make a GetApplicationByID call for applicationID (\d+)$"#)]
async fn algod_get_application_by_id(w: &mut UnitWorld, application_id: u64) {
    let _ = algod(w).app(AppId(application_id)).await;
}

#[when(
    regex = r#"^we make a GetApplicationBoxByName call for applicationID (\d+) with encoded box name "([^"]*)"$"#
)]
async fn algod_get_application_box_by_name(w: &mut UnitWorld, application_id: u64, name: String) {
    let _ = algod(w).app_box(AppId(application_id), &name).await;
}

#[when(regex = r#"^we make a GetApplicationBoxes call for applicationID (\d+) with max (\d+)$"#)]
async fn algod_get_application_boxes(w: &mut UnitWorld, application_id: u64, max: u64) {
    let _ = algod(w)
        .app_boxes(AppId(application_id), opt_u64(max))
        .await;
}

#[when(
    regex = r#"^we make an Account Information call against account "([^"]*)" with exclude "([^"]*)"$"#
)]
async fn algod_account_information_exclude(w: &mut UnitWorld, account: String, exclude: String) {
    // `Algod::account` hard-codes `exclude=None`; call the generated endpoint
    // directly to exercise the `?exclude=` query parameter.
    let conf = algod_config(w);
    let _ = algonaut::openapi_algod::apis::public_api::account_information(
        &conf,
        &account,
        None,
        opt_str(&exclude),
    )
    .await;
}

#[when(
    regex = r#"^we make an Account Asset Information call against account "([^"]*)" assetID (\d+)$"#
)]
async fn algod_account_asset_information(w: &mut UnitWorld, account: String, asset_id: u64) {
    let conf = algod_config(w);
    let _ = algonaut::openapi_algod::apis::public_api::account_asset_information(
        &conf, &account, asset_id, None,
    )
    .await;
}

#[when(
    regex = r#"^we make an Account Application Information call against account "([^"]*)" applicationID (\d+)$"#
)]
async fn algod_account_application_information(
    w: &mut UnitWorld,
    account: String,
    application_id: u64,
) {
    let _ = algod(w)
        .account_app(&parse_address(&account), AppId(application_id))
        .await;
}

#[when(
    regex = r#"^we make a GetTransactionProof call for round (\d+) txid "([^"]*)" and hashtype "([^"]*)"$"#
)]
async fn algod_get_transaction_proof(
    w: &mut UnitWorld,
    round: u64,
    txid: String,
    hashtype: String,
) {
    let conf = algod_config(w);
    let _ = algonaut::openapi_algod::apis::public_api::get_transaction_proof(
        &conf,
        round,
        &txid,
        opt_str(&hashtype),
        Some("msgpack"),
    )
    .await;
}

#[when(regex = r"^we make a GetLightBlockHeaderProof call for round (\d+)$")]
async fn algod_light_block_header_proof(w: &mut UnitWorld, round: u64) {
    let _ = algod(w).light_block_header_proof(round).await;
}

#[when(regex = r"^we make a GetStateProof call for round (\d+)$")]
async fn algod_state_proof(w: &mut UnitWorld, round: u64) {
    let _ = algod(w).state_proof(round).await;
}

#[when(regex = r"^we make a Lookup Block Hash call against round (\d+)$")]
async fn algod_lookup_block_hash(w: &mut UnitWorld, round: u64) {
    let _ = algod(w).block_hash(round).await;
}

#[when(regex = r"^we make a GetLedgerStateDelta call against round (\d+)$")]
async fn algod_get_ledger_state_delta(w: &mut UnitWorld, round: u64) {
    // `Algod::state_delta` hard-codes `format=None`; call the generated
    // endpoint directly so the `?format=msgpack` query parameter is emitted.
    let conf = algod_config(w);
    let _ = algonaut::openapi_algod::apis::public_api::get_ledger_state_delta(
        &conf,
        round,
        Some("msgpack"),
    )
    .await;
}

#[when(regex = r"^we make a SetSyncRound call against round (\d+)$")]
async fn algod_set_sync_round(w: &mut UnitWorld, round: u64) {
    let _ = algod(w).sync(round).await;
}

#[when(regex = r"^we make a GetSyncRound call$")]
async fn algod_get_sync_round(w: &mut UnitWorld) {
    let _ = algod(w).sync_round().await;
}

#[when(regex = r"^we make a UnsetSyncRound call$")]
async fn algod_unset_sync_round(w: &mut UnitWorld) {
    let _ = algod(w).unsync().await;
}

#[when(regex = r"^we make a SetBlockTimeStampOffset call against offset (\d+)$")]
async fn algod_set_block_timestamp_offset(w: &mut UnitWorld, offset: u64) {
    let _ = algod(w).set_block_timestamp_offset(offset).await;
}

#[when(regex = r"^we make a GetBlockTimeStampOffset call$")]
async fn algod_get_block_timestamp_offset(w: &mut UnitWorld) {
    let _ = algod(w).block_timestamp_offset().await;
}

#[when(regex = r#"^we make a LedgerStateDeltaForTransactionGroupResponse call for ID "([^"]*)"$"#)]
async fn algod_txn_group_state_delta(w: &mut UnitWorld, id: String) {
    // `Algod::txn_group_state_delta` hard-codes `format=None`; call the
    // generated endpoint directly so `?format=msgpack` is emitted.
    let conf = algod_config(w);
    let _ =
        algonaut::openapi_algod::apis::public_api::get_ledger_state_delta_for_transaction_group(
            &conf,
            &id,
            Some("msgpack"),
        )
        .await;
}

#[when(
    regex = r"^we make a TransactionGroupLedgerStateDeltaForRoundResponse call for round (\d+)$"
)]
async fn algod_txn_group_state_deltas_for_round(w: &mut UnitWorld, round: u64) {
    // `Algod::txn_group_state_deltas_for_round` hard-codes `format=None`;
    // call the generated endpoint directly so `?format=msgpack` is emitted.
    let conf = algod_config(w);
    let _ =
        algonaut::openapi_algod::apis::public_api::get_transaction_group_ledger_state_deltas_for_round(
            &conf,
            round,
            Some("msgpack"),
        )
        .await;
}

#[when(
    regex = r#"^we make an Account Assets Information call against account "([^"]*)" with limit (\d+) and next "([^"]*)"$"#
)]
async fn algod_account_assets_information(
    w: &mut UnitWorld,
    account: String,
    limit: u64,
    next: String,
) {
    let _ = algod(w)
        .account_assets(&parse_address(&account), opt_u64(limit), opt_str(&next))
        .await;
}

#[when(
    regex = r#"^we make an Account Applications Information call against account "([^"]*)" with limit (\d+) next "([^"]*)" and include "([^"]*)"$"#
)]
async fn algod_account_applications_information(
    w: &mut UnitWorld,
    account: String,
    limit: u64,
    next: String,
    include: String,
) {
    let _ = algod(w)
        .account_apps(
            &parse_address(&account),
            opt_u64(limit),
            opt_str(&next),
            opt_csv(&include).as_deref(),
        )
        .await;
}

#[when(regex = r"^we make a GetBlockTxids call against block number (\d+)$")]
async fn algod_get_block_txids(w: &mut UnitWorld, round: u64) {
    let _ = algod(w).block_txids(round).await;
}

// === Indexer v2 paths ======================================================

#[when(
    regex = r#"^we make a Lookup Asset Balances call against asset index (\d+) with limit (\d+) afterAddress "([^"]*)" currencyGreaterThan (\d+) currencyLessThan (\d+)$"#
)]
async fn indexer_lookup_asset_balances(
    w: &mut UnitWorld,
    index: u64,
    limit: u64,
    next: String,
    cgt: u64,
    clt: u64,
) {
    let _ = indexer(w)
        .lookup_asset_balances(
            AssetId(index),
            None,
            opt_u64(limit),
            opt_str(&next),
            Some(cgt),
            opt_u64(clt),
        )
        .await;
}

#[allow(clippy::too_many_arguments)]
#[when(
    regex = r#"^we make a Lookup Asset Transactions call against asset index (\d+) with NotePrefix "([^"]*)" TxType "([^"]*)" SigType "([^"]*)" txid "([^"]*)" round (\d+) minRound (\d+) maxRound (\d+) limit (\d+) beforeTime "([^"]*)" afterTime "([^"]*)" currencyGreaterThan (\d+) currencyLessThan (\d+) address "([^"]*)" addressRole "([^"]*)" ExcluseCloseTo "([^"]*)"$"#
)]
async fn indexer_lookup_asset_transactions(
    w: &mut UnitWorld,
    index: u64,
    note_prefix: String,
    tx_type: String,
    sig_type: String,
    txid: String,
    round: u64,
    min_round: u64,
    max_round: u64,
    limit: u64,
    before_time: String,
    after_time: String,
    cgt: u64,
    clt: u64,
    address: String,
    address_role: String,
    exclude_close_to: String,
) {
    let _ = indexer(w)
        .lookup_asset_transactions(
            AssetId(index),
            opt_u64(limit),
            None,
            opt_str(&note_prefix),
            opt_str(&tx_type),
            opt_str(&sig_type),
            opt_tx_id(&txid).as_ref(),
            opt_u64(round),
            opt_u64(min_round),
            opt_u64(max_round),
            opt_str(&before_time).map(str::to_string),
            opt_str(&after_time).map(str::to_string),
            Some(cgt),
            opt_u64(clt),
            opt_address(&address).as_ref(),
            opt_str(&address_role),
            opt_bool(&exclude_close_to),
            None,
        )
        .await;
}

#[allow(clippy::too_many_arguments)]
#[when(
    regex = r#"^we make a Lookup Asset Transactions call against asset index (\d+) with NotePrefix "([^"]*)" TxType "([^"]*)" SigType "([^"]*)" txid "([^"]*)" round (\d+) minRound (\d+) maxRound (\d+) limit (\d+) beforeTime "([^"]*)" afterTime "([^"]*)" currencyGreaterThan (\d+) currencyLessThan (\d+) address "([^"]*)" addressRole "([^"]*)" ExcluseCloseTo "([^"]*)" RekeyTo "([^"]*)"$"#
)]
async fn indexer_lookup_asset_transactions_rekey(
    w: &mut UnitWorld,
    index: u64,
    note_prefix: String,
    tx_type: String,
    sig_type: String,
    txid: String,
    round: u64,
    min_round: u64,
    max_round: u64,
    limit: u64,
    before_time: String,
    after_time: String,
    cgt: u64,
    clt: u64,
    address: String,
    address_role: String,
    exclude_close_to: String,
    rekey_to: String,
) {
    let _ = indexer(w)
        .lookup_asset_transactions(
            AssetId(index),
            opt_u64(limit),
            None,
            opt_str(&note_prefix),
            opt_str(&tx_type),
            opt_str(&sig_type),
            opt_tx_id(&txid).as_ref(),
            opt_u64(round),
            opt_u64(min_round),
            opt_u64(max_round),
            opt_str(&before_time).map(str::to_string),
            opt_str(&after_time).map(str::to_string),
            Some(cgt),
            opt_u64(clt),
            opt_address(&address).as_ref(),
            opt_str(&address_role),
            opt_bool(&exclude_close_to),
            opt_bool(&rekey_to),
        )
        .await;
}

#[allow(clippy::too_many_arguments)]
#[when(
    regex = r#"^we make a Lookup Account Transactions call against account "([^"]*)" with NotePrefix "([^"]*)" TxType "([^"]*)" SigType "([^"]*)" txid "([^"]*)" round (\d+) minRound (\d+) maxRound (\d+) limit (\d+) beforeTime "([^"]*)" afterTime "([^"]*)" currencyGreaterThan (\d+) currencyLessThan (\d+) assetIndex (\d+)$"#
)]
async fn indexer_lookup_account_transactions(
    w: &mut UnitWorld,
    account: String,
    note_prefix: String,
    tx_type: String,
    sig_type: String,
    txid: String,
    round: u64,
    min_round: u64,
    max_round: u64,
    limit: u64,
    before_time: String,
    after_time: String,
    cgt: u64,
    clt: u64,
    index: u64,
) {
    let _ = indexer(w)
        .lookup_account_transactions(
            &parse_address(&account),
            opt_u64(limit),
            None,
            opt_str(&note_prefix),
            opt_str(&tx_type),
            opt_str(&sig_type),
            opt_tx_id(&txid).as_ref(),
            opt_u64(round),
            opt_u64(min_round),
            opt_u64(max_round),
            opt_asset_id(index),
            opt_str(&before_time).map(str::to_string),
            opt_str(&after_time).map(str::to_string),
            Some(cgt),
            opt_u64(clt),
            None,
        )
        .await;
}

#[allow(clippy::too_many_arguments)]
#[when(
    regex = r#"^we make a Lookup Account Transactions call against account "([^"]*)" with NotePrefix "([^"]*)" TxType "([^"]*)" SigType "([^"]*)" txid "([^"]*)" round (\d+) minRound (\d+) maxRound (\d+) limit (\d+) beforeTime "([^"]*)" afterTime "([^"]*)" currencyGreaterThan (\d+) currencyLessThan (\d+) assetIndex (\d+) rekeyTo "([^"]*)"$"#
)]
async fn indexer_lookup_account_transactions_rekey(
    w: &mut UnitWorld,
    account: String,
    note_prefix: String,
    tx_type: String,
    sig_type: String,
    txid: String,
    round: u64,
    min_round: u64,
    max_round: u64,
    limit: u64,
    before_time: String,
    after_time: String,
    cgt: u64,
    clt: u64,
    index: u64,
    rekey_to: String,
) {
    let _ = indexer(w)
        .lookup_account_transactions(
            &parse_address(&account),
            opt_u64(limit),
            None,
            opt_str(&note_prefix),
            opt_str(&tx_type),
            opt_str(&sig_type),
            opt_tx_id(&txid).as_ref(),
            opt_u64(round),
            opt_u64(min_round),
            opt_u64(max_round),
            opt_asset_id(index),
            opt_str(&before_time).map(str::to_string),
            opt_str(&after_time).map(str::to_string),
            Some(cgt),
            opt_u64(clt),
            opt_bool(&rekey_to),
        )
        .await;
}

#[when(regex = r"^we make a Lookup Block call against round (\d+)$")]
async fn indexer_lookup_block(w: &mut UnitWorld, round: u64) {
    let _ = indexer(w).lookup_block(round, None).await;
}

#[when(regex = r#"^we make a Lookup Block call against round (\d+) and header "([^"]*)"$"#)]
async fn indexer_lookup_block_header(w: &mut UnitWorld, round: u64, header: String) {
    let _ = indexer(w).lookup_block(round, opt_bool(&header)).await;
}

#[when(
    regex = r#"^we make a Lookup Account by ID call against account "([^"]*)" with round (\d+)$"#
)]
async fn indexer_lookup_account_by_id(w: &mut UnitWorld, account: String, round: u64) {
    let _ = indexer(w)
        .lookup_account_by_id(&parse_address(&account), opt_u64(round), None, None)
        .await;
}

#[when(
    regex = r#"^we make a Lookup Account by ID call against account "([^"]*)" with exclude "([^"]*)"$"#
)]
async fn indexer_lookup_account_by_id_exclude(w: &mut UnitWorld, account: String, exclude: String) {
    let _ = indexer(w)
        .lookup_account_by_id(&parse_address(&account), None, None, opt_csv(&exclude))
        .await;
}

#[when(regex = r"^we make a Lookup Asset by ID call against asset index (\d+)$")]
async fn indexer_lookup_asset_by_id(w: &mut UnitWorld, index: u64) {
    let _ = indexer(w).lookup_asset_by_id(AssetId(index), None).await;
}

#[when(
    regex = r"^we make a Search Accounts call with assetID (\d+) limit (\d+) currencyGreaterThan (\d+) currencyLessThan (\d+) and round (\d+)$"
)]
async fn indexer_search_accounts(
    w: &mut UnitWorld,
    index: u64,
    limit: u64,
    cgt: u64,
    clt: u64,
    round: u64,
) {
    let _ = indexer(w)
        .search_for_accounts(
            opt_asset_id(index),
            opt_u64(limit),
            None,
            Some(cgt),
            None,
            None,
            opt_u64(clt),
            None,
            opt_u64(round),
            None,
        )
        .await;
}

#[when(regex = r#"^we make a Search Accounts call with onlineOnly "([^"]*)"$"#)]
async fn indexer_search_accounts_online_only(w: &mut UnitWorld, _online_only: String) {
    // NOTE: `online-only` is not a parameter the algonaut indexer client
    // exposes; see the `excluded_tags` rationale in `features_runner.rs`.
    let _ = indexer(w)
        .search_for_accounts(None, None, None, None, None, None, None, None, None, None)
        .await;
}

#[allow(clippy::too_many_arguments)]
#[when(
    regex = r#"^we make a Search For Transactions call with account "([^"]*)" NotePrefix "([^"]*)" TxType "([^"]*)" SigType "([^"]*)" txid "([^"]*)" round (\d+) minRound (\d+) maxRound (\d+) limit (\d+) beforeTime "([^"]*)" afterTime "([^"]*)" currencyGreaterThan (\d+) currencyLessThan (\d+) assetIndex (\d+) addressRole "([^"]*)" ExcluseCloseTo "([^"]*)" groupid "([^"]*)"$"#
)]
async fn indexer_search_for_transactions(
    w: &mut UnitWorld,
    account: String,
    note_prefix: String,
    tx_type: String,
    sig_type: String,
    txid: String,
    round: u64,
    min_round: u64,
    max_round: u64,
    limit: u64,
    before_time: String,
    after_time: String,
    cgt: u64,
    clt: u64,
    index: u64,
    address_role: String,
    exclude_close_to: String,
    _group_id: String,
) {
    let _ = indexer(w)
        .search_for_transactions(
            opt_u64(limit),
            None,
            opt_str(&note_prefix),
            opt_str(&tx_type),
            opt_str(&sig_type),
            opt_tx_id(&txid).as_ref(),
            opt_u64(round),
            opt_u64(min_round),
            opt_u64(max_round),
            opt_asset_id(index),
            opt_str(&before_time).map(str::to_string),
            opt_str(&after_time).map(str::to_string),
            Some(cgt),
            opt_u64(clt),
            opt_address(&account).as_ref(),
            opt_str(&address_role),
            opt_bool(&exclude_close_to),
            None,
            None,
        )
        .await;
}

#[allow(clippy::too_many_arguments)]
#[when(
    regex = r#"^we make a Search For Transactions call with account "([^"]*)" NotePrefix "([^"]*)" TxType "([^"]*)" SigType "([^"]*)" txid "([^"]*)" round (\d+) minRound (\d+) maxRound (\d+) limit (\d+) beforeTime "([^"]*)" afterTime "([^"]*)" currencyGreaterThan (\d+) currencyLessThan (\d+) assetIndex (\d+) addressRole "([^"]*)" ExcluseCloseTo "([^"]*)" groupid "([^"]*)" rekeyTo "([^"]*)"$"#
)]
async fn indexer_search_for_transactions_rekey(
    w: &mut UnitWorld,
    account: String,
    note_prefix: String,
    tx_type: String,
    sig_type: String,
    txid: String,
    round: u64,
    min_round: u64,
    max_round: u64,
    limit: u64,
    before_time: String,
    after_time: String,
    cgt: u64,
    clt: u64,
    index: u64,
    address_role: String,
    exclude_close_to: String,
    _group_id: String,
    rekey_to: String,
) {
    let _ = indexer(w)
        .search_for_transactions(
            opt_u64(limit),
            None,
            opt_str(&note_prefix),
            opt_str(&tx_type),
            opt_str(&sig_type),
            opt_tx_id(&txid).as_ref(),
            opt_u64(round),
            opt_u64(min_round),
            opt_u64(max_round),
            opt_asset_id(index),
            opt_str(&before_time).map(str::to_string),
            opt_str(&after_time).map(str::to_string),
            Some(cgt),
            opt_u64(clt),
            opt_address(&account).as_ref(),
            opt_str(&address_role),
            opt_bool(&exclude_close_to),
            opt_bool(&rekey_to),
            None,
        )
        .await;
}

#[allow(clippy::too_many_arguments)]
#[when(
    regex = r#"^we make a Search For BlockHeaders call with minRound (\d+) maxRound (\d+) limit (\d+) nextToken "([^"]*)" beforeTime "([^"]*)" afterTime "([^"]*)" proposers "([^"]*)" expired "([^"]*)" absent "([^"]*)"$"#
)]
async fn indexer_search_for_block_headers(
    w: &mut UnitWorld,
    min_round: u64,
    max_round: u64,
    limit: u64,
    next: String,
    before_time: String,
    after_time: String,
    proposers: String,
    expired: String,
    absent: String,
) {
    let _ = indexer(w)
        .search_for_block_headers(
            opt_u64(limit),
            opt_str(&next),
            opt_u64(min_round),
            opt_u64(max_round),
            opt_str(&before_time).map(str::to_string),
            opt_str(&after_time).map(str::to_string),
            opt_csv(&proposers),
            opt_csv(&expired),
            opt_csv(&absent),
        )
        .await;
}

#[when(
    regex = r#"^we make a SearchForAssets call with limit (\d+) creator "([^"]*)" name "([^"]*)" unit "([^"]*)" index (\d+)$"#
)]
async fn indexer_search_for_assets(
    w: &mut UnitWorld,
    limit: u64,
    creator: String,
    name: String,
    unit: String,
    index: u64,
) {
    let _ = indexer(w)
        .search_for_assets(
            None,
            opt_u64(limit),
            None,
            opt_address(&creator).as_ref(),
            opt_str(&name),
            opt_str(&unit),
            opt_asset_id(index),
        )
        .await;
}

#[when(
    regex = r#"^we make a Search Accounts call with assetID (\d+) limit (\d+) currencyGreaterThan (\d+) currencyLessThan (\d+) round (\d+) and authenticating address "([^"]*)"$"#
)]
async fn indexer_search_accounts_auth(
    w: &mut UnitWorld,
    index: u64,
    limit: u64,
    cgt: u64,
    clt: u64,
    round: u64,
    auth_addr: String,
) {
    let _ = indexer(w)
        .search_for_accounts(
            opt_asset_id(index),
            opt_u64(limit),
            None,
            Some(cgt),
            None,
            None,
            opt_u64(clt),
            opt_address(&auth_addr).as_ref(),
            opt_u64(round),
            None,
        )
        .await;
}

#[when(regex = r#"^we make a Search Accounts call with exclude "([^"]*)"$"#)]
async fn indexer_search_accounts_exclude(w: &mut UnitWorld, exclude: String) {
    let _ = indexer(w)
        .search_for_accounts(
            None,
            None,
            None,
            None,
            None,
            opt_csv(&exclude),
            None,
            None,
            None,
            None,
        )
        .await;
}

#[when(regex = r"^we make a SearchForApplications call with applicationID (\d+)$")]
async fn indexer_search_for_applications(w: &mut UnitWorld, application_id: u64) {
    let _ = indexer(w)
        .search_for_applications(opt_app_id(application_id), None, None, None, None)
        .await;
}

#[when(regex = r#"^we make a SearchForApplications call with creator "([^"]*)"$"#)]
async fn indexer_search_for_applications_creator(w: &mut UnitWorld, creator: String) {
    let _ = indexer(w)
        .search_for_applications(None, opt_address(&creator).as_ref(), None, None, None)
        .await;
}

#[when(regex = r"^we make a LookupApplications call with applicationID (\d+)$")]
async fn indexer_lookup_applications(w: &mut UnitWorld, application_id: u64) {
    let _ = indexer(w)
        .lookup_application_by_id(AppId(application_id), None)
        .await;
}

#[when(
    regex = r#"^we make a LookupApplicationBoxByIDandName call with applicationID (\d+) with encoded box name "([^"]*)"$"#
)]
async fn indexer_lookup_application_box(w: &mut UnitWorld, application_id: u64, name: String) {
    let _ = indexer(w)
        .lookup_application_box_by_id_and_name(AppId(application_id), &name)
        .await;
}

#[when(
    regex = r#"^we make a SearchForApplicationBoxes call with applicationID (\d+) with max (\d+) nextToken "([^"]*)"$"#
)]
async fn indexer_search_for_application_boxes(
    w: &mut UnitWorld,
    application_id: u64,
    max: u64,
    next: String,
) {
    let _ = indexer(w)
        .search_for_application_boxes(AppId(application_id), opt_u64(max), opt_str(&next))
        .await;
}

#[allow(clippy::too_many_arguments)]
#[when(
    regex = r#"^we make a LookupApplicationLogsByID call with applicationID (\d+) limit (\d+) minRound (\d+) maxRound (\d+) nextToken "([^"]*)" sender "([^"]*)" and txID "([^"]*)"$"#
)]
async fn indexer_lookup_application_logs(
    w: &mut UnitWorld,
    application_id: u64,
    limit: u64,
    min_round: u64,
    max_round: u64,
    next: String,
    sender: String,
    txid: String,
) {
    let _ = indexer(w)
        .lookup_application_logs_by_id(
            AppId(application_id),
            opt_u64(limit),
            opt_str(&next),
            opt_tx_id(&txid).as_ref(),
            opt_u64(min_round),
            opt_u64(max_round),
            opt_str(&sender),
        )
        .await;
}

#[when(
    regex = r#"^we make a LookupAccountAssets call with accountID "([^"]*)" assetID (\d+) includeAll "([^"]*)" limit (\d+) next "([^"]*)"$"#
)]
async fn indexer_lookup_account_assets(
    w: &mut UnitWorld,
    account_id: String,
    asset_id: u64,
    include_all: String,
    limit: u64,
    next: String,
) {
    let _ = indexer(w)
        .lookup_account_assets(
            &parse_address(&account_id),
            opt_asset_id(asset_id),
            opt_bool(&include_all),
            opt_u64(limit),
            opt_str(&next),
        )
        .await;
}

#[when(
    regex = r#"^we make a LookupAccountCreatedAssets call with accountID "([^"]*)" assetID (\d+) includeAll "([^"]*)" limit (\d+) next "([^"]*)"$"#
)]
async fn indexer_lookup_account_created_assets(
    w: &mut UnitWorld,
    account_id: String,
    asset_id: u64,
    include_all: String,
    limit: u64,
    next: String,
) {
    let _ = indexer(w)
        .lookup_account_created_assets(
            &parse_address(&account_id),
            opt_asset_id(asset_id),
            opt_bool(&include_all),
            opt_u64(limit),
            opt_str(&next),
        )
        .await;
}

#[when(
    regex = r#"^we make a LookupAccountAppLocalStates call with accountID "([^"]*)" applicationID (\d+) includeAll "([^"]*)" limit (\d+) next "([^"]*)"$"#
)]
async fn indexer_lookup_account_app_local_states(
    w: &mut UnitWorld,
    account_id: String,
    application_id: u64,
    include_all: String,
    limit: u64,
    next: String,
) {
    let _ = indexer(w)
        .lookup_account_app_local_states(
            &parse_address(&account_id),
            opt_app_id(application_id),
            opt_bool(&include_all),
            opt_u64(limit),
            opt_str(&next),
        )
        .await;
}

#[when(
    regex = r#"^we make a LookupAccountCreatedApplications call with accountID "([^"]*)" applicationID (\d+) includeAll "([^"]*)" limit (\d+) next "([^"]*)"$"#
)]
async fn indexer_lookup_account_created_applications(
    w: &mut UnitWorld,
    account_id: String,
    application_id: u64,
    include_all: String,
    limit: u64,
    next: String,
) {
    let _ = indexer(w)
        .lookup_account_created_applications(
            &parse_address(&account_id),
            opt_app_id(application_id),
            opt_bool(&include_all),
            opt_u64(limit),
            opt_str(&next),
        )
        .await;
}

/// Treat the string `"true"` as `Some(true)`, everything else (including
/// `"false"` and empty) as "argument omitted".
fn opt_bool(s: &str) -> Option<bool> {
    if s == "true" { Some(true) } else { None }
}
