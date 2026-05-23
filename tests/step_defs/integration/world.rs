use algonaut::{
    algod::v2::Algod,
    atomic::{
        AbiArgValue, AtomicGroupBuilder, ExecuteOutcome, SignedAtomicGroup, TransactionWithSigner,
        UnsignedAtomicGroup,
    },
    indexer::v2::Indexer,
    kmd::v1::Kmd,
    simulate::SimulateResponse,
};
use algonaut_abi::{abi_interactions::AbiMethod, abi_type::AbiType, sourcemap::SourceMap};
use algonaut_core::{Address, AppId, AssetId, MultisigAddress, TransactionId};
use algonaut_crypto::Ed25519PublicKey;
use algonaut_model::algod::SuggestedParams;
use algonaut_model::algod::{
    AssetParams, DryrunResponse, SimulateRequest, SimulateTransactionResponse,
};
use algonaut_transaction::{
    SignedTransaction, Signer, Transaction,
    account::Account,
    auction::{Bid, SignedBid},
};
use cucumber;
use std::sync::Arc;

#[derive(Default, Debug, cucumber::World)]
pub struct World {
    pub algod: Option<Algod>,
    pub indexer: Option<Indexer>,

    pub kmd: Option<Kmd>,
    pub handle: Option<String>,
    pub password: Option<String>,
    pub accounts: Option<Vec<Address>>,

    pub transient_account: Option<Account>,

    pub tx: Option<Transaction>,
    pub transaction_id: Option<TransactionId>,

    pub app_id: Option<AppId>,
    pub app_ids: Vec<AppId>,

    pub suggested_params: Option<SuggestedParams>,

    pub note: Option<Vec<u8>>,

    pub tx_signer: Option<Arc<dyn Signer>>,
    pub tx_with_signer: Option<TransactionWithSigner>,
    pub group_builder: Option<AtomicGroupBuilder>,
    pub unsigned_group: Option<UnsignedAtomicGroup>,
    pub signed_group: Option<SignedAtomicGroup>,
    pub tx_composer_methods: Option<Vec<AbiMethod>>,
    pub signed_txs: Option<Vec<SignedTransaction>>,
    pub abi_method: Option<AbiMethod>,
    pub abi_method_arg_types: Option<Vec<AbiType>>,
    pub abi_method_arg_values: Option<Vec<AbiArgValue>>,
    pub tx_composer_res: Option<ExecuteOutcome>,

    pub versions: Option<Vec<String>>,

    pub compile_status: Option<u16>,
    pub compile_result: Option<String>,
    pub compile_hash: Option<String>,
    pub compiled_program: Option<Vec<u8>>,
    pub compiled_sourcemap: Option<SourceMap>,

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

    pub asset_id: Option<AssetId>,
    pub asset_info: Option<AssetParams>,
    pub expected_asset_params: Option<AssetParams>,
    pub asset_creator: Option<Address>,
    pub asset_second: Option<Address>,

    pub dryrun_response: Option<DryrunResponse>,
    /// The kind of dryrun test: "lsig", "approv", or "clearp".
    pub dryrun_kind: Option<String>,

    pub simulate_request: Option<SimulateRequest>,
    /// Raw response from a direct `algod.simulate` call; the deep
    /// wire-level assertions (exec traces, state changes) read this.
    pub simulate_response: Option<SimulateTransactionResponse>,
    /// Typed composer simulate result, from the `UnsignedAtomicGroup`
    /// simulate path.
    pub simulate_outcome: Option<SimulateResponse>,
    pub simulate_unsigned: bool,
}

impl World {
    /// Take the staged group as an [`UnsignedAtomicGroup`], building it from the
    /// [`AtomicGroupBuilder`] if `build` hasn't been called yet.
    pub fn take_unsigned_group(&mut self) -> UnsignedAtomicGroup {
        if let Some(unsigned) = self.unsigned_group.take() {
            unsigned
        } else {
            self.group_builder
                .take()
                .expect("no composer in progress")
                .build()
                .expect("group build failed")
        }
    }

    /// Take the staged group as a [`SignedAtomicGroup`], building and signing as
    /// needed.
    pub async fn take_signed_group(&mut self) -> SignedAtomicGroup {
        if let Some(signed) = self.signed_group.take() {
            signed
        } else {
            self.take_unsigned_group()
                .sign()
                .await
                .expect("signing failed")
        }
    }
}
