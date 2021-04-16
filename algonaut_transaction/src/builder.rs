use crate::transaction::{
    AssetConfigurationTransaction, AssetParams, KeyRegistration, Payment, Transaction,
    TransactionType,
};
use algonaut_core::{Address, MicroAlgos, Round, VotePk, VrfPk};
use algonaut_crypto::HashDigest;

#[derive(Default)]
pub struct Txn {
    fee: MicroAlgos,
    first_valid: Round,
    genesis_hash: Option<HashDigest>,
    last_valid: Round,
    sender: Option<Address>,
    txn_type: Option<TransactionType>,
    genesis_id: String,
    group: Option<HashDigest>,
    lease: Option<HashDigest>,
    note: Vec<u8>,
    rekey_to: Option<Address>,
}

impl Txn {
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

    pub fn txn_type(mut self, txn_type: TransactionType) -> Self {
        self.txn_type = Some(txn_type);
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
        self.note = note;
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

#[derive(Default)]
pub struct ConfigureAsset {
    config_asset: u64,
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
        self.config_asset = config_asset;
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
