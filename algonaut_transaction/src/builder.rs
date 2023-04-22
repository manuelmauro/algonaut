use crate::{
    error::TransactionError,
    transaction::{
        ApplicationCallOnComplete, ApplicationCallTransaction, AssetAcceptTransaction,
        AssetClawbackTransaction, AssetConfigurationTransaction, AssetFreezeTransaction,
        AssetParams, AssetTransferTransaction, KeyRegistration, Payment, StateSchema, Transaction,
        TransactionType,
    },
};
use algonaut_algod::models::TransactionParams200Response;
use algonaut_core::{Address, CompiledTeal, MicroAlgos, Round, VotePk, VrfPk};
use algonaut_crypto::HashDigest;

/// A builder for [Transaction].
pub struct TxnBuilder {
    fee: MicroAlgos,
    first_valid: Round,
    genesis_hash: HashDigest,
    last_valid: Round,
    txn_type: TransactionType,
    genesis_id: Option<String>,
    group: Option<HashDigest>,
    lease: Option<HashDigest>,
    note: Option<Vec<u8>>,
    rekey_to: Option<Address>,
}

impl TxnBuilder {
    /// Convenience to initialize builder with suggested transaction params
    ///
    /// The txn fee is estimated, based on params. To set the fee manually, use [with_fee](Self::with_fee) or [new](Self::new).
    pub fn with(params: &TransactionParams200Response, txn_type: TransactionType) -> Self {
        Self::with_fee(params, MicroAlgos(params.min_fee), txn_type)
    }

    /// Convenience to initialize builder with suggested transaction params, and set the fee manually (ignoring the fee fields in params).
    ///
    /// Useful e.g. in txns groups where one txn pays the fee for others.
    pub fn with_fee(
        params: &TransactionParams200Response,
        fee: MicroAlgos,
        txn_type: TransactionType,
    ) -> Self {
        Self::new(
            fee,
            Round(params.last_round),
            Round(params.last_round + 1000),
            params.genesis_hash,
            txn_type,
        )
        .genesis_id(params.genesis_id.clone())
    }

    pub fn new(
        fee: MicroAlgos,
        first_valid: Round,
        last_valid: Round,
        genesis_hash: HashDigest,
        txn_type: TransactionType,
    ) -> Self {
        TxnBuilder {
            fee,
            first_valid,
            genesis_hash,
            last_valid,
            txn_type,
            genesis_id: None,
            group: None,
            lease: None,
            note: None,
            rekey_to: None,
        }
    }

    pub fn genesis_id(mut self, id: String) -> Self {
        self.genesis_id = Some(id);
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

    pub fn build(self) -> Result<Transaction, TransactionError> {
        Ok(self.build_tx(self.fee))
    }

    fn build_tx(&self, fee: MicroAlgos) -> Transaction {
        Transaction {
            fee,
            first_valid: self.first_valid,
            genesis_hash: self.genesis_hash,
            last_valid: self.last_valid,
            txn_type: self.txn_type.clone(),
            genesis_id: self.genesis_id.clone(),
            group: self.group,
            lease: self.lease,
            note: self.note.clone(),
            rekey_to: self.rekey_to,
        }
    }
}

/// A builder for [Payment].
pub struct Pay {
    sender: Address,
    receiver: Address,
    amount: MicroAlgos,
    close_remainder_to: Option<Address>,
}

impl Pay {
    pub fn new(sender: Address, receiver: Address, amount: MicroAlgos) -> Self {
        Pay {
            sender,
            receiver,
            amount,
            close_remainder_to: None,
        }
    }

    pub fn close_remainder_to(mut self, close_remainder_to: Address) -> Self {
        self.close_remainder_to = Some(close_remainder_to);
        self
    }

    pub fn build(self) -> TransactionType {
        TransactionType::Payment(Payment {
            sender: self.sender,
            receiver: self.receiver,
            amount: self.amount,
            close_remainder_to: self.close_remainder_to,
        })
    }
}

/// A builder for [KeyRegistration].
pub struct RegisterKey {
    sender: Address,
    vote_pk: Option<VotePk>,
    selection_pk: Option<VrfPk>,
    vote_first: Option<Round>,
    vote_last: Option<Round>,
    vote_key_dilution: Option<u64>,
    nonparticipating: Option<bool>,
}

impl RegisterKey {
    pub fn online(
        sender: Address,
        vote_pk: VotePk,
        selection_pk: VrfPk,
        vote_first: Round,
        vote_last: Round,
        vote_key_dilution: u64,
    ) -> Self {
        RegisterKey {
            sender,
            vote_pk: Some(vote_pk),
            selection_pk: Some(selection_pk),
            vote_first: Some(vote_first),
            vote_last: Some(vote_last),
            vote_key_dilution: Some(vote_key_dilution),
            nonparticipating: None,
        }
    }

    pub fn offline(sender: Address) -> Self {
        RegisterKey {
            sender,
            vote_pk: None,
            selection_pk: None,
            vote_first: None,
            vote_last: None,
            vote_key_dilution: None,
            nonparticipating: None,
        }
    }

    pub fn nonpartipating(sender: Address, nonparticipating: bool) -> Self {
        RegisterKey {
            sender,
            vote_pk: None,
            selection_pk: None,
            vote_first: None,
            vote_last: None,
            vote_key_dilution: None,
            nonparticipating: Some(nonparticipating),
        }
    }

    pub fn build(self) -> TransactionType {
        TransactionType::KeyRegistration(KeyRegistration {
            sender: self.sender,
            vote_pk: self.vote_pk,
            selection_pk: self.selection_pk,
            vote_first: self.vote_first,
            vote_last: self.vote_last,
            vote_key_dilution: self.vote_key_dilution,
            nonparticipating: self.nonparticipating,
        })
    }
}

/// A builder for [AssetConfigurationTransaction].
pub struct CreateAsset {
    sender: Address,
    total: Option<u64>,
    decimals: Option<u32>,
    default_frozen: Option<bool>,
    unit_name: Option<String>,
    asset_name: Option<String>,
    url: Option<String>,
    meta_data_hash: Option<Vec<u8>>,
    manager: Option<Address>,
    reserve: Option<Address>,
    freeze: Option<Address>,
    clawback: Option<Address>,
}

impl CreateAsset {
    pub fn new(sender: Address, total: u64, decimals: u32, default_frozen: bool) -> Self {
        CreateAsset {
            sender,
            total: Some(total),
            decimals: Some(decimals),
            default_frozen: Some(default_frozen),
            unit_name: None,
            asset_name: None,
            url: None,
            meta_data_hash: None,
            manager: None,
            reserve: None,
            freeze: None,
            clawback: None,
        }
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

    pub fn build(self) -> TransactionType {
        TransactionType::AssetConfigurationTransaction(AssetConfigurationTransaction {
            sender: self.sender,
            config_asset: None,
            params: Some(AssetParams {
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
            }),
        })
    }
}

/// A builder for [AssetConfigurationTransaction].
pub struct UpdateAsset {
    sender: Address,
    asset_id: u64,
    total: Option<u64>,
    decimals: Option<u32>,
    default_frozen: Option<bool>,
    unit_name: Option<String>,
    asset_name: Option<String>,
    url: Option<String>,
    meta_data_hash: Option<Vec<u8>>,
    manager: Option<Address>,
    reserve: Option<Address>,
    freeze: Option<Address>,
    clawback: Option<Address>,
}

impl UpdateAsset {
    pub fn new(sender: Address, asset_id: u64) -> Self {
        UpdateAsset {
            sender,
            asset_id,
            total: None,
            decimals: None,
            default_frozen: None,
            unit_name: None,
            asset_name: None,
            url: None,
            meta_data_hash: None,
            manager: None,
            reserve: None,
            freeze: None,
            clawback: None,
        }
    }

    pub fn total(mut self, total: u64) -> Self {
        self.total = Some(total);
        self
    }

    pub fn decimals(mut self, decimals: u32) -> Self {
        self.decimals = Some(decimals);
        self
    }

    pub fn default_frozen(mut self, default_frozen: bool) -> Self {
        self.default_frozen = Some(default_frozen);
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

    pub fn build(self) -> TransactionType {
        TransactionType::AssetConfigurationTransaction(AssetConfigurationTransaction {
            sender: self.sender,
            config_asset: Some(self.asset_id),
            params: Some(AssetParams {
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
            }),
        })
    }
}

/// A builder for [AssetConfigurationTransaction].
pub struct DestroyAsset {
    sender: Address,
    asset_id: u64,
}

impl DestroyAsset {
    pub fn new(sender: Address, asset_id: u64) -> Self {
        DestroyAsset { sender, asset_id }
    }

    pub fn build(self) -> TransactionType {
        TransactionType::AssetConfigurationTransaction(AssetConfigurationTransaction {
            sender: self.sender,
            config_asset: Some(self.asset_id),
            params: None,
        })
    }
}

/// A builder for [AssetTransferTransaction].
pub struct TransferAsset {
    sender: Address,
    xfer: u64,
    amount: u64,
    receiver: Address,
    close_to: Option<Address>,
}

impl TransferAsset {
    pub fn new(sender: Address, asset_id: u64, amount: u64, receiver: Address) -> Self {
        TransferAsset {
            sender,
            xfer: asset_id,
            amount,
            receiver,
            close_to: None,
        }
    }

    pub fn close_to(mut self, close_to: Address) -> Self {
        self.close_to = Some(close_to);
        self
    }

    pub fn build(self) -> TransactionType {
        TransactionType::AssetTransferTransaction(AssetTransferTransaction {
            sender: self.sender,
            xfer: self.xfer,
            amount: self.amount,
            receiver: self.receiver,
            close_to: self.close_to,
        })
    }
}

/// A builder for [AssetAcceptTransaction].
pub struct AcceptAsset {
    sender: Address,
    asset_id: u64,
}

impl AcceptAsset {
    pub fn new(sender: Address, asset_id: u64) -> Self {
        AcceptAsset { sender, asset_id }
    }

    pub fn build(self) -> TransactionType {
        TransactionType::AssetAcceptTransaction(AssetAcceptTransaction {
            sender: self.sender,
            xfer: self.asset_id,
        })
    }
}

/// A builder for [AssetClawbackTransaction].
pub struct ClawbackAsset {
    sender: Address,
    asset_id: u64,
    asset_amount: u64,
    asset_sender: Address,
    asset_receiver: Address,
    asset_close_to: Option<Address>,
}

impl ClawbackAsset {
    pub fn new(
        sender: Address,
        asset_id: u64,
        asset_amount: u64,
        asset_sender: Address,
        asset_receiver: Address,
    ) -> Self {
        ClawbackAsset {
            sender,
            asset_id,
            asset_amount,
            asset_sender,
            asset_receiver,
            asset_close_to: None,
        }
    }

    pub fn asset_close_to(mut self, asset_close_to: Address) -> Self {
        self.asset_close_to = Some(asset_close_to);
        self
    }

    pub fn build(self) -> TransactionType {
        TransactionType::AssetClawbackTransaction(AssetClawbackTransaction {
            sender: self.sender,
            xfer: self.asset_id,
            asset_amount: self.asset_amount,
            asset_sender: self.asset_sender,
            asset_receiver: self.asset_receiver,
            asset_close_to: self.asset_close_to,
        })
    }
}

/// A builder for [AssetFreezeTransaction].
pub struct FreezeAsset {
    sender: Address,
    freeze_account: Address,
    asset_id: u64,
    frozen: bool,
}

impl FreezeAsset {
    pub fn new(sender: Address, freeze_account: Address, asset_id: u64, frozen: bool) -> Self {
        FreezeAsset {
            sender,
            freeze_account,
            asset_id,
            frozen,
        }
    }

    pub fn build(self) -> TransactionType {
        TransactionType::AssetFreezeTransaction(AssetFreezeTransaction {
            sender: self.sender,
            freeze_account: self.freeze_account,
            asset_id: self.asset_id,
            frozen: self.frozen,
        })
    }
}

/// A builder for [ApplicationCallTransaction].
pub struct CreateApplication {
    sender: Address,
    accounts: Option<Vec<Address>>,
    approval_program: Option<CompiledTeal>,
    app_arguments: Option<Vec<Vec<u8>>>,
    clear_state_program: Option<CompiledTeal>,
    foreign_apps: Option<Vec<u64>>,
    foreign_assets: Option<Vec<u64>>,
    global_state_schema: Option<StateSchema>,
    local_state_schema: Option<StateSchema>,
    extra_pages: u32,
}

impl CreateApplication {
    pub fn new(
        sender: Address,
        approval_program: CompiledTeal,
        clear_state_program: CompiledTeal,
        global_state_schema: StateSchema,
        local_state_schema: StateSchema,
    ) -> Self {
        CreateApplication {
            sender,
            accounts: None,
            approval_program: Some(approval_program),
            app_arguments: None,
            clear_state_program: Some(clear_state_program),
            foreign_apps: None,
            foreign_assets: None,
            global_state_schema: Some(global_state_schema),
            local_state_schema: Some(local_state_schema),
            extra_pages: 0,
        }
    }

    pub fn accounts(mut self, accounts: Vec<Address>) -> Self {
        self.accounts = Some(accounts);
        self
    }

    pub fn app_arguments(mut self, app_arguments: Vec<Vec<u8>>) -> Self {
        self.app_arguments = Some(app_arguments);
        self
    }

    pub fn foreign_apps(mut self, foreign_apps: Vec<u64>) -> Self {
        self.foreign_apps = Some(foreign_apps);
        self
    }

    pub fn foreign_assets(mut self, foreign_assets: Vec<u64>) -> Self {
        self.foreign_assets = Some(foreign_assets);
        self
    }

    pub fn extra_pages(mut self, extra_pages: u32) -> Self {
        self.extra_pages = extra_pages;
        self
    }

    pub fn build(self) -> TransactionType {
        TransactionType::ApplicationCallTransaction(ApplicationCallTransaction {
            sender: self.sender,
            app_id: None,
            on_complete: ApplicationCallOnComplete::NoOp,
            accounts: self.accounts,
            approval_program: self.approval_program,
            app_arguments: self.app_arguments,
            clear_state_program: self.clear_state_program,
            foreign_apps: self.foreign_apps,
            foreign_assets: self.foreign_assets,
            global_state_schema: self.global_state_schema,
            local_state_schema: self.local_state_schema,
            extra_pages: self.extra_pages,
        })
    }
}

/// A builder for [ApplicationCallTransaction].
pub struct UpdateApplication {
    sender: Address,
    app_id: u64,
    accounts: Option<Vec<Address>>,
    approval_program: Option<CompiledTeal>,
    app_arguments: Option<Vec<Vec<u8>>>,
    clear_state_program: Option<CompiledTeal>,
    foreign_apps: Option<Vec<u64>>,
    foreign_assets: Option<Vec<u64>>,
}

impl UpdateApplication {
    pub fn new(
        sender: Address,
        app_id: u64,
        approval_program: CompiledTeal,
        clear_state_program: CompiledTeal,
    ) -> Self {
        UpdateApplication {
            sender,
            app_id,
            accounts: None,
            approval_program: Some(approval_program),
            app_arguments: None,
            clear_state_program: Some(clear_state_program),
            foreign_apps: None,
            foreign_assets: None,
        }
    }

    pub fn accounts(mut self, accounts: Vec<Address>) -> Self {
        self.accounts = Some(accounts);
        self
    }

    pub fn app_arguments(mut self, app_arguments: Vec<Vec<u8>>) -> Self {
        self.app_arguments = Some(app_arguments);
        self
    }

    pub fn foreign_apps(mut self, foreign_apps: Vec<u64>) -> Self {
        self.foreign_apps = Some(foreign_apps);
        self
    }

    pub fn foreign_assets(mut self, foreign_assets: Vec<u64>) -> Self {
        self.foreign_assets = Some(foreign_assets);
        self
    }

    pub fn build(self) -> TransactionType {
        TransactionType::ApplicationCallTransaction(ApplicationCallTransaction {
            sender: self.sender,
            app_id: Some(self.app_id),
            on_complete: ApplicationCallOnComplete::UpdateApplication,
            accounts: self.accounts,
            approval_program: self.approval_program,
            app_arguments: self.app_arguments,
            clear_state_program: self.clear_state_program,
            foreign_apps: self.foreign_apps,
            foreign_assets: self.foreign_assets,
            global_state_schema: None,
            local_state_schema: None,
            extra_pages: 0,
        })
    }
}

/// A builder for [ApplicationCallTransaction].
pub struct CallApplication {
    sender: Address,
    app_id: u64,
    accounts: Option<Vec<Address>>,
    app_arguments: Option<Vec<Vec<u8>>>,
    foreign_apps: Option<Vec<u64>>,
    foreign_assets: Option<Vec<u64>>,
}

impl CallApplication {
    pub fn new(sender: Address, app_id: u64) -> Self {
        CallApplication {
            sender,
            app_id,
            accounts: None,
            app_arguments: None,
            foreign_apps: None,
            foreign_assets: None,
        }
    }

    pub fn accounts(mut self, accounts: Vec<Address>) -> Self {
        self.accounts = Some(accounts);
        self
    }

    pub fn app_arguments(mut self, app_arguments: Vec<Vec<u8>>) -> Self {
        self.app_arguments = Some(app_arguments);
        self
    }

    pub fn foreign_apps(mut self, foreign_apps: Vec<u64>) -> Self {
        self.foreign_apps = Some(foreign_apps);
        self
    }

    pub fn foreign_assets(mut self, foreign_assets: Vec<u64>) -> Self {
        self.foreign_assets = Some(foreign_assets);
        self
    }

    pub fn build(self) -> TransactionType {
        TransactionType::ApplicationCallTransaction(ApplicationCallTransaction {
            sender: self.sender,
            app_id: Some(self.app_id),
            on_complete: ApplicationCallOnComplete::NoOp,
            accounts: self.accounts,
            approval_program: None,
            app_arguments: self.app_arguments,
            clear_state_program: None,
            foreign_apps: self.foreign_apps,
            foreign_assets: self.foreign_assets,
            global_state_schema: None,
            local_state_schema: None,
            extra_pages: 0,
        })
    }
}

/// A builder for [ApplicationCallTransaction].
pub struct ClearApplication {
    sender: Address,
    app_id: u64,
    accounts: Option<Vec<Address>>,
    app_arguments: Option<Vec<Vec<u8>>>,
    foreign_apps: Option<Vec<u64>>,
    foreign_assets: Option<Vec<u64>>,
}

impl ClearApplication {
    pub fn new(sender: Address, app_id: u64) -> Self {
        ClearApplication {
            sender,
            app_id,
            accounts: None,
            app_arguments: None,
            foreign_apps: None,
            foreign_assets: None,
        }
    }

    pub fn accounts(mut self, accounts: Vec<Address>) -> Self {
        self.accounts = Some(accounts);
        self
    }

    pub fn app_arguments(mut self, app_arguments: Vec<Vec<u8>>) -> Self {
        self.app_arguments = Some(app_arguments);
        self
    }

    pub fn foreign_apps(mut self, foreign_apps: Vec<u64>) -> Self {
        self.foreign_apps = Some(foreign_apps);
        self
    }

    pub fn foreign_assets(mut self, foreign_assets: Vec<u64>) -> Self {
        self.foreign_assets = Some(foreign_assets);
        self
    }

    pub fn build(self) -> TransactionType {
        TransactionType::ApplicationCallTransaction(ApplicationCallTransaction {
            sender: self.sender,
            app_id: Some(self.app_id),
            on_complete: ApplicationCallOnComplete::ClearState,
            accounts: self.accounts,
            approval_program: None,
            app_arguments: self.app_arguments,
            clear_state_program: None,
            foreign_apps: self.foreign_apps,
            foreign_assets: self.foreign_assets,
            global_state_schema: None,
            local_state_schema: None,
            extra_pages: 0,
        })
    }
}

/// A builder for [ApplicationCallTransaction].
pub struct CloseApplication {
    sender: Address,
    app_id: u64,
    accounts: Option<Vec<Address>>,
    app_arguments: Option<Vec<Vec<u8>>>,
    foreign_apps: Option<Vec<u64>>,
    foreign_assets: Option<Vec<u64>>,
}

impl CloseApplication {
    pub fn new(sender: Address, app_id: u64) -> Self {
        CloseApplication {
            sender,
            app_id,
            accounts: None,
            app_arguments: None,
            foreign_apps: None,
            foreign_assets: None,
        }
    }

    pub fn accounts(mut self, accounts: Vec<Address>) -> Self {
        self.accounts = Some(accounts);
        self
    }

    pub fn app_arguments(mut self, app_arguments: Vec<Vec<u8>>) -> Self {
        self.app_arguments = Some(app_arguments);
        self
    }

    pub fn foreign_apps(mut self, foreign_apps: Vec<u64>) -> Self {
        self.foreign_apps = Some(foreign_apps);
        self
    }

    pub fn foreign_assets(mut self, foreign_assets: Vec<u64>) -> Self {
        self.foreign_assets = Some(foreign_assets);
        self
    }

    pub fn build(self) -> TransactionType {
        TransactionType::ApplicationCallTransaction(ApplicationCallTransaction {
            sender: self.sender,
            app_id: Some(self.app_id),
            on_complete: ApplicationCallOnComplete::CloseOut,
            accounts: self.accounts,
            approval_program: None,
            app_arguments: self.app_arguments,
            clear_state_program: None,
            foreign_apps: self.foreign_apps,
            foreign_assets: self.foreign_assets,
            global_state_schema: None,
            local_state_schema: None,
            extra_pages: 0,
        })
    }
}

/// A builder for [ApplicationCallTransaction].
pub struct DeleteApplication {
    sender: Address,
    app_id: u64,
    accounts: Option<Vec<Address>>,
    app_arguments: Option<Vec<Vec<u8>>>,
    foreign_apps: Option<Vec<u64>>,
    foreign_assets: Option<Vec<u64>>,
}

impl DeleteApplication {
    pub fn new(sender: Address, app_id: u64) -> Self {
        DeleteApplication {
            sender,
            app_id,
            accounts: None,
            app_arguments: None,
            foreign_apps: None,
            foreign_assets: None,
        }
    }

    pub fn accounts(mut self, accounts: Vec<Address>) -> Self {
        self.accounts = Some(accounts);
        self
    }

    pub fn app_arguments(mut self, app_arguments: Vec<Vec<u8>>) -> Self {
        self.app_arguments = Some(app_arguments);
        self
    }

    pub fn foreign_apps(mut self, foreign_apps: Vec<u64>) -> Self {
        self.foreign_apps = Some(foreign_apps);
        self
    }

    pub fn foreign_assets(mut self, foreign_assets: Vec<u64>) -> Self {
        self.foreign_assets = Some(foreign_assets);
        self
    }

    pub fn build(self) -> TransactionType {
        TransactionType::ApplicationCallTransaction(ApplicationCallTransaction {
            sender: self.sender,
            app_id: Some(self.app_id),
            on_complete: ApplicationCallOnComplete::DeleteApplication,
            accounts: self.accounts,
            approval_program: None,
            app_arguments: self.app_arguments,
            clear_state_program: None,
            foreign_apps: self.foreign_apps,
            foreign_assets: self.foreign_assets,
            global_state_schema: None,
            local_state_schema: None,
            extra_pages: 0,
        })
    }
}

/// A builder for [ApplicationCallTransaction].
pub struct OptInApplication {
    sender: Address,
    app_id: u64,
    accounts: Option<Vec<Address>>,
    app_arguments: Option<Vec<Vec<u8>>>,
    foreign_apps: Option<Vec<u64>>,
    foreign_assets: Option<Vec<u64>>,
}

impl OptInApplication {
    pub fn new(sender: Address, app_id: u64) -> Self {
        OptInApplication {
            sender,
            app_id,
            accounts: None,
            app_arguments: None,
            foreign_apps: None,
            foreign_assets: None,
        }
    }

    pub fn accounts(mut self, accounts: Vec<Address>) -> Self {
        self.accounts = Some(accounts);
        self
    }

    pub fn app_arguments(mut self, app_arguments: Vec<Vec<u8>>) -> Self {
        self.app_arguments = Some(app_arguments);
        self
    }

    pub fn foreign_apps(mut self, foreign_apps: Vec<u64>) -> Self {
        self.foreign_apps = Some(foreign_apps);
        self
    }

    pub fn foreign_assets(mut self, foreign_assets: Vec<u64>) -> Self {
        self.foreign_assets = Some(foreign_assets);
        self
    }

    pub fn build(self) -> TransactionType {
        TransactionType::ApplicationCallTransaction(ApplicationCallTransaction {
            sender: self.sender,
            app_id: Some(self.app_id),
            on_complete: ApplicationCallOnComplete::OptIn,
            accounts: self.accounts,
            approval_program: None,
            app_arguments: self.app_arguments,
            clear_state_program: None,
            foreign_apps: self.foreign_apps,
            foreign_assets: self.foreign_assets,
            global_state_schema: None,
            local_state_schema: None,
            extra_pages: 0,
        })
    }
}
