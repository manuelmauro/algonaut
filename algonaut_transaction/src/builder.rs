use crate::{
    error::TransactionError,
    transaction::{
        ApplicationCallOnComplete, ApplicationCallTransaction, AssetAcceptTransaction,
        AssetClawbackTransaction, AssetConfigurationTransaction, AssetFreezeTransaction,
        AssetParams, AssetTransferTransaction, BoxReference, KeyRegistration, Payment, StateSchema,
        Transaction, TransactionType,
    },
};
use algonaut_core::{
    Address, AppId, AssetId, CompiledTeal, MicroAlgos, Round, StateProofPk, VotePk, VrfPk,
};
use algonaut_crypto::HashDigest;

pub trait TransactionParams {
    fn last_round(&self) -> u64;
    fn min_fee(&self) -> u64;
    fn genesis_hash(&self) -> HashDigest;
    fn genesis_id(&self) -> &String;
}

/// Shared header fields every transaction can carry. Embedded inside each
/// per-type builder ([`Pay`], [`CreateAsset`], [`CallApplication`], …), so
/// every builder has a single terminal `build(&params)` that finalises both
/// the header and the type-specific fields.
#[derive(Debug, Default, Clone)]
pub struct TxnHeader {
    pub(crate) fee: Option<MicroAlgos>,
    pub(crate) note: Option<Vec<u8>>,
    pub(crate) lease: Option<HashDigest>,
    pub(crate) rekey_to: Option<Address>,
    pub(crate) group: Option<HashDigest>,
    pub(crate) genesis_id: Option<String>,
}

impl TxnHeader {
    /// Combine this header with the suggested params and a type-specific
    /// `txn_type` to produce a finished [`Transaction`]. Used by every
    /// per-type builder's terminal `build(&params)`.
    ///
    /// Named `apply` rather than `into_transaction` because the latter
    /// reads like an `Into`-trait method (`fn into_X(self) -> X`), which
    /// this is not — it takes two extra arguments and can never be that
    /// trait.
    pub(crate) fn apply(
        self,
        params: &impl TransactionParams,
        txn_type: TransactionType,
    ) -> Result<Transaction, TransactionError> {
        let fee = self.fee.unwrap_or(MicroAlgos(params.min_fee()));
        let first_valid = Round(params.last_round());
        let last_valid = Round(params.last_round() + 1000);
        Ok(Transaction {
            fee,
            first_valid,
            genesis_hash: params.genesis_hash(),
            last_valid,
            txn_type,
            genesis_id: Some(
                self.genesis_id
                    .unwrap_or_else(|| params.genesis_id().clone()),
            ),
            group: self.group,
            lease: self.lease,
            note: self.note,
            rekey_to: self.rekey_to,
        })
    }
}

/// Mint the six fluent header setters (`fee`, `note`, `lease`, `rekey_to`,
/// `group`, `genesis_id`) on a per-type builder that has a `header:
/// TxnHeader` field. Used at the bottom of every builder's `impl` block.
macro_rules! impl_txn_header_setters {
    ($t:ty) => {
        impl $t {
            /// Override the per-byte-estimated fee from
            /// [`TransactionParams::min_fee`] with an explicit value. Useful
            /// in transaction groups where one txn pays the fee for others.
            pub fn fee(mut self, fee: MicroAlgos) -> Self {
                self.header.fee = Some(fee);
                self
            }

            /// Attach an opaque note to the transaction.
            pub fn note(mut self, note: Vec<u8>) -> Self {
                self.header.note = Some(note);
                self
            }

            /// Attach a lease — prevents a second transaction with the same
            /// lease + sender from being committed within the validity
            /// window.
            pub fn lease(mut self, lease: HashDigest) -> Self {
                self.header.lease = Some(lease);
                self
            }

            /// Rekey the sender's account to a new auth address as part of
            /// this transaction.
            pub fn rekey_to(mut self, rekey_to: Address) -> Self {
                self.header.rekey_to = Some(rekey_to);
                self
            }

            /// Stamp a precomputed group ID on this transaction. Normally
            /// you don't call this directly; [`crate::tx_group::TxGroup`]
            /// does it via its `TryFrom<Vec<Transaction>>` impl.
            pub fn group(mut self, group: HashDigest) -> Self {
                self.header.group = Some(group);
                self
            }

            /// Override the suggested-params genesis ID. Rarely needed.
            pub fn genesis_id(mut self, id: String) -> Self {
                self.header.genesis_id = Some(id);
                self
            }
        }
    };
}

/// A builder for [Payment].
pub struct Pay {
    sender: Address,
    receiver: Address,
    amount: MicroAlgos,
    close_remainder_to: Option<Address>,
    header: TxnHeader,
}

impl Pay {
    pub fn new(sender: Address, receiver: Address, amount: MicroAlgos) -> Self {
        Pay {
            sender,
            receiver,
            amount,
            close_remainder_to: None,
            header: TxnHeader::default(),
        }
    }

    /// A zero-amount self-payment that rekeys `from`'s account to a new
    /// authorising address.
    ///
    /// Algorand has no dedicated rekey transaction type; rekey is a
    /// header field that any transaction can carry, and a zero-amount
    /// self-payment is the canonical minimal carrier. The full
    /// [`Pay::new`] + [`Pay::rekey_to`] form stays available for the
    /// "rekey *and* actually pay someone" case.
    pub fn rekey(from: Address, new_auth: Address) -> Self {
        Self::new(from, from, MicroAlgos(0)).rekey_to(new_auth)
    }

    pub fn close_remainder_to(mut self, close_remainder_to: Address) -> Self {
        self.close_remainder_to = Some(close_remainder_to);
        self
    }

    pub fn build(self, params: &impl TransactionParams) -> Result<Transaction, TransactionError> {
        let txn_type = TransactionType::Payment(Payment {
            sender: self.sender,
            receiver: self.receiver,
            amount: self.amount,
            close_remainder_to: self.close_remainder_to,
        });
        self.header.apply(params, txn_type)
    }
}

impl_txn_header_setters!(Pay);

/// A builder for [KeyRegistration].
pub struct RegisterKey {
    sender: Address,
    vote_pk: Option<VotePk>,
    selection_pk: Option<VrfPk>,
    vote_first: Option<Round>,
    vote_last: Option<Round>,
    vote_key_dilution: Option<u64>,
    state_proof_key: Option<StateProofPk>,
    nonparticipating: Option<bool>,
    header: TxnHeader,
}

impl RegisterKey {
    /// Build an **online** v2 key-registration transaction.
    ///
    /// Since the v34 consensus upgrade an online registration must carry
    /// a state-proof (BLS) public key; pass it as `state_proof_key`.
    pub fn online(
        sender: Address,
        vote_pk: VotePk,
        selection_pk: VrfPk,
        state_proof_key: StateProofPk,
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
            state_proof_key: Some(state_proof_key),
            nonparticipating: None,
            header: TxnHeader::default(),
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
            state_proof_key: None,
            nonparticipating: None,
            header: TxnHeader::default(),
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
            state_proof_key: None,
            nonparticipating: Some(nonparticipating),
            header: TxnHeader::default(),
        }
    }

    pub fn build(self, params: &impl TransactionParams) -> Result<Transaction, TransactionError> {
        let txn_type = TransactionType::KeyRegistration(KeyRegistration {
            sender: self.sender,
            vote_pk: self.vote_pk,
            selection_pk: self.selection_pk,
            vote_first: self.vote_first,
            vote_last: self.vote_last,
            vote_key_dilution: self.vote_key_dilution,
            state_proof_key: self.state_proof_key,
            nonparticipating: self.nonparticipating,
        });
        self.header.apply(params, txn_type)
    }
}

impl_txn_header_setters!(RegisterKey);

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
    header: TxnHeader,
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
            header: TxnHeader::default(),
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

    pub fn build(self, params: &impl TransactionParams) -> Result<Transaction, TransactionError> {
        let txn_type =
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
            });
        self.header.apply(params, txn_type)
    }
}

impl_txn_header_setters!(CreateAsset);

/// A builder for [AssetConfigurationTransaction].
pub struct UpdateAsset {
    sender: Address,
    asset_id: AssetId,
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
    header: TxnHeader,
}

impl UpdateAsset {
    pub fn new(sender: Address, asset_id: AssetId) -> Self {
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
            header: TxnHeader::default(),
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

    pub fn build(self, params: &impl TransactionParams) -> Result<Transaction, TransactionError> {
        let txn_type =
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
            });
        self.header.apply(params, txn_type)
    }
}

impl_txn_header_setters!(UpdateAsset);

/// A builder for [AssetConfigurationTransaction].
pub struct DestroyAsset {
    sender: Address,
    asset_id: AssetId,
    header: TxnHeader,
}

impl DestroyAsset {
    pub fn new(sender: Address, asset_id: AssetId) -> Self {
        DestroyAsset {
            sender,
            asset_id,
            header: TxnHeader::default(),
        }
    }

    pub fn build(self, params: &impl TransactionParams) -> Result<Transaction, TransactionError> {
        let txn_type =
            TransactionType::AssetConfigurationTransaction(AssetConfigurationTransaction {
                sender: self.sender,
                config_asset: Some(self.asset_id),
                params: None,
            });
        self.header.apply(params, txn_type)
    }
}

impl_txn_header_setters!(DestroyAsset);

/// A builder for [AssetTransferTransaction].
pub struct TransferAsset {
    sender: Address,
    xfer: AssetId,
    amount: u64,
    receiver: Address,
    close_to: Option<Address>,
    header: TxnHeader,
}

impl TransferAsset {
    pub fn new(sender: Address, asset_id: AssetId, amount: u64, receiver: Address) -> Self {
        TransferAsset {
            sender,
            xfer: asset_id,
            amount,
            receiver,
            close_to: None,
            header: TxnHeader::default(),
        }
    }

    pub fn close_to(mut self, close_to: Address) -> Self {
        self.close_to = Some(close_to);
        self
    }

    pub fn build(self, params: &impl TransactionParams) -> Result<Transaction, TransactionError> {
        let txn_type = TransactionType::AssetTransferTransaction(AssetTransferTransaction {
            sender: self.sender,
            xfer: self.xfer,
            amount: self.amount,
            receiver: self.receiver,
            close_to: self.close_to,
        });
        self.header.apply(params, txn_type)
    }
}

impl_txn_header_setters!(TransferAsset);

/// A builder for [AssetAcceptTransaction].
pub struct AcceptAsset {
    sender: Address,
    asset_id: AssetId,
    header: TxnHeader,
}

impl AcceptAsset {
    pub fn new(sender: Address, asset_id: AssetId) -> Self {
        AcceptAsset {
            sender,
            asset_id,
            header: TxnHeader::default(),
        }
    }

    pub fn build(self, params: &impl TransactionParams) -> Result<Transaction, TransactionError> {
        let txn_type = TransactionType::AssetAcceptTransaction(AssetAcceptTransaction {
            sender: self.sender,
            xfer: self.asset_id,
        });
        self.header.apply(params, txn_type)
    }
}

impl_txn_header_setters!(AcceptAsset);

/// A builder for [AssetClawbackTransaction].
pub struct ClawbackAsset {
    sender: Address,
    asset_id: AssetId,
    asset_amount: u64,
    asset_sender: Address,
    asset_receiver: Address,
    asset_close_to: Option<Address>,
    header: TxnHeader,
}

impl ClawbackAsset {
    pub fn new(
        sender: Address,
        asset_id: AssetId,
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
            header: TxnHeader::default(),
        }
    }

    pub fn asset_close_to(mut self, asset_close_to: Address) -> Self {
        self.asset_close_to = Some(asset_close_to);
        self
    }

    pub fn build(self, params: &impl TransactionParams) -> Result<Transaction, TransactionError> {
        let txn_type = TransactionType::AssetClawbackTransaction(AssetClawbackTransaction {
            sender: self.sender,
            xfer: self.asset_id,
            asset_amount: self.asset_amount,
            asset_sender: self.asset_sender,
            asset_receiver: self.asset_receiver,
            asset_close_to: self.asset_close_to,
        });
        self.header.apply(params, txn_type)
    }
}

impl_txn_header_setters!(ClawbackAsset);

/// A builder for [AssetFreezeTransaction].
pub struct FreezeAsset {
    sender: Address,
    freeze_account: Address,
    asset_id: AssetId,
    frozen: bool,
    header: TxnHeader,
}

impl FreezeAsset {
    pub fn new(sender: Address, freeze_account: Address, asset_id: AssetId, frozen: bool) -> Self {
        FreezeAsset {
            sender,
            freeze_account,
            asset_id,
            frozen,
            header: TxnHeader::default(),
        }
    }

    pub fn build(self, params: &impl TransactionParams) -> Result<Transaction, TransactionError> {
        let txn_type = TransactionType::AssetFreezeTransaction(AssetFreezeTransaction {
            sender: self.sender,
            freeze_account: self.freeze_account,
            asset_id: self.asset_id,
            frozen: self.frozen,
        });
        self.header.apply(params, txn_type)
    }
}

impl_txn_header_setters!(FreezeAsset);

/// A builder for [ApplicationCallTransaction].
pub struct CreateApplication {
    sender: Address,
    accounts: Option<Vec<Address>>,
    approval_program: Option<CompiledTeal>,
    app_arguments: Option<Vec<Vec<u8>>>,
    clear_state_program: Option<CompiledTeal>,
    foreign_apps: Option<Vec<AppId>>,
    foreign_assets: Option<Vec<AssetId>>,
    global_state_schema: Option<StateSchema>,
    local_state_schema: Option<StateSchema>,
    extra_pages: u32,
    boxes: Option<Vec<BoxReference>>,
    header: TxnHeader,
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
            boxes: None,
            header: TxnHeader::default(),
        }
    }

    pub fn accounts(mut self, accounts: Vec<Address>) -> Self {
        self.accounts = Some(accounts);
        self
    }

    /// Raw byte-string arguments passed to the contract's TEAL program
    /// (the on-wire `apaa` field). The protocol imposes no type system
    /// here — each `Vec<u8>` lands on the AVM stack as a `bytes` value,
    /// and the TEAL program decides how to interpret it.
    ///
    /// **For ARC-4 (ABI-typed) calls, use `MethodCall` instead** — it
    /// encodes the method selector and arguments for you and decodes
    /// the return value. This setter is the lower-level escape hatch
    /// for calling contracts that don't follow ARC-4.
    pub fn app_arguments(mut self, app_arguments: Vec<Vec<u8>>) -> Self {
        self.app_arguments = Some(app_arguments);
        self
    }

    pub fn foreign_apps(mut self, foreign_apps: Vec<AppId>) -> Self {
        self.foreign_apps = Some(foreign_apps);
        self
    }

    pub fn foreign_assets(mut self, foreign_assets: Vec<AssetId>) -> Self {
        self.foreign_assets = Some(foreign_assets);
        self
    }

    pub fn extra_pages(mut self, extra_pages: u32) -> Self {
        self.extra_pages = extra_pages;
        self
    }

    pub fn boxes(mut self, boxes: Vec<BoxReference>) -> Self {
        self.boxes = Some(boxes);
        self
    }

    pub fn build(self, params: &impl TransactionParams) -> Result<Transaction, TransactionError> {
        let txn_type = TransactionType::ApplicationCallTransaction(ApplicationCallTransaction {
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
            boxes: self.boxes,
        });
        self.header.apply(params, txn_type)
    }
}

impl_txn_header_setters!(CreateApplication);

/// A builder for [ApplicationCallTransaction].
pub struct UpdateApplication {
    sender: Address,
    app_id: AppId,
    accounts: Option<Vec<Address>>,
    approval_program: Option<CompiledTeal>,
    app_arguments: Option<Vec<Vec<u8>>>,
    clear_state_program: Option<CompiledTeal>,
    foreign_apps: Option<Vec<AppId>>,
    foreign_assets: Option<Vec<AssetId>>,
    boxes: Option<Vec<BoxReference>>,
    header: TxnHeader,
}

impl UpdateApplication {
    pub fn new(
        sender: Address,
        app_id: AppId,
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
            boxes: None,
            header: TxnHeader::default(),
        }
    }

    pub fn accounts(mut self, accounts: Vec<Address>) -> Self {
        self.accounts = Some(accounts);
        self
    }

    /// Raw byte-string arguments passed to the contract's TEAL program
    /// (the on-wire `apaa` field). The protocol imposes no type system
    /// here — each `Vec<u8>` lands on the AVM stack as a `bytes` value,
    /// and the TEAL program decides how to interpret it.
    ///
    /// **For ARC-4 (ABI-typed) calls, use `MethodCall` instead** — it
    /// encodes the method selector and arguments for you and decodes
    /// the return value. This setter is the lower-level escape hatch
    /// for calling contracts that don't follow ARC-4.
    pub fn app_arguments(mut self, app_arguments: Vec<Vec<u8>>) -> Self {
        self.app_arguments = Some(app_arguments);
        self
    }

    pub fn foreign_apps(mut self, foreign_apps: Vec<AppId>) -> Self {
        self.foreign_apps = Some(foreign_apps);
        self
    }

    pub fn foreign_assets(mut self, foreign_assets: Vec<AssetId>) -> Self {
        self.foreign_assets = Some(foreign_assets);
        self
    }

    pub fn boxes(mut self, boxes: Vec<BoxReference>) -> Self {
        self.boxes = Some(boxes);
        self
    }

    pub fn build(self, params: &impl TransactionParams) -> Result<Transaction, TransactionError> {
        let txn_type = TransactionType::ApplicationCallTransaction(ApplicationCallTransaction {
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
            boxes: self.boxes,
        });
        self.header.apply(params, txn_type)
    }
}

impl_txn_header_setters!(UpdateApplication);

/// A builder for [ApplicationCallTransaction].
pub struct CallApplication {
    sender: Address,
    app_id: AppId,
    accounts: Option<Vec<Address>>,
    app_arguments: Option<Vec<Vec<u8>>>,
    foreign_apps: Option<Vec<AppId>>,
    foreign_assets: Option<Vec<AssetId>>,
    boxes: Option<Vec<BoxReference>>,
    header: TxnHeader,
}

impl CallApplication {
    pub fn new(sender: Address, app_id: AppId) -> Self {
        CallApplication {
            sender,
            app_id,
            accounts: None,
            app_arguments: None,
            foreign_apps: None,
            foreign_assets: None,
            boxes: None,
            header: TxnHeader::default(),
        }
    }

    pub fn accounts(mut self, accounts: Vec<Address>) -> Self {
        self.accounts = Some(accounts);
        self
    }

    /// Raw byte-string arguments passed to the contract's TEAL program
    /// (the on-wire `apaa` field). The protocol imposes no type system
    /// here — each `Vec<u8>` lands on the AVM stack as a `bytes` value,
    /// and the TEAL program decides how to interpret it.
    ///
    /// **For ARC-4 (ABI-typed) calls, use `MethodCall` instead** — it
    /// encodes the method selector and arguments for you and decodes
    /// the return value. This setter is the lower-level escape hatch
    /// for calling contracts that don't follow ARC-4.
    pub fn app_arguments(mut self, app_arguments: Vec<Vec<u8>>) -> Self {
        self.app_arguments = Some(app_arguments);
        self
    }

    pub fn foreign_apps(mut self, foreign_apps: Vec<AppId>) -> Self {
        self.foreign_apps = Some(foreign_apps);
        self
    }

    pub fn foreign_assets(mut self, foreign_assets: Vec<AssetId>) -> Self {
        self.foreign_assets = Some(foreign_assets);
        self
    }

    pub fn boxes(mut self, boxes: Vec<BoxReference>) -> Self {
        self.boxes = Some(boxes);
        self
    }

    pub fn build(self, params: &impl TransactionParams) -> Result<Transaction, TransactionError> {
        let txn_type = TransactionType::ApplicationCallTransaction(ApplicationCallTransaction {
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
            boxes: self.boxes,
        });
        self.header.apply(params, txn_type)
    }
}

impl_txn_header_setters!(CallApplication);

/// A builder for [ApplicationCallTransaction].
pub struct ClearApplication {
    sender: Address,
    app_id: AppId,
    accounts: Option<Vec<Address>>,
    app_arguments: Option<Vec<Vec<u8>>>,
    foreign_apps: Option<Vec<AppId>>,
    foreign_assets: Option<Vec<AssetId>>,
    boxes: Option<Vec<BoxReference>>,
    header: TxnHeader,
}

impl ClearApplication {
    pub fn new(sender: Address, app_id: AppId) -> Self {
        ClearApplication {
            sender,
            app_id,
            accounts: None,
            app_arguments: None,
            foreign_apps: None,
            foreign_assets: None,
            boxes: None,
            header: TxnHeader::default(),
        }
    }

    pub fn accounts(mut self, accounts: Vec<Address>) -> Self {
        self.accounts = Some(accounts);
        self
    }

    /// Raw byte-string arguments passed to the contract's TEAL program
    /// (the on-wire `apaa` field). The protocol imposes no type system
    /// here — each `Vec<u8>` lands on the AVM stack as a `bytes` value,
    /// and the TEAL program decides how to interpret it.
    ///
    /// **For ARC-4 (ABI-typed) calls, use `MethodCall` instead** — it
    /// encodes the method selector and arguments for you and decodes
    /// the return value. This setter is the lower-level escape hatch
    /// for calling contracts that don't follow ARC-4.
    pub fn app_arguments(mut self, app_arguments: Vec<Vec<u8>>) -> Self {
        self.app_arguments = Some(app_arguments);
        self
    }

    pub fn foreign_apps(mut self, foreign_apps: Vec<AppId>) -> Self {
        self.foreign_apps = Some(foreign_apps);
        self
    }

    pub fn foreign_assets(mut self, foreign_assets: Vec<AssetId>) -> Self {
        self.foreign_assets = Some(foreign_assets);
        self
    }

    pub fn boxes(mut self, boxes: Vec<BoxReference>) -> Self {
        self.boxes = Some(boxes);
        self
    }

    pub fn build(self, params: &impl TransactionParams) -> Result<Transaction, TransactionError> {
        let txn_type = TransactionType::ApplicationCallTransaction(ApplicationCallTransaction {
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
            boxes: self.boxes,
        });
        self.header.apply(params, txn_type)
    }
}

impl_txn_header_setters!(ClearApplication);

/// A builder for [ApplicationCallTransaction].
pub struct CloseApplication {
    sender: Address,
    app_id: AppId,
    accounts: Option<Vec<Address>>,
    app_arguments: Option<Vec<Vec<u8>>>,
    foreign_apps: Option<Vec<AppId>>,
    foreign_assets: Option<Vec<AssetId>>,
    boxes: Option<Vec<BoxReference>>,
    header: TxnHeader,
}

impl CloseApplication {
    pub fn new(sender: Address, app_id: AppId) -> Self {
        CloseApplication {
            sender,
            app_id,
            accounts: None,
            app_arguments: None,
            foreign_apps: None,
            foreign_assets: None,
            boxes: None,
            header: TxnHeader::default(),
        }
    }

    pub fn accounts(mut self, accounts: Vec<Address>) -> Self {
        self.accounts = Some(accounts);
        self
    }

    /// Raw byte-string arguments passed to the contract's TEAL program
    /// (the on-wire `apaa` field). The protocol imposes no type system
    /// here — each `Vec<u8>` lands on the AVM stack as a `bytes` value,
    /// and the TEAL program decides how to interpret it.
    ///
    /// **For ARC-4 (ABI-typed) calls, use `MethodCall` instead** — it
    /// encodes the method selector and arguments for you and decodes
    /// the return value. This setter is the lower-level escape hatch
    /// for calling contracts that don't follow ARC-4.
    pub fn app_arguments(mut self, app_arguments: Vec<Vec<u8>>) -> Self {
        self.app_arguments = Some(app_arguments);
        self
    }

    pub fn foreign_apps(mut self, foreign_apps: Vec<AppId>) -> Self {
        self.foreign_apps = Some(foreign_apps);
        self
    }

    pub fn foreign_assets(mut self, foreign_assets: Vec<AssetId>) -> Self {
        self.foreign_assets = Some(foreign_assets);
        self
    }

    pub fn boxes(mut self, boxes: Vec<BoxReference>) -> Self {
        self.boxes = Some(boxes);
        self
    }

    pub fn build(self, params: &impl TransactionParams) -> Result<Transaction, TransactionError> {
        let txn_type = TransactionType::ApplicationCallTransaction(ApplicationCallTransaction {
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
            boxes: self.boxes,
        });
        self.header.apply(params, txn_type)
    }
}

impl_txn_header_setters!(CloseApplication);

/// A builder for [ApplicationCallTransaction].
pub struct DeleteApplication {
    sender: Address,
    app_id: AppId,
    accounts: Option<Vec<Address>>,
    app_arguments: Option<Vec<Vec<u8>>>,
    foreign_apps: Option<Vec<AppId>>,
    foreign_assets: Option<Vec<AssetId>>,
    boxes: Option<Vec<BoxReference>>,
    header: TxnHeader,
}

impl DeleteApplication {
    pub fn new(sender: Address, app_id: AppId) -> Self {
        DeleteApplication {
            sender,
            app_id,
            accounts: None,
            app_arguments: None,
            foreign_apps: None,
            foreign_assets: None,
            boxes: None,
            header: TxnHeader::default(),
        }
    }

    pub fn accounts(mut self, accounts: Vec<Address>) -> Self {
        self.accounts = Some(accounts);
        self
    }

    /// Raw byte-string arguments passed to the contract's TEAL program
    /// (the on-wire `apaa` field). The protocol imposes no type system
    /// here — each `Vec<u8>` lands on the AVM stack as a `bytes` value,
    /// and the TEAL program decides how to interpret it.
    ///
    /// **For ARC-4 (ABI-typed) calls, use `MethodCall` instead** — it
    /// encodes the method selector and arguments for you and decodes
    /// the return value. This setter is the lower-level escape hatch
    /// for calling contracts that don't follow ARC-4.
    pub fn app_arguments(mut self, app_arguments: Vec<Vec<u8>>) -> Self {
        self.app_arguments = Some(app_arguments);
        self
    }

    pub fn foreign_apps(mut self, foreign_apps: Vec<AppId>) -> Self {
        self.foreign_apps = Some(foreign_apps);
        self
    }

    pub fn foreign_assets(mut self, foreign_assets: Vec<AssetId>) -> Self {
        self.foreign_assets = Some(foreign_assets);
        self
    }

    pub fn boxes(mut self, boxes: Vec<BoxReference>) -> Self {
        self.boxes = Some(boxes);
        self
    }

    pub fn build(self, params: &impl TransactionParams) -> Result<Transaction, TransactionError> {
        let txn_type = TransactionType::ApplicationCallTransaction(ApplicationCallTransaction {
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
            boxes: self.boxes,
        });
        self.header.apply(params, txn_type)
    }
}

impl_txn_header_setters!(DeleteApplication);

/// A builder for [ApplicationCallTransaction].
pub struct OptInApplication {
    sender: Address,
    app_id: AppId,
    accounts: Option<Vec<Address>>,
    app_arguments: Option<Vec<Vec<u8>>>,
    foreign_apps: Option<Vec<AppId>>,
    foreign_assets: Option<Vec<AssetId>>,
    boxes: Option<Vec<BoxReference>>,
    header: TxnHeader,
}

impl OptInApplication {
    pub fn new(sender: Address, app_id: AppId) -> Self {
        OptInApplication {
            sender,
            app_id,
            accounts: None,
            app_arguments: None,
            foreign_apps: None,
            foreign_assets: None,
            boxes: None,
            header: TxnHeader::default(),
        }
    }

    pub fn accounts(mut self, accounts: Vec<Address>) -> Self {
        self.accounts = Some(accounts);
        self
    }

    /// Raw byte-string arguments passed to the contract's TEAL program
    /// (the on-wire `apaa` field). The protocol imposes no type system
    /// here — each `Vec<u8>` lands on the AVM stack as a `bytes` value,
    /// and the TEAL program decides how to interpret it.
    ///
    /// **For ARC-4 (ABI-typed) calls, use `MethodCall` instead** — it
    /// encodes the method selector and arguments for you and decodes
    /// the return value. This setter is the lower-level escape hatch
    /// for calling contracts that don't follow ARC-4.
    pub fn app_arguments(mut self, app_arguments: Vec<Vec<u8>>) -> Self {
        self.app_arguments = Some(app_arguments);
        self
    }

    pub fn foreign_apps(mut self, foreign_apps: Vec<AppId>) -> Self {
        self.foreign_apps = Some(foreign_apps);
        self
    }

    pub fn foreign_assets(mut self, foreign_assets: Vec<AssetId>) -> Self {
        self.foreign_assets = Some(foreign_assets);
        self
    }

    pub fn boxes(mut self, boxes: Vec<BoxReference>) -> Self {
        self.boxes = Some(boxes);
        self
    }

    pub fn build(self, params: &impl TransactionParams) -> Result<Transaction, TransactionError> {
        let txn_type = TransactionType::ApplicationCallTransaction(ApplicationCallTransaction {
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
            boxes: self.boxes,
        });
        self.header.apply(params, txn_type)
    }
}

impl_txn_header_setters!(OptInApplication);
