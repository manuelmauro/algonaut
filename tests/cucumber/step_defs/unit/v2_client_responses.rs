//! Step definitions for the `v2algodclient_responses` and
//! `v2indexerclient_responses` unit features.
//!
//! These are fixture-driven response-deserialization unit tests. A canned
//! HTTP response body (a base64-decoded fixture from
//! `tests/features/resources/`) is loaded into a local
//! [`ResponseMockServer`]; the SDK is pointed at it, a high-level
//! `Algod`/`Indexer` call is made, and the *parsed* response is asserted on.
//!
//! The call arguments are deliberately placeholders — the mock answers every
//! request with the same canned body regardless of path or query.

use crate::step_defs::unit::mock_server::ResponseMockServer;
use crate::step_defs::unit::world::{UnitResponse, UnitWorld};
use algonaut::algod::v2::Algod;
use algonaut::indexer::v2::Indexer;
use algonaut_core::{Address, AssetId, Round, TransactionId};
use algonaut_encoding::decode_base64;
use cucumber::{given, then, when};
use std::fs;
use std::path::Path;

/// A token (the `Configuration` requires one; its value is irrelevant to a
/// canned-response server).
const TOKEN: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

/// Placeholder Algorand address: the mock server answers every request with
/// the same canned body regardless of the path, so the address value is
/// irrelevant; this just needs to be well-formed.
fn placeholder_address() -> Address {
    "7ZUECA7HFLZTXENRV24SHLU4AVPUTMTTDUFUBNBD64C73F3UHRTHAIOF6Q"
        .parse()
        .expect("placeholder address is well-formed")
}

/// Placeholder transaction ID: see `placeholder_address` — value is
/// irrelevant to the response-mock test, only the shape matters.
fn placeholder_transaction_id() -> TransactionId {
    TransactionId("placeholder-txid".to_string())
}

/// Load a fixture's bytes as the raw HTTP response body.
///
/// `*.base64` fixtures hold a base64-encoded body (msgpack or otherwise) and
/// are decoded; every other fixture (`*.json`, …) is served verbatim.
fn load_body(directory: &str, file: &str) -> Vec<u8> {
    let path = Path::new("tests/features/resources")
        .join(directory)
        .join(file);
    let raw = fs::read(&path).unwrap_or_else(|e| panic!("reading fixture {path:?}: {e}"));
    if file.ends_with(".base64") {
        // Fixtures may carry trailing newlines/whitespace; strip before decode.
        let trimmed: Vec<u8> = raw
            .iter()
            .copied()
            .filter(|b| !b.is_ascii_whitespace())
            .collect();
        decode_base64(&trimmed).unwrap_or_else(|e| panic!("base64-decoding {path:?}: {e}"))
    } else {
        raw
    }
}

fn algod(w: &UnitWorld) -> Algod {
    let base = w
        .response_server
        .as_ref()
        .expect("response mock server not started")
        .base_url
        .clone();
    Algod::new(&base, TOKEN).expect("algod client")
}

fn indexer(w: &UnitWorld) -> Indexer {
    let base = w
        .response_server
        .as_ref()
        .expect("response mock server not started")
        .base_url
        .clone();
    Indexer::new(&base, TOKEN).expect("indexer client")
}

/// Stash the call outcome: a flattened `Ok(())`/`Err(msg)` plus, on success,
/// the parsed payload wrapped in [`UnitResponse`].
fn record<T>(
    w: &mut UnitWorld,
    result: Result<T, algonaut::Error>,
    wrap: impl FnOnce(T) -> UnitResponse,
) {
    match result {
        Ok(value) => {
            w.last_call_error = Some(Ok(()));
            w.last_response = Some(wrap(value));
        }
        Err(e) => {
            w.last_call_error = Some(Err(e.to_string()));
            w.last_response = None;
        }
    }
}

// --- Given -----------------------------------------------------------------

#[given(regex = r#"^mock http responses in "([^"]*)" loaded from "([^"]*)"$"#)]
async fn mock_http_responses(w: &mut UnitWorld, jsonfiles: String, directory: String) {
    // `jsonfiles` may be a comma-separated list; the upstream `*_responses`
    // scenarios only ever name a single body, so use the first.
    let file = jsonfiles
        .split(',')
        .next()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .expect("no fixture file named");
    let body = load_body(&directory, file);
    // `*.base64` fixtures hold a base64-encoded *msgpack* body; serve them as
    // `application/msgpack` so the SDK's content negotiation decodes them with
    // `rmp_serde`. Everything else (`*.json`, …) is JSON.
    let content_type = if file.ends_with(".base64") {
        "application/msgpack"
    } else {
        "application/json"
    };
    w.response_server = Some(ResponseMockServer::start(body, content_type).await);
}

// === Algod v2 responses ====================================================

#[when(regex = r"^we make any Pending Transaction Information call$")]
async fn algod_pending_txn(w: &mut UnitWorld) {
    let r = algod(w)
        .pending_transaction(&placeholder_transaction_id())
        .await;
    record(w, r, UnitResponse::PendingTransaction);
}

#[when(regex = r"^we make any Pending Transactions Information call$")]
async fn algod_pending_txns(w: &mut UnitWorld) {
    let r = algod(w).pending_transactions(None, None).await;
    record(w, r, UnitResponse::PendingTransactions);
}

#[when(regex = r"^we make any Send Raw Transaction call$")]
async fn algod_send_raw_txn(w: &mut UnitWorld) {
    let r = algod(w).send_raw(&[0u8]).await;
    record(w, r, UnitResponse::RawTransaction);
}

#[when(regex = r"^we make any Pending Transactions By Address call$")]
async fn algod_pending_txns_by_address(w: &mut UnitWorld) {
    let r = algod(w)
        .address_pending_transactions(&placeholder_address(), None, None)
        .await;
    record(w, r, UnitResponse::PendingTransactions);
}

#[when(regex = r"^we make any Node Status call$")]
async fn algod_node_status(w: &mut UnitWorld) {
    let r = algod(w).status().await;
    record(w, r, UnitResponse::Status);
}

#[when(regex = r"^we make any Ledger Supply call$")]
async fn algod_ledger_supply(w: &mut UnitWorld) {
    let r = algod(w).supply().await;
    record(w, r, UnitResponse::Supply);
}

#[when(regex = r"^we make any Status After Block call$")]
async fn algod_status_after_block(w: &mut UnitWorld) {
    let r = algod(w).status_after_block(Round(0)).await;
    record(w, r, UnitResponse::Status);
}

#[when(regex = r"^we make any Account Information call$")]
async fn algod_account_information(w: &mut UnitWorld) {
    let r = algod(w).account(&placeholder_address()).await;
    record(w, r, UnitResponse::Account);
}

#[when(regex = r"^we make any Get Block call$")]
async fn algod_get_block(w: &mut UnitWorld) {
    let r = algod(w).block(0).await;
    record(w, r, UnitResponse::Block);
}

#[when(regex = r"^we make any Suggested Transaction Parameters call$")]
async fn algod_suggested_params(w: &mut UnitWorld) {
    let r = algod(w).suggested_params().await;
    record(w, r, UnitResponse::TransactionParams);
}

#[when(regex = r"^we make any Dryrun call$")]
async fn algod_dryrun(w: &mut UnitWorld) {
    let r = algod(w).teal_dryrun(None).await;
    record(w, r, UnitResponse::Dryrun);
}

// === Indexer v2 responses ==================================================

#[when(regex = r"^we make any LookupAssetBalances call$")]
async fn indexer_lookup_asset_balances(w: &mut UnitWorld) {
    let r = indexer(w)
        .lookup_asset_balances(AssetId(0), None, None, None, None, None)
        .await;
    record(w, r, UnitResponse::AssetBalances);
}

#[when(regex = r"^we make any LookupAssetTransactions call$")]
async fn indexer_lookup_asset_transactions(w: &mut UnitWorld) {
    let r = indexer(w)
        .lookup_asset_transactions(
            AssetId(0),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await;
    record(w, r, UnitResponse::Transactions);
}

#[when(regex = r"^we make any LookupAccountTransactions call$")]
async fn indexer_lookup_account_transactions(w: &mut UnitWorld) {
    let r = indexer(w)
        .lookup_account_transactions(
            &placeholder_address(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await;
    record(w, r, UnitResponse::Transactions);
}

#[when(regex = r"^we make any LookupBlock call$")]
async fn indexer_lookup_block(w: &mut UnitWorld) {
    let r = indexer(w).lookup_block(0, None).await;
    record(w, r, UnitResponse::IndexerBlock);
}

#[when(regex = r"^we make any LookupAccountByID call$")]
async fn indexer_lookup_account_by_id(w: &mut UnitWorld) {
    let r = indexer(w)
        .lookup_account_by_id(&placeholder_address(), None, None, None)
        .await;
    record(w, r, UnitResponse::IndexerAccount);
}

#[when(regex = r"^we make any LookupAssetByID call$")]
async fn indexer_lookup_asset_by_id(w: &mut UnitWorld) {
    let r = indexer(w).lookup_asset_by_id(AssetId(0), None).await;
    record(w, r, UnitResponse::IndexerAsset);
}

#[when(regex = r"^we make any SearchAccounts call$")]
async fn indexer_search_accounts(w: &mut UnitWorld) {
    let r = indexer(w)
        .search_for_accounts(None, None, None, None, None, None, None, None, None, None)
        .await;
    record(w, r, UnitResponse::Accounts);
}

#[when(regex = r"^we make any SearchForTransactions call$")]
async fn indexer_search_for_transactions(w: &mut UnitWorld) {
    let r = indexer(w)
        .search_for_transactions(
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None,
        )
        .await;
    record(w, r, UnitResponse::Transactions);
}

#[when(regex = r"^we make any SearchForBlockHeaders call$")]
async fn indexer_search_for_block_headers(w: &mut UnitWorld) {
    let r = indexer(w)
        .search_for_block_headers(None, None, None, None, None, None, None, None, None)
        .await;
    record(w, r, UnitResponse::BlockHeaders);
}

#[when(regex = r"^we make any SearchForAssets call$")]
async fn indexer_search_for_assets(w: &mut UnitWorld) {
    // `SearchForAssets` shares its response model with
    // `lookup_account_created_assets`; the assertion only reads the asset
    // array, so reuse the `AssetBalances`-adjacent path via the typed helper.
    let r = indexer(w)
        .search_for_assets(None, None, None, None, None, None, None)
        .await;
    match r {
        Ok(value) => {
            w.last_call_error = Some(Ok(()));
            w.last_response = Some(UnitResponse::SearchAssets(value));
        }
        Err(e) => {
            w.last_call_error = Some(Err(e.to_string()));
            w.last_response = None;
        }
    }
}

// --- Then: error string ----------------------------------------------------

#[then(regex = r#"^expect error string to contain "([^"]*)"$"#)]
async fn expect_error_string(w: &mut UnitWorld, expected: String) {
    let outcome = w.last_call_error.as_ref().expect("no client call recorded");
    if expected.is_empty() {
        assert!(
            outcome.is_ok(),
            "expected the call to succeed, got error: {:?}",
            outcome.as_ref().err()
        );
    } else {
        match outcome {
            Ok(()) => panic!("expected an error containing `{expected}`, but the call succeeded"),
            Err(msg) => assert!(
                msg.contains(&expected),
                "expected error to contain `{expected}`, got `{msg}`"
            ),
        }
    }
}

// --- Then/And: algod response assertions -----------------------------------

#[then(
    regex = r#"^the parsed Pending Transaction Information response should have sender "([^"]*)"$"#
)]
async fn algod_assert_pending_txn_sender(w: &mut UnitWorld, sender: String) {
    let UnitResponse::PendingTransaction(resp) = w.last_response.as_ref().expect("no response")
    else {
        panic!("last response is not a PendingTransactionResponse");
    };
    let actual = txn_sender(&resp.txn);
    assert_eq!(actual, sender, "pending transaction sender mismatch");
}

#[then(
    regex = r#"^the parsed Pending Transactions Information response should contain an array of len (\d+) and element number (\d+) should have sender "([^"]*)"$"#
)]
async fn algod_assert_pending_txns(w: &mut UnitWorld, len: usize, idx: usize, sender: String) {
    let UnitResponse::PendingTransactions(resp) = w.last_response.as_ref().expect("no response")
    else {
        panic!("last response is not a PendingTransactions");
    };
    assert_eq!(resp.top_transactions.len(), len, "pending tx array length");
    let actual = txn_sender(&resp.top_transactions[idx]);
    assert_eq!(actual, sender, "pending transaction sender mismatch");
}

#[then(
    regex = r#"^the parsed Pending Transactions By Address response should contain an array of len (\d+) and element number (\d+) should have sender "([^"]*)"$"#
)]
async fn algod_assert_pending_txns_by_address(
    w: &mut UnitWorld,
    len: usize,
    idx: usize,
    sender: String,
) {
    algod_assert_pending_txns(w, len, idx, sender).await;
}

#[then(regex = r#"^the parsed Send Raw Transaction response should have txid "([^"]*)"$"#)]
async fn algod_assert_send_raw_txn(w: &mut UnitWorld, transaction_id: String) {
    let UnitResponse::RawTransaction(resp) = w.last_response.as_ref().expect("no response") else {
        panic!("last response is not a SubmitResponse");
    };
    assert_eq!(resp.tx_id, transaction_id, "raw transaction txid mismatch");
}

#[then(regex = r"^the parsed Node Status response should have a last round of (\d+)$")]
async fn algod_assert_node_status(w: &mut UnitWorld, round: u64) {
    let UnitResponse::Status(resp) = w.last_response.as_ref().expect("no response") else {
        panic!("last response is not a NodeStatus");
    };
    assert_eq!(resp.last_round.0, round, "node status last round mismatch");
}

#[then(regex = r"^the parsed Status After Block response should have a last round of (\d+)$")]
async fn algod_assert_status_after_block(w: &mut UnitWorld, round: u64) {
    algod_assert_node_status(w, round).await;
}

#[then(
    regex = r"^the parsed Ledger Supply response should have totalMoney (\d+) onlineMoney (\d+) on round (\d+)$"
)]
async fn algod_assert_ledger_supply(w: &mut UnitWorld, total: u64, online: u64, round: u64) {
    let UnitResponse::Supply(resp) = w.last_response.as_ref().expect("no response") else {
        panic!("last response is not a Supply");
    };
    assert_eq!(
        resp.total_money.0, total,
        "ledger supply total money mismatch"
    );
    assert_eq!(
        resp.online_money.0, online,
        "ledger supply online money mismatch"
    );
    assert_eq!(
        resp.current_round.0, round,
        "ledger supply current round mismatch"
    );
}

#[then(regex = r#"^the parsed Account Information response should have address "([^"]*)"$"#)]
async fn algod_assert_account_information(w: &mut UnitWorld, address: String) {
    let UnitResponse::Account(resp) = w.last_response.as_ref().expect("no response") else {
        panic!("last response is not an Account");
    };
    assert_eq!(resp.address, address, "account address mismatch");
}

#[then(regex = r#"^the parsed Get Block response should have rewards pool "([^"]*)"$"#)]
async fn algod_assert_get_block_pool(w: &mut UnitWorld, pool: String) {
    let UnitResponse::Block(resp) = w.last_response.as_ref().expect("no response") else {
        panic!("last response is not a BlockResponse");
    };
    let actual = resp
        .block
        .rewards_pool_base64()
        .expect("block has no rewards pool");
    assert_eq!(actual, pool, "block rewards pool mismatch");
}

#[then(
    regex = r#"^the parsed Get Block response should have rewards pool "([^"]*)" and no certificate or payset$"#
)]
async fn algod_assert_get_block_header_only(w: &mut UnitWorld, pool: String) {
    let UnitResponse::Block(resp) = w.last_response.as_ref().expect("no response") else {
        panic!("last response is not a BlockResponse");
    };
    let actual = resp
        .block
        .rewards_pool_base64()
        .expect("block has no rewards pool");
    assert_eq!(actual, pool, "block rewards pool mismatch");
    assert!(
        resp.block.txns.is_none() || resp.block.txns.as_ref().is_some_and(Vec::is_empty),
        "header-only block should carry no payset"
    );
}

#[then(regex = r#"^the parsed Get Block response should have heartbeat address "([^"]*)"$"#)]
async fn algod_assert_get_block_heartbeat(w: &mut UnitWorld, hbaddress: String) {
    let UnitResponse::Block(resp) = w.last_response.as_ref().expect("no response") else {
        panic!("last response is not a BlockResponse");
    };
    let txns = resp.block.txns.as_ref().expect("block has no payset");
    let actual = txns
        .iter()
        .filter_map(|t| t.txn.as_ref())
        .find_map(|txn| txn.heartbeat_address())
        .expect("block payset has no heartbeat transaction")
        .to_string();
    assert_eq!(actual, hbaddress, "block heartbeat address mismatch");
}

#[then(
    regex = r"^the parsed Suggested Transaction Parameters response should have first round valid of (\d+)$"
)]
async fn algod_assert_suggested_params(w: &mut UnitWorld, round: u64) {
    let UnitResponse::TransactionParams(resp) = w.last_response.as_ref().expect("no response")
    else {
        panic!("last response is not a SuggestedParams");
    };
    assert_eq!(
        resp.last_round.0, round,
        "suggested params last round mismatch"
    );
}

#[then(regex = r#"^the parsed Dryrun Response should have global delta "([^"]*)" with (\d+)$"#)]
async fn algod_assert_dryrun(w: &mut UnitWorld, key: String, action: u64) {
    let UnitResponse::Dryrun(resp) = w.last_response.as_ref().expect("no response") else {
        panic!("last response is not a DryrunResponse");
    };
    let deltas = resp.txns[0]
        .global_delta
        .as_ref()
        .expect("dryrun txn has no global delta");
    let entry = deltas
        .iter()
        .find(|kv| kv.key == key)
        .unwrap_or_else(|| panic!("no global-delta entry for key `{key}`"));
    assert_eq!(entry.value.action, action, "global delta action mismatch");
}

/// Pull the `snd` (sender) base32 address out of a parsed signed transaction.
///
/// The pending-transaction fixtures arrive msgpack-encoded; the SDK decodes
/// the inner `txn.snd` into an `Address`, which renders as the canonical
/// base32-checksum string.
fn txn_sender(header: &algonaut_model::algod::ext::transaction::TransactionHeader) -> String {
    header
        .txn
        .as_ref()
        .and_then(|txn| txn.sender())
        .map(ToString::to_string)
        .expect("transaction has no `txn.snd`")
}

// --- Then/And: indexer response assertions ---------------------------------

#[then(
    regex = r#"^the parsed LookupAssetBalances response should be valid on round (\d+), and contain an array of len (\d+) and element number (\d+) should have address "([^"]*)" amount (\d+) and frozen state "([^"]*)"$"#
)]
async fn indexer_assert_asset_balances(
    w: &mut UnitWorld,
    round: u64,
    len: usize,
    idx: usize,
    address: String,
    amount: u64,
    frozen: String,
) {
    let UnitResponse::AssetBalances(resp) = w.last_response.as_ref().expect("no response") else {
        panic!("last response is not a AssetBalancesResponse");
    };
    assert_eq!(resp.current_round, round, "asset balances round mismatch");
    assert_eq!(resp.balances.len(), len, "asset balances array length");
    let holding = &resp.balances[idx];
    assert_eq!(holding.address, address, "balance address mismatch");
    assert_eq!(holding.amount, amount, "balance amount mismatch");
    assert_eq!(
        holding.is_frozen,
        frozen == "true",
        "balance frozen state mismatch"
    );
}

#[then(
    regex = r#"^the parsed LookupAssetTransactions response should be valid on round (\d+), and contain an array of len (\d+) and element number (\d+) should have sender "([^"]*)"$"#
)]
async fn indexer_assert_asset_transactions(
    w: &mut UnitWorld,
    round: u64,
    len: usize,
    idx: usize,
    sender: String,
) {
    indexer_assert_transactions(w, round, len, idx, sender).await;
}

#[then(
    regex = r#"^the parsed LookupAccountTransactions response should be valid on round (\d+), and contain an array of len (\d+) and element number (\d+) should have sender "([^"]*)"$"#
)]
async fn indexer_assert_account_transactions(
    w: &mut UnitWorld,
    round: u64,
    len: usize,
    idx: usize,
    sender: String,
) {
    indexer_assert_transactions(w, round, len, idx, sender).await;
}

/// Shared `Transactions` assertion: round + array length, and (when the array
/// is non-empty) the sender of one element. `sender == "N/A"` marks the
/// upstream "empty result set" rows, where no element is inspected.
async fn indexer_assert_transactions(
    w: &mut UnitWorld,
    round: u64,
    len: usize,
    idx: usize,
    sender: String,
) {
    let UnitResponse::Transactions(resp) = w.last_response.as_ref().expect("no response") else {
        panic!("last response is not a transactions response");
    };
    assert_eq!(resp.current_round, round, "transactions round mismatch");
    assert_eq!(resp.transactions.len(), len, "transactions array length");
    if sender != "N/A" {
        assert_eq!(
            resp.transactions[idx].sender, sender,
            "transaction sender mismatch"
        );
    }
}

#[then(regex = r#"^the parsed LookupBlock response should have previous block hash "([^"]*)"$"#)]
async fn indexer_assert_lookup_block(w: &mut UnitWorld, prev_hash: String) {
    let UnitResponse::IndexerBlock(block) = w.last_response.as_ref().expect("no response") else {
        panic!("last response is not an indexer Block");
    };
    assert_eq!(
        block.previous_block_hash.to_string(),
        prev_hash,
        "previous block hash mismatch"
    );
}

#[then(regex = r#"^the parsed LookupAccountByID response should have address "([^"]*)"$"#)]
async fn indexer_assert_lookup_account(w: &mut UnitWorld, address: String) {
    let UnitResponse::IndexerAccount(resp) = w.last_response.as_ref().expect("no response") else {
        panic!("last response is not a AccountResponse");
    };
    assert_eq!(resp.account.address, address, "account address mismatch");
}

#[then(regex = r"^the parsed LookupAssetByID response should have index (\d+)$")]
async fn indexer_assert_lookup_asset(w: &mut UnitWorld, index: u64) {
    let UnitResponse::IndexerAsset(resp) = w.last_response.as_ref().expect("no response") else {
        panic!("last response is not a AssetResponse");
    };
    assert_eq!(resp.asset.index, index, "asset index mismatch");
}

#[then(
    regex = r#"^the parsed SearchAccounts response should be valid on round (\d+) and the array should be of len (\d+) and the element at index (\d+) should have address "([^"]*)"$"#
)]
async fn indexer_assert_search_accounts(
    w: &mut UnitWorld,
    round: u64,
    len: usize,
    idx: usize,
    address: String,
) {
    let UnitResponse::Accounts(resp) = w.last_response.as_ref().expect("no response") else {
        panic!("last response is not a AccountsResponse");
    };
    assert_eq!(resp.current_round, round, "search accounts round mismatch");
    assert_eq!(resp.accounts.len(), len, "search accounts array length");
    assert_eq!(
        resp.accounts[idx].address, address,
        "search accounts address mismatch"
    );
}

// Two upstream `@unit.indexer.rekey` scenarios place this assertion in an
// `And` step directly after a `When` (no intervening `Then`); cucumber-rs
// resolves an `And` to the *previous* step's keyword, so the same handler
// must be registered as both `#[when]` and `#[then]`.
#[when(
    regex = r#"^the parsed SearchAccounts response should be valid on round (\d+) and the array should be of len (\d+) and the element at index (\d+) should have authorizing address "([^"]*)"$"#
)]
#[then(
    regex = r#"^the parsed SearchAccounts response should be valid on round (\d+) and the array should be of len (\d+) and the element at index (\d+) should have authorizing address "([^"]*)"$"#
)]
async fn indexer_assert_search_accounts_auth(
    w: &mut UnitWorld,
    round: u64,
    len: usize,
    idx: usize,
    auth_addr: String,
) {
    let UnitResponse::Accounts(resp) = w.last_response.as_ref().expect("no response") else {
        panic!("last response is not a AccountsResponse");
    };
    assert_eq!(resp.current_round, round, "search accounts round mismatch");
    assert_eq!(resp.accounts.len(), len, "search accounts array length");
    let actual = resp.accounts[idx]
        .auth_addr
        .as_deref()
        .expect("account has no authorizing address");
    assert_eq!(actual, auth_addr, "authorizing address mismatch");
}

#[then(
    regex = r#"^the parsed SearchForTransactions response should be valid on round (\d+) and the array should be of len (\d+) and the element at index (\d+) should have sender "([^"]*)"$"#
)]
async fn indexer_assert_search_transactions_sender(
    w: &mut UnitWorld,
    round: u64,
    len: usize,
    idx: usize,
    sender: String,
) {
    indexer_assert_transactions(w, round, len, idx, sender).await;
}

// `@unit.indexer.rekey` places this in an `And` after a `When`; register the
// handler for both keywords (see the SearchAccounts auth comment above).
#[when(
    regex = r#"^the parsed SearchForTransactions response should be valid on round (\d+) and the array should be of len (\d+) and the element at index (\d+) should have rekey-to "([^"]*)"$"#
)]
#[then(
    regex = r#"^the parsed SearchForTransactions response should be valid on round (\d+) and the array should be of len (\d+) and the element at index (\d+) should have rekey-to "([^"]*)"$"#
)]
async fn indexer_assert_search_transactions_rekey(
    w: &mut UnitWorld,
    round: u64,
    len: usize,
    idx: usize,
    rekey_to: String,
) {
    let UnitResponse::Transactions(resp) = w.last_response.as_ref().expect("no response") else {
        panic!("last response is not a transactions response");
    };
    assert_eq!(resp.current_round, round, "transactions round mismatch");
    assert_eq!(resp.transactions.len(), len, "transactions array length");
    let actual = resp.transactions[idx]
        .rekey_to
        .as_deref()
        .expect("transaction has no rekey-to address");
    assert_eq!(actual, rekey_to, "rekey-to address mismatch");
}

#[then(
    regex = r#"^the parsed SearchForTransactions response should be valid on round (\d+) and the array should be of len (\d+) and the element at index (\d+) should have hbaddress "([^"]*)"$"#
)]
async fn indexer_assert_search_transactions_heartbeat(
    w: &mut UnitWorld,
    round: u64,
    len: usize,
    idx: usize,
    hbaddress: String,
) {
    let UnitResponse::Transactions(resp) = w.last_response.as_ref().expect("no response") else {
        panic!("last response is not a transactions response");
    };
    assert_eq!(resp.current_round, round, "transactions round mismatch");
    assert_eq!(resp.transactions.len(), len, "transactions array length");
    let hb = resp.transactions[idx]
        .heartbeat_transaction
        .as_ref()
        .expect("transaction has no heartbeat sub-object");
    assert_eq!(hb.hb_address, hbaddress, "heartbeat address mismatch");
}

#[then(
    regex = r#"^the parsed SearchForBlockHeaders response should have a block array of len (\d+) and the element at index (\d+) should have round "([^"]*)"$"#
)]
async fn indexer_assert_search_block_headers(
    w: &mut UnitWorld,
    len: usize,
    idx: usize,
    round: u64,
) {
    let UnitResponse::BlockHeaders(resp) = w.last_response.as_ref().expect("no response") else {
        panic!("last response is not a BlockHeadersResponse");
    };
    assert_eq!(resp.blocks.len(), len, "block headers array length");
    assert_eq!(resp.blocks[idx].round, round, "block header round mismatch");
}

#[then(
    regex = r#"^the parsed SearchForAssets response should be valid on round (\d+) and the array should be of len (\d+) and the element at index (\d+) should have asset index (\d+)$"#
)]
async fn indexer_assert_search_assets(
    w: &mut UnitWorld,
    round: u64,
    len: usize,
    idx: usize,
    asset_index: u64,
) {
    let UnitResponse::SearchAssets(resp) = w.last_response.as_ref().expect("no response") else {
        panic!("last response is not a SearchForAssets response");
    };
    assert_eq!(resp.current_round, round, "search assets round mismatch");
    assert_eq!(resp.assets.len(), len, "search assets array length");
    assert_eq!(
        resp.assets[idx].index, asset_index,
        "search assets asset index mismatch"
    );
}
