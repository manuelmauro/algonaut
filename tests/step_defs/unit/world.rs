use crate::step_defs::unit::mock_server::{MockServer, ResponseMockServer};
use algonaut::algod::v2::Algod;
use algonaut::indexer::v2::Indexer;
use algonaut_core::{Address, MultisigAddress};
use algonaut_transaction::{SignedTransaction, Transaction, account::Account};
use cucumber;

/// Cucumber world for the **unit** features. These features don't need
/// a live algod or kmd — only fixture / parser state. Keep it tightly
/// scoped: anything that needs network access should live in the
/// integration `World`.
///
/// `dead_code` is suppressed wholesale because step-defs across the 17
/// unit features will populate these fields incrementally. Without the
/// allow, every PR that wires a new feature triggers spurious
/// dead-code warnings.
#[allow(dead_code)]
#[derive(Default, Debug, cucumber::World)]
pub struct UnitWorld {
    pub account: Option<Account>,
    pub address: Option<Address>,
    pub mnemonic: Option<String>,
    pub roundtrip_mnemonic: Option<String>,
    pub tx: Option<Transaction>,
    pub signed_tx: Option<SignedTransaction>,
    pub multisig: Option<MultisigAddress>,
    pub microalgos: Option<u64>,
    pub roundtrip_microalgos: Option<u64>,

    // --- v2 client path features ----------------------------------------
    /// Recording HTTP server the path features point the SDK clients at.
    pub mock_server: Option<MockServer>,
    /// `algonaut` algod v2 client wired to the mock server.
    pub algod: Option<Algod>,
    /// `algonaut` indexer v2 client wired to the mock server.
    pub indexer: Option<Indexer>,

    // --- v2 client response features ------------------------------------
    /// Canned-response HTTP server the `*_responses` features point the SDK
    /// clients at. Holds the base64-decoded fixture as the reply body.
    pub response_server: Option<ResponseMockServer>,
    /// The `Result` of the last `When we make any <X> call` step, as the
    /// SDK's error string (or `Ok(())` for a successful call). The parsed
    /// response value itself is stashed in the typed `last_*` fields below.
    pub last_call_error: Option<Result<(), String>>,
    /// Parsed response payloads from the last `When` step, one slot per
    /// response shape the `Then`/`And` assertions need to inspect.
    pub last_response: Option<UnitResponse>,
}

/// The parsed response from the last `*_responses` `When` step. Each variant
/// holds exactly the typed payload the matching assertion steps read back.
#[allow(dead_code)]
#[derive(Debug)]
pub enum UnitResponse {
    PendingTransaction(algonaut::openapi_algod::models::PendingTransactionResponse),
    PendingTransactions(
        algonaut::openapi_algod::models::GetPendingTransactionsByAddress200Response,
    ),
    RawTransaction(algonaut::openapi_algod::models::RawTransaction200Response),
    Status(algonaut_model::client_responses::NodeStatus),
    Supply(algonaut_model::client_responses::Supply),
    Account(algonaut::openapi_algod::models::Account),
    Block(algonaut::openapi_algod::ext::block::BlockResponse),
    TransactionParams(algonaut_model::client_responses::SuggestedParams),
    Dryrun(algonaut::openapi_algod::models::TealDryrun200Response),
    AssetBalances(algonaut::openapi_indexer::models::LookupAssetBalances200Response),
    Transactions(algonaut::openapi_indexer::models::LookupAccountTransactions200Response),
    IndexerBlock(algonaut::openapi_indexer::models::Block),
    IndexerAccount(algonaut::openapi_indexer::models::LookupAccountById200Response),
    IndexerAsset(algonaut::openapi_indexer::models::LookupAssetById200Response),
    Accounts(algonaut::openapi_indexer::models::SearchForAccounts200Response),
    BlockHeaders(algonaut::openapi_indexer::models::SearchForBlockHeaders200Response),
    /// `search_for_assets` reuses the `LookupAccountCreatedAssets` shape.
    SearchAssets(algonaut::openapi_indexer::models::LookupAccountCreatedAssets200Response),
}
