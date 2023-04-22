use algonaut::{
    algod::v2::Algod,
    atomic_transaction_composer::AtomicTransactionComposer,
    atomic_transaction_composer::{
        transaction_signer::TransactionSigner, AbiArgValue, ExecuteResult, TransactionWithSigner,
    },
    kmd::v1::Kmd,
};
use algonaut_abi::{abi_interactions::AbiMethod, abi_type::AbiType};
use algonaut_algod::models::TransactionParams200Response;
use algonaut_core::Address;
use algonaut_transaction::{account::Account, SignedTransaction, Transaction};
use cucumber;

#[derive(Default, Debug, cucumber::World)]
pub struct World {
    pub algod: Option<Algod>,

    pub kmd: Option<Kmd>,
    pub handle: Option<String>,
    pub password: Option<String>,
    pub accounts: Option<Vec<Address>>,

    pub transient_account: Option<Account>,

    pub tx: Option<Transaction>,
    pub tx_id: Option<String>,

    pub app_id: Option<u64>,
    pub app_ids: Vec<u64>,

    pub tx_params: Option<TransactionParams200Response>,

    pub note: Option<Vec<u8>>,

    pub tx_signer: Option<TransactionSigner>,
    pub tx_with_signer: Option<TransactionWithSigner>,
    pub tx_composer: Option<AtomicTransactionComposer>,
    pub tx_composer_methods: Option<Vec<AbiMethod>>,
    pub signed_txs: Option<Vec<SignedTransaction>>,
    pub abi_method: Option<AbiMethod>,
    pub abi_method_arg_types: Option<Vec<AbiType>>,
    pub abi_method_arg_values: Option<Vec<AbiArgValue>>,
    pub tx_composer_res: Option<ExecuteResult>,
}
