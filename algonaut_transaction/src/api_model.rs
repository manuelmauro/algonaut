use std::convert::{TryFrom, TryInto};

use algonaut_core::{
    Address, LogicSignature, MicroAlgos, MultisigSignature, Round, Signature, SignedLogic,
    ToMsgPack, VotePk, VrfPk,
};
use algonaut_crypto::HashDigest;
use serde::{Deserialize, Serialize, Serializer};

use crate::{
    error::TransactionError,
    transaction::{
        ApplicationCallTransaction, AssetAcceptTransaction, AssetClawbackTransaction,
        AssetConfigurationTransaction, AssetFreezeTransaction, AssetParams,
        AssetTransferTransaction, KeyRegistration, Payment, StateSchema, TransactionSignature,
    },
    tx_group::TxGroup,
    SignedTransaction, Transaction, TransactionType,
};

// Important: When signing:
// - Fields have to be sorted alphabetically.
// - Keys must be excluded if they've no value.
// The signature validation fails otherwise.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiTransaction {
    #[serde(rename = "aamt", skip_serializing_if = "Option::is_none")]
    pub asset_amount: Option<u64>,

    #[serde(rename = "aclose", skip_serializing_if = "Option::is_none")]
    pub asset_close_to: Option<Address>,

    #[serde(rename = "afrz", skip_serializing_if = "Option::is_none")]
    pub frozen: Option<bool>,

    #[serde(rename = "amt", skip_serializing_if = "Option::is_none")]
    pub amount: Option<u64>,

    #[serde(rename = "apaa", skip_serializing_if = "Option::is_none")]
    pub app_arguments: Option<Vec<u8>>,

    #[serde(rename = "apan", skip_serializing_if = "Option::is_none")]
    pub on_complete: Option<u64>,

    #[serde(rename = "apap", skip_serializing_if = "Option::is_none")]
    pub approval_program: Option<Address>,

    #[serde(rename = "apar", skip_serializing_if = "Option::is_none")]
    pub asset_params: Option<ApiAssetParams>,

    #[serde(rename = "apas", skip_serializing_if = "Option::is_none")]
    pub foreign_assets: Option<Address>,

    #[serde(rename = "apat", skip_serializing_if = "Option::is_none")]
    pub accounts: Option<Vec<Address>>,

    #[serde(rename = "apfa", skip_serializing_if = "Option::is_none")]
    pub foreign_apps: Option<Address>,

    #[serde(rename = "apgs", skip_serializing_if = "Option::is_none")]
    pub global_state_schema: Option<ApiStateSchema>,

    #[serde(rename = "apid", skip_serializing_if = "Option::is_none")]
    pub app_id: Option<u64>,

    #[serde(rename = "apls", skip_serializing_if = "Option::is_none")]
    pub local_state_schema: Option<ApiStateSchema>,

    #[serde(rename = "apsu", skip_serializing_if = "Option::is_none")]
    pub clear_state_program: Option<Address>,

    #[serde(rename = "arcv", skip_serializing_if = "Option::is_none")]
    pub asset_receiver: Option<Address>,

    #[serde(rename = "asnd", skip_serializing_if = "Option::is_none")]
    pub asset_sender: Option<Address>,

    #[serde(rename = "caid", skip_serializing_if = "Option::is_none")]
    pub config_asset: Option<u64>,

    #[serde(rename = "close", skip_serializing_if = "Option::is_none")]
    pub close_reminder_to: Option<Address>,

    #[serde(rename = "fadd", skip_serializing_if = "Option::is_none")]
    pub freeze_account: Option<Address>,

    #[serde(rename = "faid", skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<u64>,

    #[serde(rename = "fee")]
    pub fee: MicroAlgos,

    #[serde(rename = "fv")]
    pub first_valid: Round,

    #[serde(rename = "gen", skip_serializing_if = "Option::is_none")]
    pub genesis_id: Option<String>,

    #[serde(rename = "gh")]
    pub genesis_hash: HashDigest,

    #[serde(rename = "grp", skip_serializing_if = "Option::is_none")]
    pub group: Option<HashDigest>,

    #[serde(rename = "lv")]
    pub last_valid: Round,

    #[serde(rename = "lx", skip_serializing_if = "Option::is_none")]
    pub lease: Option<HashDigest>,

    #[serde(rename = "nonpart", skip_serializing_if = "Option::is_none")]
    pub nonparticipating: Option<bool>,

    #[serde(
        default,
        rename = "note",
        with = "serde_bytes",
        skip_serializing_if = "Option::is_none"
    )]
    pub note: Option<Vec<u8>>,

    #[serde(rename = "rcv", skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Address>,

    #[serde(rename = "rekey", skip_serializing_if = "Option::is_none")]
    pub rekey_to: Option<Address>,

    #[serde(rename = "selkey", skip_serializing_if = "Option::is_none")]
    pub selection_pk: Option<VrfPk>,

    #[serde(rename = "snd")]
    pub sender: Address,

    #[serde(rename = "type")]
    pub type_: String,

    #[serde(rename = "votefst", skip_serializing_if = "Option::is_none")]
    pub vote_first: Option<Round>,

    #[serde(rename = "votekd", skip_serializing_if = "Option::is_none")]
    pub vote_key_dilution: Option<u64>,

    #[serde(rename = "votekey", skip_serializing_if = "Option::is_none")]
    pub vote_pk: Option<VotePk>,

    #[serde(rename = "votelst", skip_serializing_if = "Option::is_none")]
    pub vote_last: Option<Round>,

    #[serde(rename = "xaid", skip_serializing_if = "Option::is_none")]
    pub xfer: Option<u64>,
}

impl From<Transaction> for ApiTransaction {
    fn from(t: Transaction) -> Self {
        let mut api_t = ApiTransaction {
            // Common fields
            fee: t.fee,
            first_valid: t.first_valid,
            genesis_id: t.genesis_id.clone(),
            genesis_hash: t.genesis_hash,
            group: t.group,
            last_valid: t.last_valid,
            lease: t.lease,
            note: t.note.clone(),
            rekey_to: t.rekey_to,
            sender: t.sender(),
            type_: to_api_transaction_type(&t.txn_type).to_owned(),
            ///////////////
            asset_amount: None,
            asset_close_to: None,
            frozen: None,
            amount: None,
            app_arguments: None,
            on_complete: None,
            approval_program: None,
            asset_params: None,
            foreign_assets: None,
            accounts: None,
            foreign_apps: None,
            global_state_schema: None,
            app_id: None,
            local_state_schema: None,
            clear_state_program: None,
            asset_receiver: None,
            asset_sender: None,
            config_asset: None,
            close_reminder_to: None,
            freeze_account: None,
            asset_id: None,
            receiver: None,
            selection_pk: None,
            vote_first: None,
            vote_key_dilution: None,
            vote_pk: None,
            vote_last: None,
            xfer: None,
            nonparticipating: None,
        };

        match &t.txn_type {
            TransactionType::Payment(payment) => {
                api_t.receiver = Some(payment.receiver);
                api_t.amount = Some(payment.amount.0);
                api_t.close_reminder_to = payment.close_remainder_to;
            }
            TransactionType::KeyRegistration(reg) => {
                api_t.vote_pk = reg.vote_pk;
                api_t.selection_pk = reg.selection_pk;
                api_t.vote_first = reg.vote_first;
                api_t.vote_last = reg.vote_last;
                api_t.vote_key_dilution = reg.vote_key_dilution;
                api_t.nonparticipating = reg.nonparticipating;
            }
            TransactionType::AssetConfigurationTransaction(config) => {
                api_t.asset_params = config.to_owned().params.map(|p| p.into());
                api_t.config_asset = config.config_asset;
            }
            TransactionType::AssetTransferTransaction(transfer) => {
                api_t.xfer = Some(transfer.xfer);
                api_t.asset_amount = Some(transfer.amount);
                api_t.asset_receiver = Some(transfer.receiver);
                api_t.asset_close_to = transfer.close_to;
            }
            TransactionType::AssetAcceptTransaction(accept) => {
                api_t.xfer = Some(accept.xfer);
                api_t.asset_receiver = Some(accept.sender);
            }
            TransactionType::AssetClawbackTransaction(clawback) => {
                api_t.xfer = Some(clawback.xfer);
                api_t.asset_amount = Some(clawback.asset_amount);
                api_t.asset_sender = Some(clawback.asset_sender);
                api_t.asset_receiver = Some(clawback.asset_receiver);
                api_t.asset_close_to = clawback.asset_close_to;
            }
            TransactionType::AssetFreezeTransaction(freeze) => {
                api_t.freeze_account = Some(freeze.freeze_account);
                api_t.asset_id = Some(freeze.asset_id);
                api_t.frozen = Some(freeze.frozen);
            }
            TransactionType::ApplicationCallTransaction(call) => {
                api_t.app_id = Some(call.app_id);
                api_t.on_complete = Some(call.on_complete);
                api_t.accounts = call.accounts.to_owned();
                api_t.approval_program = call.approval_program;
                api_t.app_arguments = call.app_arguments.to_owned();
                api_t.clear_state_program = call.clear_state_program;
                api_t.foreign_apps = call.foreign_apps;
                api_t.foreign_assets = call.foreign_assets;
                api_t.global_state_schema = call.to_owned().global_state_schema.map(|s| s.into());
                api_t.local_state_schema = call.to_owned().local_state_schema.map(|s| s.into());
            }
        }
        api_t
    }
}

impl From<ApiTransaction> for Transaction {
    fn from(api_t: ApiTransaction) -> Self {
        let txn_type = match api_t.type_.as_ref() {
            "pay" => TransactionType::Payment(Payment {
                sender: api_t.sender,
                receiver: api_t.receiver.unwrap(),
                amount: MicroAlgos(api_t.amount.unwrap()),
                close_remainder_to: api_t.close_reminder_to,
            }),
            "keyreg" => TransactionType::KeyRegistration(KeyRegistration {
                sender: api_t.sender,
                vote_pk: api_t.vote_pk,
                selection_pk: api_t.selection_pk,
                vote_first: api_t.vote_first,
                vote_last: api_t.vote_last,
                vote_key_dilution: api_t.vote_key_dilution,
                nonparticipating: api_t.nonparticipating,
            }),
            "acfg" => {
                TransactionType::AssetConfigurationTransaction(AssetConfigurationTransaction {
                    sender: api_t.sender,
                    params: None, // TODO
                    config_asset: api_t.config_asset,
                })
            }
            "axfer" => parse_asset_transfer_transaction(&api_t),
            "afrz" => TransactionType::AssetFreezeTransaction(AssetFreezeTransaction {
                sender: api_t.sender,
                freeze_account: api_t.freeze_account.unwrap(),
                asset_id: api_t.asset_id.unwrap(),
                frozen: api_t.frozen.unwrap(),
            }),
            "appl" => TransactionType::ApplicationCallTransaction(ApplicationCallTransaction {
                sender: api_t.sender,
                app_id: api_t.app_id.unwrap(),
                on_complete: api_t.on_complete.unwrap(),
                accounts: api_t.accounts,
                approval_program: api_t.approval_program,
                app_arguments: api_t.app_arguments,
                clear_state_program: api_t.clear_state_program,
                foreign_apps: api_t.foreign_apps,
                foreign_assets: api_t.foreign_assets,
                global_state_schema: api_t.global_state_schema.map(|s| s.into()),
                local_state_schema: api_t.local_state_schema.map(|s| s.into()),
            }),
            _ => panic!(),
        };
        Transaction {
            fee: api_t.fee,
            first_valid: api_t.first_valid,
            genesis_id: api_t.genesis_id,
            genesis_hash: api_t.genesis_hash,
            group: api_t.group,
            last_valid: api_t.last_valid,
            lease: api_t.lease,
            note: api_t.note.clone(),
            rekey_to: api_t.rekey_to,
            txn_type,
        }
    }
}

fn parse_asset_transfer_transaction(api_t: &ApiTransaction) -> TransactionType {
    match (
        api_t.xfer,
        api_t.asset_sender,
        api_t.asset_receiver,
        api_t.asset_amount,
    ) {
        (Some(xfer), Some(asset_sender), Some(asset_receiver), Some(asset_amount)) => {
            TransactionType::AssetClawbackTransaction(AssetClawbackTransaction {
                sender: api_t.sender,
                xfer,
                asset_amount,
                asset_sender,
                asset_receiver,
                asset_close_to: api_t.asset_close_to,
            })
        }
        (Some(xfer), None, Some(asset_receiver), Some(asset_amount)) => {
            TransactionType::AssetTransferTransaction(AssetTransferTransaction {
                sender: api_t.sender,
                xfer,
                amount: asset_amount,
                receiver: asset_receiver,
                close_to: api_t.asset_close_to,
            })
        }
        (Some(xfer), None, None, None) => {
            TransactionType::AssetAcceptTransaction(AssetAcceptTransaction {
                sender: api_t.sender,
                xfer,
            })
        }
        _ => panic!(),
    }
}

impl TryFrom<ApiSignedTransaction> for SignedTransaction {
    type Error = TransactionError;

    fn try_from(api_t: ApiSignedTransaction) -> Result<Self, Self::Error> {
        Ok(SignedTransaction {
            transaction: api_t.transaction.clone().into(),
            transaction_id: api_t.transaction_id.clone(),
            sig: transaction_signature(&api_t)?,
        })
    }
}

fn transaction_signature(
    api_t: &ApiSignedTransaction,
) -> Result<TransactionSignature, TransactionError> {
    match (&api_t.sig, &api_t.lsig, &api_t.msig) {
        (Some(sig), None, None) => Ok(TransactionSignature::Single(sig.clone())),
        (None, Some(lsig), None) => Ok(TransactionSignature::Logic(lsig.clone().try_into()?)),
        (None, None, Some(msig)) => Ok(TransactionSignature::Multi(msig.clone())),
        _ => Err(TransactionError::Deserialization(format!(
            "Invalid sig combination: {:?}",
            api_t
        ))),
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ApiAssetParams {
    #[serde(rename = "am", skip_serializing_if = "Option::is_none")]
    pub meta_data_hash: Option<Vec<u8>>,

    #[serde(rename = "an", skip_serializing_if = "Option::is_none")]
    pub asset_name: Option<String>,

    #[serde(rename = "au", skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    #[serde(rename = "c", skip_serializing_if = "Option::is_none")]
    pub clawback: Option<Address>,

    #[serde(rename = "dc")]
    pub decimals: Option<u32>,

    #[serde(rename = "df", skip_serializing)]
    pub default_frozen: Option<bool>,

    #[serde(rename = "f", skip_serializing_if = "Option::is_none")]
    pub freeze: Option<Address>,

    #[serde(rename = "m", skip_serializing_if = "Option::is_none")]
    pub manager: Option<Address>,

    #[serde(rename = "r", skip_serializing_if = "Option::is_none")]
    pub reserve: Option<Address>,

    #[serde(rename = "t")]
    pub total: Option<u64>,

    #[serde(rename = "un", skip_serializing_if = "Option::is_none")]
    pub unit_name: Option<String>,
}

impl From<AssetParams> for ApiAssetParams {
    fn from(params: AssetParams) -> Self {
        ApiAssetParams {
            asset_name: params.asset_name,
            decimals: params.decimals,
            default_frozen: params.default_frozen,
            total: params.total,
            unit_name: params.unit_name,
            meta_data_hash: params.meta_data_hash,
            url: params.url,
            clawback: params.clawback,
            freeze: params.freeze,
            manager: params.manager,
            reserve: params.reserve,
        }
    }
}

impl From<ApiAssetParams> for AssetParams {
    fn from(params: ApiAssetParams) -> Self {
        AssetParams {
            asset_name: params.asset_name,
            decimals: params.decimals,
            default_frozen: params.default_frozen,
            total: params.total,
            unit_name: params.unit_name,
            meta_data_hash: params.meta_data_hash,
            url: params.url,
            clawback: params.clawback,
            freeze: params.freeze,
            manager: params.manager,
            reserve: params.reserve,
        }
    }
}

fn to_api_transaction_type<'a>(type_: &TransactionType) -> &'a str {
    match type_ {
        TransactionType::Payment(_) => "pay",
        TransactionType::KeyRegistration(_) => "keyreg",
        TransactionType::AssetConfigurationTransaction(_) => "acfg",
        TransactionType::AssetTransferTransaction(_) => "axfer",
        TransactionType::AssetAcceptTransaction(_) => "axfer",
        TransactionType::AssetClawbackTransaction(_) => "axfer",
        TransactionType::AssetFreezeTransaction(_) => "afrz",
        TransactionType::ApplicationCallTransaction(_) => "appl",
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiSignedTransaction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<Signature>,

    #[serde(rename = "msig", skip_serializing_if = "Option::is_none")]
    pub msig: Option<MultisigSignature>,

    #[serde(rename = "lsig", skip_serializing_if = "Option::is_none")]
    pub lsig: Option<ApiSignedLogic>,

    #[serde(rename = "txn")]
    pub transaction: ApiTransaction,

    #[serde(skip)]
    pub transaction_id: String,
}

impl From<SignedTransaction> for ApiSignedTransaction {
    fn from(t: SignedTransaction) -> Self {
        let (sig, msig, lsig) = match t.sig {
            TransactionSignature::Single(sig) => (Some(sig), None, None),
            TransactionSignature::Multi(msig) => (None, Some(msig), None),
            TransactionSignature::Logic(lsig) => (None, None, Some(lsig)),
        };
        ApiSignedTransaction {
            sig,
            msig,
            lsig: lsig.map(|l| l.into()),
            transaction: t.transaction.into(),
            transaction_id: t.transaction_id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApiStateSchema {
    #[serde(rename = "nui")]
    pub number_ints: u64,

    #[serde(rename = "nbs")]
    pub number_byteslices: u64,
}

impl From<StateSchema> for ApiStateSchema {
    fn from(state_schema: StateSchema) -> Self {
        ApiStateSchema {
            number_ints: state_schema.number_ints,
            number_byteslices: state_schema.number_byteslices,
        }
    }
}

impl From<ApiStateSchema> for StateSchema {
    fn from(state_schema: ApiStateSchema) -> Self {
        StateSchema {
            number_ints: state_schema.number_ints,
            number_byteslices: state_schema.number_byteslices,
        }
    }
}

impl ToMsgPack for ApiTransaction {}
impl ToMsgPack for ApiSignedTransaction {}
impl ToMsgPack for Transaction {}
impl ToMsgPack for SignedTransaction {}
impl ToMsgPack for TxGroup {}

/// Convenience to serialize Transaction directly to msg pack
impl Serialize for Transaction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let api_transaction: ApiTransaction = self.to_owned().into();
        api_transaction.serialize(serializer)
    }
}

/// Convenience to deserialize Transaction directly from msg pack
impl<'de> Deserialize<'de> for Transaction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let api_transaction = ApiTransaction::deserialize(deserializer)?;
        Ok(api_transaction.into())
    }
}

/// Convenience to serialize SignedTransaction directly to msg pack
impl Serialize for SignedTransaction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let api_transaction: ApiSignedTransaction = self.to_owned().into();
        api_transaction.serialize(serializer)
    }
}

/// Convenience to deserialize SignedTransaction directly from msg pack
impl<'de> Deserialize<'de> for SignedTransaction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(ApiSignedTransaction::deserialize(deserializer)?
            .try_into()
            .map_err(serde::de::Error::custom)?)
    }
}

#[derive(Default, Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct ApiSignedLogic {
    pub logic: Vec<u8>,
    pub sig: Option<Signature>,
    pub msig: Option<MultisigSignature>,
    pub args: Vec<Vec<u8>>,
}

impl From<SignedLogic> for ApiSignedLogic {
    fn from(s: SignedLogic) -> Self {
        let (sig, msig) = match s.sig {
            LogicSignature::ContractAccount => (None, None),
            LogicSignature::DelegatedSig(sig) => (Some(sig), None),
            LogicSignature::DelegatedMultiSig(msig) => (None, Some(msig)),
        };
        ApiSignedLogic {
            logic: s.logic,
            sig,
            msig,
            args: s.args,
        }
    }
}

impl TryFrom<ApiSignedLogic> for SignedLogic {
    type Error = TransactionError;

    fn try_from(s: ApiSignedLogic) -> Result<Self, Self::Error> {
        let sig = match (s.sig, s.msig) {
            (Some(sig), None) => LogicSignature::DelegatedSig(sig),
            (None, Some(msig)) => LogicSignature::DelegatedMultiSig(msig),
            (None, None) => LogicSignature::ContractAccount,
            _ => Err(TransactionError::Deserialization(
                "Invalid sig/msig combination".to_owned(),
            ))?,
        };
        Ok(SignedLogic {
            logic: s.logic,
            args: s.args,
            sig,
        })
    }
}

impl Serialize for ApiSignedLogic {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        // For some reason SerializeStruct ends up serializing as an array, so this explicitly serializes as a map
        use serde::ser::SerializeMap;
        let mut state = serializer.serialize_map(Some(4))?;
        state.serialize_entry("l", &self.logic)?;
        state.serialize_entry("arg", &self.args)?;
        state.serialize_entry("sig", &self.sig)?;
        state.serialize_entry("msig", &self.msig)?;
        state.end()
    }
}
