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
}
