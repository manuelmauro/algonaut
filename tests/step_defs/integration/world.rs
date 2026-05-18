use algonaut::{
    algod::v2::Algod,
    atomic_transaction_composer::AtomicTransactionComposer,
    atomic_transaction_composer::{
        AbiArgValue, ExecuteResult, TransactionWithSigner, transaction_signer::TransactionSigner,
    },
    kmd::v1::Kmd,
};
use algonaut_abi::{abi_interactions::AbiMethod, abi_type::AbiType};
use algonaut_algod::models::TransactionParams200Response;
use algonaut_core::{Address, MultisigAddress};
use algonaut_crypto::Ed25519PublicKey;
use algonaut_transaction::{
    SignedTransaction, Transaction,
    account::Account,
    auction::{Bid, SignedBid},
};
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

    pub versions: Option<Vec<String>>,

    pub compile_status: Option<u16>,
    pub compile_result: Option<String>,
    pub compile_hash: Option<String>,
    pub compiled_program: Option<Vec<u8>>,

    pub sender_account: Option<Account>,
    pub signed_tx: Option<SignedTransaction>,
    pub multisig: Option<MultisigAddress>,
    pub last_send_succeeded: Option<bool>,

    pub rekey_target: Option<Address>,

    pub bid: Option<Bid>,
    pub signed_bid: Option<SignedBid>,
    pub signed_bid_roundtrip: Option<SignedBid>,

    pub generated_account: Option<Account>,
    pub generated_kmd_address: Option<Address>,
    pub created_wallet_id: Option<String>,
    pub created_wallet_handle: Option<String>,
    pub created_wallet_name: Option<String>,
    pub kmd_signed_tx_bytes: Option<Vec<u8>>,
    pub kmd_signed_multisig_bytes: Option<Vec<u8>>,
    pub exported_multisig_pks: Option<Vec<Ed25519PublicKey>>,
}
