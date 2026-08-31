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
    PendingTransaction(algonaut::model::algod::PendingTransactionResponse),
    PendingTransactions(algonaut::model::algod::PendingTransactions),
    RawTransaction(algonaut::model::algod::SubmitResponse),
    Status(algonaut::model::algod::NodeStatus),
    Supply(algonaut::model::algod::Supply),
    Account(algonaut::model::algod::Account),
    Block(algonaut::model::algod::ext::block::BlockResponse),
    TransactionParams(algonaut_model::algod::SuggestedParams),
    AssetBalances(algonaut::model::indexer::AssetBalancesResponse),
    Transactions(algonaut::model::indexer::AccountTransactionsResponse),
    IndexerBlock(algonaut::model::indexer::Block),
    IndexerAccount(algonaut::model::indexer::AccountResponse),
    IndexerAsset(algonaut::model::indexer::AssetResponse),
    Accounts(algonaut::model::indexer::AccountsResponse),
    BlockHeaders(algonaut::model::indexer::BlockHeadersResponse),
    /// `search_for_assets` reuses the `LookupAccountCreatedAssets` shape.
    SearchAssets(algonaut::model::indexer::AccountCreatedAssetsResponse),
}
