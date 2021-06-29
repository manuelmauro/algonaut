use crate::transaction::{
    ApplicationCallTransaction, AssetAcceptTransaction, AssetClawbackTransaction,
    AssetConfigurationTransaction, AssetFreezeTransaction, AssetParams, AssetTransferTransaction,
    KeyRegistration, Payment, StateSchema, Transaction, TransactionType,
};
use algonaut_core::{Address, MicroAlgos, Round, VotePk, VrfPk};
use algonaut_crypto::HashDigest;

/// A builder for [Transaction].
#[derive(Default)]
pub struct TxnBuilder {
    fee: MicroAlgos,
    first_valid: Round,
    genesis_hash: Option<HashDigest>,
    last_valid: Round,
    sender: Option<Address>,
    txn_type: Option<TransactionType>,
    genesis_id: String,
    group: Option<HashDigest>,
    lease: Option<HashDigest>,
    note: Option<Vec<u8>>,
    rekey_to: Option<Address>,
}

impl TxnBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fee(mut self, fee: MicroAlgos) -> Self {
        self.fee = fee;
        self
    }

    pub fn first_valid(mut self, first_valid: Round) -> Self {
        self.first_valid = first_valid;
        self
    }

    pub fn genesis_hash(mut self, genesis_hash: HashDigest) -> Self {
        self.genesis_hash = Some(genesis_hash);
        self
    }

    pub fn last_valid(mut self, last_valid: Round) -> Self {
        self.last_valid = last_valid;
        self
    }

    pub fn sender(mut self, sender: Address) -> Self {
        self.sender = Some(sender);
        self
    }

    pub fn payment(mut self, txn: Payment) -> Self {
        self.txn_type = Some(TransactionType::Payment(txn));
        self
    }

    pub fn key_registration(mut self, txn: KeyRegistration) -> Self {
        self.txn_type = Some(TransactionType::KeyRegistration(txn));
        self
    }

    pub fn asset_configuration(mut self, txn: AssetConfigurationTransaction) -> Self {
        self.txn_type = Some(TransactionType::AssetConfigurationTransaction(txn));
        self
    }

    pub fn asset_transfer(mut self, txn: AssetTransferTransaction) -> Self {
        self.txn_type = Some(TransactionType::AssetTransferTransaction(txn));
        self
    }

    pub fn asset_accept(mut self, txn: AssetAcceptTransaction) -> Self {
        self.txn_type = Some(TransactionType::AssetAcceptTransaction(txn));
        self
    }

    pub fn asset_clawback(mut self, txn: AssetClawbackTransaction) -> Self {
        self.txn_type = Some(TransactionType::AssetClawbackTransaction(txn));
        self
    }

    pub fn asset_freeze(mut self, txn: AssetFreezeTransaction) -> Self {
        self.txn_type = Some(TransactionType::AssetFreezeTransaction(txn));
        self
    }

    pub fn application_call(mut self, txn: ApplicationCallTransaction) -> Self {
        self.txn_type = Some(TransactionType::ApplicationCallTransaction(txn));
        self
    }

    pub fn genesis_id(mut self, genesis_id: String) -> Self {
        self.genesis_id = genesis_id;
        self
    }

    pub fn group(mut self, group: HashDigest) -> Self {
        self.group = Some(group);
        self
    }

    pub fn lease(mut self, lease: HashDigest) -> Self {
        self.lease = Some(lease);
        self
    }

    pub fn note(mut self, note: Vec<u8>) -> Self {
        self.note = Some(note);
        self
    }

    pub fn rekey_to(mut self, rekey_to: Address) -> Self {
        self.rekey_to = Some(rekey_to);
        self
    }

    pub fn build(self) -> Transaction {
        Transaction {
            fee: self.fee,
            first_valid: self.first_valid,
            genesis_hash: self.genesis_hash.unwrap(),
            last_valid: self.last_valid,
            sender: self.sender.unwrap(),
            txn_type: self.txn_type.unwrap(),
            genesis_id: self.genesis_id,
            group: self.group,
            lease: self.lease,
            note: self.note,
            rekey_to: self.rekey_to,
        }
    }
}

/// A builder for [Payment].
#[derive(Default)]
pub struct Pay {
    receiver: Option<Address>,
    amount: MicroAlgos,
    close_remainder_to: Option<Address>,
}

impl Pay {
    pub fn new() -> Self {
        Pay::default()
    }

    pub fn to(mut self, receiver: Address) -> Self {
        self.receiver = Some(receiver);
        self
    }

    pub fn amount(mut self, amount: MicroAlgos) -> Self {
        self.amount = amount;
        self
    }

    pub fn close_remainder_to(mut self, close_remainder_to: Address) -> Self {
        self.close_remainder_to = Some(close_remainder_to);
        self
    }

    pub fn build(self) -> Payment {
        Payment {
            receiver: self.receiver.unwrap(),
            amount: self.amount,
            close_remainder_to: self.close_remainder_to,
        }
    }
}

/// A builder for [KeyRegistration].
#[derive(Default)]
pub struct RegisterKey {
    vote_pk: Option<VotePk>,
    selection_pk: Option<VrfPk>,
    vote_first: Round,
    vote_last: Round,
    vote_key_dilution: u64,
    nonparticipating: Option<bool>,
}

impl RegisterKey {
    pub fn new() -> Self {
        RegisterKey::default()
    }

    pub fn vote_pk(mut self, vote_pk: VotePk) -> Self {
        self.vote_pk = Some(vote_pk);
        self
    }

    pub fn selection_pk(mut self, selection_pk: VrfPk) -> Self {
        self.selection_pk = Some(selection_pk);
        self
    }

    pub fn vote_first(mut self, vote_first: Round) -> Self {
        self.vote_first = vote_first;
        self
    }

    pub fn vote_last(mut self, vote_last: Round) -> Self {
        self.vote_last = vote_last;
        self
    }

    pub fn vote_key_dilution(mut self, vote_key_dilution: u64) -> Self {
        self.vote_key_dilution = vote_key_dilution;
        self
    }

    pub fn nonparticipating(mut self, nonparticipating: Option<bool>) -> Self {
        self.nonparticipating = nonparticipating;
        self
    }

    pub fn build(self) -> KeyRegistration {
        KeyRegistration {
            vote_pk: self.vote_pk.unwrap(),
            selection_pk: self.selection_pk.unwrap(),
            vote_first: self.vote_first,
            vote_last: self.vote_last,
            vote_key_dilution: self.vote_key_dilution,
            nonparticipating: self.nonparticipating,
        }
    }
}

/// A builder for [AssetConfigurationTransaction].
#[derive(Default)]
pub struct ConfigureAsset {
    config_asset: Option<u64>,
    total: u64,
    decimals: u32,
    default_frozen: bool,
    unit_name: Option<String>,
    asset_name: Option<String>,
    url: Option<String>,
    meta_data_hash: Option<Vec<u8>>,
    manager: Option<Address>,
    reserve: Option<Address>,
    freeze: Option<Address>,
    clawback: Option<Address>,
}

impl ConfigureAsset {
    pub fn new() -> Self {
        ConfigureAsset::default()
    }

    pub fn config_asset(mut self, config_asset: u64) -> Self {
        self.config_asset = Some(config_asset);
        self
    }

    pub fn total(mut self, total: u64) -> Self {
        self.total = total;
        self
    }

    pub fn decimals(mut self, decimals: u32) -> Self {
        self.decimals = decimals;
        self
    }

    pub fn default_frozen(mut self, default_frozen: bool) -> Self {
        self.default_frozen = default_frozen;
        self
    }

    pub fn unit_name(mut self, unit_name: String) -> Self {
        self.unit_name = Some(unit_name);
        self
    }

    pub fn asset_name(mut self, asset_name: String) -> Self {
        self.asset_name = Some(asset_name);
        self
    }

    pub fn url(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }

    pub fn meta_data_hash(mut self, meta_data_hash: Vec<u8>) -> Self {
        self.meta_data_hash = Some(meta_data_hash);
        self
    }

    pub fn manager(mut self, manager: Address) -> Self {
        self.manager = Some(manager);
        self
    }

    pub fn reserve(mut self, reserve: Address) -> Self {
        self.reserve = Some(reserve);
        self
    }

    pub fn freeze(mut self, freeze: Address) -> Self {
        self.freeze = Some(freeze);
        self
    }

    pub fn clawback(mut self, clawback: Address) -> Self {
        self.clawback = Some(clawback);
        self
    }

    pub fn build(self) -> AssetConfigurationTransaction {
        AssetConfigurationTransaction {
            config_asset: self.config_asset,
            params: AssetParams {
                total: self.total,
                decimals: self.decimals,
                default_frozen: self.default_frozen,
                unit_name: self.unit_name,
                asset_name: self.asset_name,
                url: self.url,
                meta_data_hash: self.meta_data_hash,
                manager: self.manager,
                reserve: self.reserve,
                freeze: self.freeze,
                clawback: self.clawback,
            },
        }
    }
}

/// A builder for [AssetTransferTransaction].
#[derive(Default)]
pub struct TransferAsset {
    xfer: u64,
    amount: u64,
    receiver: Option<Address>,
    close_to: Option<Address>,
}

impl TransferAsset {
    pub fn new() -> Self {
        TransferAsset::default()
    }

    pub fn xfer(mut self, xfer: u64) -> Self {
        self.xfer = xfer;
        self
    }

    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = amount;
        self
    }

    pub fn receiver(mut self, receiver: Address) -> Self {
        self.receiver = Some(receiver);
        self
    }

    pub fn close_to(mut self, close_to: Address) -> Self {
        self.close_to = Some(close_to);
        self
    }

    pub fn build(self) -> AssetTransferTransaction {
        AssetTransferTransaction {
            xfer: self.xfer,
            amount: self.amount,
            receiver: self.receiver.unwrap(),
            close_to: self.close_to,
        }
    }
}

/// A builder for [AssetAcceptTransaction].
#[derive(Default)]
pub struct AcceptAsset {
    xfer: u64,
    receiver: Option<Address>,
}

impl AcceptAsset {
    pub fn new() -> Self {
        AcceptAsset::default()
    }

    pub fn xfer(mut self, xfer: u64) -> Self {
        self.xfer = xfer;
        self
    }

    pub fn receiver(mut self, receiver: Address) -> Self {
        self.receiver = Some(receiver);
        self
    }

    pub fn build(self) -> AssetAcceptTransaction {
        AssetAcceptTransaction {
            xfer: self.xfer,
            receiver: self.receiver.unwrap(),
        }
    }
}

/// A builder for [AssetClawbackTransaction].
#[derive(Default)]
pub struct ClawbackAsset {
    sender: Option<Address>,
    xfer: u64,
    asset_amount: u64,
    asset_sender: Option<Address>,
    asset_receiver: Option<Address>,
    asset_close_to: Option<Address>,
}

impl ClawbackAsset {
    pub fn new() -> Self {
        ClawbackAsset::default()
    }

    pub fn sender(mut self, sender: Address) -> Self {
        self.sender = Some(sender);
        self
    }

    pub fn xfer(mut self, xfer: u64) -> Self {
        self.xfer = xfer;
        self
    }

    pub fn asset_amount(mut self, asset_amount: u64) -> Self {
        self.asset_amount = asset_amount;
        self
    }

    pub fn asset_sender(mut self, asset_sender: Address) -> Self {
        self.asset_sender = Some(asset_sender);
        self
    }

    pub fn asset_receiver(mut self, asset_receiver: Address) -> Self {
        self.asset_receiver = Some(asset_receiver);
        self
    }

    pub fn asset_close_to(mut self, asset_close_to: Address) -> Self {
        self.asset_close_to = Some(asset_close_to);
        self
    }

    pub fn build(self) -> AssetClawbackTransaction {
        AssetClawbackTransaction {
            xfer: self.xfer,
            asset_amount: self.asset_amount,
            asset_sender: self.asset_sender.unwrap(),
            asset_receiver: self.asset_receiver.unwrap(),
            asset_close_to: self.asset_close_to.unwrap(),
        }
    }
}

/// A builder for [AssetFreezeTransaction].
#[derive(Default)]
pub struct FreezeAsset {
    freeze_account: Option<Address>,
    asset_id: u64,
    frozen: bool,
}

impl FreezeAsset {
    pub fn new() -> Self {
        FreezeAsset::default()
    }

    pub fn freeze_account(mut self, freeze_account: Address) -> Self {
        self.freeze_account = Some(freeze_account);
        self
    }

    pub fn asset_id(mut self, asset_id: u64) -> Self {
        self.asset_id = asset_id;
        self
    }

    pub fn frozen(mut self, frozen: bool) -> Self {
        self.frozen = frozen;
        self
    }

    pub fn build(self) -> AssetFreezeTransaction {
        AssetFreezeTransaction {
            freeze_account: self.freeze_account.unwrap(),
            asset_id: self.asset_id,
            frozen: self.frozen,
        }
    }
}

/// A builder for [ApplicationCallTransaction].
#[derive(Default)]
pub struct CallApplication {
    app_id: u64,
    on_complete: u64,
    accounts: Option<Vec<Address>>,
    approval_program: Option<Address>,
    app_arguments: Option<Vec<u8>>,
    clear_state_program: Option<Address>,
    foreign_apps: Option<Address>,
    foreign_assets: Option<Address>,
    global_state_schema: Option<StateSchema>,
    local_state_schema: Option<StateSchema>,
}

impl CallApplication {
    pub fn new() -> Self {
        CallApplication::default()
    }

    pub fn app_id(mut self, app_id: u64) -> Self {
        self.app_id = app_id;
        self
    }

    pub fn on_complete(mut self, on_complete: u64) -> Self {
        self.on_complete = on_complete;
        self
    }

    pub fn accounts(mut self, accounts: Vec<Address>) -> Self {
        self.accounts = Some(accounts);
        self
    }

    pub fn approval_program(mut self, approval_program: Address) -> Self {
        self.approval_program = Some(approval_program);
        self
    }

    pub fn app_arguments(mut self, app_arguments: Vec<u8>) -> Self {
        self.app_arguments = Some(app_arguments);
        self
    }

    pub fn clear_state_program(mut self, clear_state_program: Address) -> Self {
        self.clear_state_program = Some(clear_state_program);
        self
    }

    pub fn foreign_apps(mut self, foreign_apps: Address) -> Self {
        self.foreign_apps = Some(foreign_apps);
        self
    }

    pub fn foreign_assets(mut self, foreign_assets: Address) -> Self {
        self.foreign_assets = Some(foreign_assets);
        self
    }

    pub fn global_state_schema(mut self, global_state_schema: StateSchema) -> Self {
        self.global_state_schema = Some(global_state_schema);
        self
    }

    pub fn local_state_schema(mut self, local_state_schema: StateSchema) -> Self {
        self.local_state_schema = Some(local_state_schema);
        self
    }

    pub fn build(self) -> ApplicationCallTransaction {
        ApplicationCallTransaction {
            app_id: self.app_id,
            on_complete: self.on_complete,
            accounts: self.accounts,
            approval_program: self.approval_program,
            app_arguments: self.app_arguments,
            clear_state_program: self.clear_state_program,
            foreign_apps: self.foreign_apps,
            foreign_assets: self.foreign_assets,
            global_state_schema: self.global_state_schema,
            local_state_schema: self.local_state_schema,
        }
    }
}
