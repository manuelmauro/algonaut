use std::convert::{TryFrom, TryInto};

use crate::{
    error::TransactionError,
    transaction::{
        to_tx_type_enum, ApplicationCallOnComplete, ApplicationCallTransaction,
        AssetAcceptTransaction, AssetClawbackTransaction, AssetConfigurationTransaction,
        AssetFreezeTransaction, AssetParams, AssetTransferTransaction, KeyRegistration, Payment,
        SignedLogic, StateProofTransaction, StateProofType, StateSchema, TransactionSignature,
    },
    tx_group::TxGroup,
    SignedTransaction, Transaction, TransactionType,
};
use algonaut_core::{CompiledTeal, LogicSignature, MicroAlgos, Round, ToMsgPack};
use algonaut_model::transaction::{
    ApiAssetParams, ApiSignedLogic, ApiSignedLogicArg, ApiSignedTransaction, ApiStateSchema,
    ApiTransaction, AppArgument,
};
use num_traits::Num;
use serde::{Deserialize, Serialize};

impl From<Transaction> for ApiTransaction {
    fn from(t: Transaction) -> Self {
        let mut api_t = ApiTransaction {
            // Common fields
            fee: num_as_api_option(t.fee.0).map(MicroAlgos),
            first_valid: num_as_api_option(t.first_valid.0).map(Round),
            genesis_id: t.genesis_id.clone().and_then(str_as_api_option),
            genesis_hash: t.genesis_hash,
            group: t.group,
            last_valid: num_as_api_option(t.last_valid.0).map(Round),
            lease: t.lease,
            note: t.note.clone().and_then(vec_as_api_option),
            rekey_to: t.rekey_to,
            sender: t.sender(),
            type_: to_tx_type_enum(&t.txn_type).to_api_str().to_owned(),
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
            extra_pages: None,

            state_proof_type: None,
            state_proof: None,
            state_proof_message: None,
        };

        match &t.txn_type {
            TransactionType::Payment(payment) => {
                api_t.receiver = Some(payment.receiver);
                api_t.amount = num_as_api_option(payment.amount.0);
                api_t.close_reminder_to = payment.close_remainder_to;
            }
            TransactionType::KeyRegistration(reg) => {
                api_t.vote_pk = reg.vote_pk;
                api_t.selection_pk = reg.selection_pk;
                api_t.vote_first = reg.vote_first;
                api_t.vote_last = reg.vote_last;
                api_t.vote_key_dilution = reg.vote_key_dilution.and_then(num_as_api_option);
                api_t.nonparticipating = reg.nonparticipating.and_then(bool_as_api_option);
            }
            TransactionType::AssetConfigurationTransaction(config) => {
                api_t.asset_params = config.to_owned().params.map(|p| p.into());
                api_t.config_asset = config.config_asset.and_then(num_as_api_option);
            }
            TransactionType::AssetTransferTransaction(transfer) => {
                api_t.xfer = num_as_api_option(transfer.xfer);
                api_t.asset_amount = num_as_api_option(transfer.amount);
                api_t.asset_receiver = Some(transfer.receiver);
                api_t.asset_close_to = transfer.close_to;
            }
            TransactionType::AssetAcceptTransaction(accept) => {
                api_t.xfer = Some(accept.xfer);
                api_t.asset_receiver = Some(accept.sender);
            }
            TransactionType::AssetClawbackTransaction(clawback) => {
                api_t.xfer = Some(clawback.xfer);
                api_t.asset_amount = num_as_api_option(clawback.asset_amount);
                api_t.asset_sender = Some(clawback.asset_sender);
                api_t.asset_receiver = Some(clawback.asset_receiver);
                api_t.asset_close_to = clawback.asset_close_to;
            }
            TransactionType::AssetFreezeTransaction(freeze) => {
                api_t.freeze_account = Some(freeze.freeze_account);
                api_t.asset_id = num_as_api_option(freeze.asset_id);
                api_t.frozen = bool_as_api_option(freeze.frozen);
            }
            TransactionType::ApplicationCallTransaction(call) => {
                api_t.app_id = call.app_id.and_then(num_as_api_option);
                api_t.on_complete =
                    num_as_api_option(application_call_on_complete_to_int(&call.on_complete));
                api_t.accounts = call.accounts.clone().and_then(vec_as_api_option);
                api_t.approval_program = call
                    .approval_program
                    .to_owned()
                    .map(|c| c.0)
                    .and_then(vec_as_api_option);
                api_t.app_arguments = call
                    .app_arguments
                    .to_owned()
                    .map(|args| args.into_iter().map(AppArgument).collect())
                    .and_then(vec_as_api_option);
                api_t.clear_state_program = call
                    .clear_state_program
                    .to_owned()
                    .map(|c| c.0)
                    .and_then(vec_as_api_option);
                api_t.foreign_apps = call.foreign_apps.clone().and_then(vec_as_api_option);
                api_t.foreign_assets = call.foreign_assets.clone().and_then(vec_as_api_option);
                api_t.global_state_schema =
                    call.to_owned().global_state_schema.and_then(|s| s.into());
                api_t.local_state_schema =
                    call.to_owned().local_state_schema.and_then(|s| s.into());
                api_t.extra_pages = num_as_api_option(call.extra_pages);
            }
            TransactionType::StateProofTransaction(stpf) => {
                api_t.state_proof_type = {
                    let stpf = stpf.state_proof_type;
                    match stpf {
                        StateProofType::StateProofBasic => Some(0),
                    }
                };
                //api_t.state_proof = {
                //    1;
                //};
                //api_t.state_proof_message = {

                //};
            }
        }
        api_t
    }
}

impl TryFrom<ApiTransaction> for Transaction {
    type Error = TransactionError;

    fn try_from(api_t: ApiTransaction) -> Result<Self, Self::Error> {
        let txn_type = match api_t.type_.as_ref() {
            "pay" => TransactionType::Payment(Payment {
                sender: api_t.sender,
                receiver: api_t.receiver.ok_or_else(|| {
                    TransactionError::Deserialization("receiver missing".to_owned())
                })?,
                amount: MicroAlgos(num_from_api_option(api_t.amount)),
                close_remainder_to: api_t.close_reminder_to,
            }),
            "keyreg" => TransactionType::KeyRegistration(KeyRegistration {
                sender: api_t.sender,
                vote_pk: api_t.vote_pk,
                selection_pk: api_t.selection_pk,
                vote_first: api_t.vote_first,
                vote_last: api_t.vote_last,
                vote_key_dilution: Some(num_from_api_option(api_t.vote_key_dilution)),
                nonparticipating: api_t.nonparticipating,
            }),
            "acfg" => {
                TransactionType::AssetConfigurationTransaction(AssetConfigurationTransaction {
                    sender: api_t.sender,
                    params: api_t.asset_params.map(|p| p.into()),
                    // None is not mapped to "zero value": the possible "zero value" (asset creation) is represented as None in the domain.
                    config_asset: api_t.config_asset,
                })
            }
            "axfer" => parse_asset_transfer_transaction(&api_t)?,
            "afrz" => TransactionType::AssetFreezeTransaction(AssetFreezeTransaction {
                sender: api_t.sender,
                freeze_account: api_t.freeze_account.ok_or_else(|| {
                    TransactionError::Deserialization("freeze_account missing".to_owned())
                })?,
                asset_id: api_t.asset_id.ok_or_else(|| {
                    TransactionError::Deserialization("asset_id missing".to_owned())
                })?,
                frozen: bool_from_api_option(api_t.frozen),
            }),
            "appl" => {
                let on_complete =
                    int_to_application_call_on_complete(num_from_api_option(api_t.on_complete))?;
                TransactionType::ApplicationCallTransaction(ApplicationCallTransaction {
                    sender: api_t.sender,
                    app_id: api_t.app_id,
                    on_complete: on_complete.clone(),
                    accounts: api_t.accounts,
                    approval_program: api_t.approval_program.map(CompiledTeal),
                    app_arguments: api_t
                        .app_arguments
                        .map(|args| args.into_iter().map(|a| a.0).collect()),
                    clear_state_program: api_t.clear_state_program.map(CompiledTeal),
                    foreign_apps: api_t.foreign_apps,
                    foreign_assets: api_t.foreign_assets,

                    global_state_schema: parse_state_schema(
                        on_complete.clone(),
                        api_t.app_id,
                        api_t.global_state_schema,
                    ),
                    local_state_schema: parse_state_schema(
                        on_complete,
                        api_t.app_id,
                        api_t.local_state_schema,
                    ),

                    extra_pages: num_from_api_option(api_t.extra_pages),
                })
            }
            "stpf" => {
                TransactionType::StateProofTransaction(StateProofTransaction {
                    sender: api_t.sender,
                    state_proof_type: StateProofType::StateProofBasic,
                    //state_proof: api_t.state_proof,
                    //message: api_t.state_proof_message,
                })
            }

            unsupported_type => {
                return Err(TransactionError::Deserialization(format!(
                    "Not supported transaction type: {}",
                    unsupported_type
                )))
            }
        };
        Ok(Transaction {
            fee: MicroAlgos(num_from_api_option(api_t.fee.map(|f| f.0))),
            first_valid: Round(num_from_api_option(api_t.first_valid.map(|r| r.0))),
            genesis_id: api_t.genesis_id,
            genesis_hash: api_t.genesis_hash,
            group: api_t.group,
            last_valid: Round(num_from_api_option(api_t.last_valid.map(|r| r.0))),
            lease: api_t.lease,
            note: api_t.note.clone(),
            rekey_to: api_t.rekey_to,
            txn_type,
        })
    }
}

fn parse_state_schema(
    on_complete: ApplicationCallOnComplete,
    app_id: Option<u64>,
    api_state_schema: Option<ApiStateSchema>,
) -> Option<StateSchema> {
    match (on_complete, app_id) {
        // App creation (has schema)
        (ApplicationCallOnComplete::NoOp, None) => Some(
            api_state_schema
                .map(|s| s.into())
                // on creation we know that there's a schema, so we map None to schema with 0 values.
                // The API sends None because struct with 0s is considered a "zero value" and skipped.
                .unwrap_or_else(|| StateSchema {
                    number_ints: 0,
                    number_byteslices: 0,
                }),
        ),
        // Not app creation (has no schema)
        _ => None,
    }
}

fn parse_asset_transfer_transaction(
    api_t: &ApiTransaction,
) -> Result<TransactionType, TransactionError> {
    match (
        api_t.xfer,
        api_t.asset_sender,
        api_t.asset_receiver,
        api_t.asset_amount,
    ) {
        (Some(xfer), Some(asset_sender), Some(asset_receiver), Some(asset_amount)) => Ok(
            TransactionType::AssetClawbackTransaction(AssetClawbackTransaction {
                sender: api_t.sender,
                xfer,
                asset_amount,
                asset_sender,
                asset_receiver,
                asset_close_to: api_t.asset_close_to,
            }),
        ),
        (Some(xfer), None, Some(asset_receiver), None) if asset_receiver == api_t.sender => Ok(
            TransactionType::AssetAcceptTransaction(AssetAcceptTransaction {
                sender: api_t.sender,
                xfer,
            }),
        ),
        (Some(xfer), None, Some(asset_receiver), asset_amount) => Ok(
            TransactionType::AssetTransferTransaction(AssetTransferTransaction {
                sender: api_t.sender,
                xfer,
                amount: num_from_api_option(asset_amount),
                receiver: asset_receiver,
                close_to: api_t.asset_close_to,
            }),
        ),
        _ => Err(TransactionError::Deserialization(format!(
            "Invalid api asset transfer transaction: {:?}",
            api_t
        ))),
    }
}

impl TryFrom<ApiSignedTransaction> for SignedTransaction {
    type Error = TransactionError;

    fn try_from(api_t: ApiSignedTransaction) -> Result<Self, Self::Error> {
        Ok(SignedTransaction {
            transaction: api_t.transaction.clone().try_into()?,
            transaction_id: api_t.transaction_id.clone(),
            sig: transaction_signature(&api_t)?,
            auth_address: None,
        })
    }
}

fn transaction_signature(
    api_t: &ApiSignedTransaction,
) -> Result<TransactionSignature, TransactionError> {
    match (&api_t.sig, &api_t.lsig, &api_t.msig) {
        (Some(sig), None, None) => Ok(TransactionSignature::Single(*sig)),
        (None, Some(lsig), None) => Ok(TransactionSignature::Logic(lsig.clone().try_into()?)),
        (None, None, Some(msig)) => Ok(TransactionSignature::Multi(msig.clone())),
        _ => Err(TransactionError::Deserialization(format!(
            "Invalid sig combination: {:?}",
            api_t
        ))),
    }
}
impl From<AssetParams> for ApiAssetParams {
    fn from(params: AssetParams) -> Self {
        ApiAssetParams {
            asset_name: params.asset_name.and_then(str_as_api_option),
            decimals: params.decimals.and_then(num_as_api_option),
            default_frozen: params.default_frozen.and_then(bool_as_api_option),
            total: params.total.and_then(num_as_api_option),
            unit_name: params.unit_name.and_then(str_as_api_option),
            meta_data_hash: params.meta_data_hash.and_then(vec_as_api_option),
            url: params.url.and_then(str_as_api_option),
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
            decimals: Some(num_from_api_option(params.decimals)),
            default_frozen: Some(bool_from_api_option(params.default_frozen)),
            total: Some(num_from_api_option(params.total)),
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
            auth_address: t.auth_address,
        }
    }
}

impl From<StateSchema> for Option<ApiStateSchema> {
    fn from(state_schema: StateSchema) -> Self {
        match state_schema {
            StateSchema {
                number_ints: 0,
                number_byteslices: 0,
            } => None,
            _ => Some(ApiStateSchema {
                number_ints: num_as_api_option(state_schema.number_ints),
                number_byteslices: num_as_api_option(state_schema.number_byteslices),
            }),
        }
    }
}

impl From<ApiStateSchema> for StateSchema {
    fn from(state_schema: ApiStateSchema) -> Self {
        StateSchema {
            number_ints: num_from_api_option(state_schema.number_ints),
            number_byteslices: num_from_api_option(state_schema.number_byteslices),
        }
    }
}

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
        api_transaction.try_into().map_err(serde::de::Error::custom)
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
        ApiSignedTransaction::deserialize(deserializer)?
            .try_into()
            .map_err(serde::de::Error::custom)
    }
}

impl From<SignedLogic> for ApiSignedLogic {
    fn from(s: SignedLogic) -> Self {
        let (sig, msig) = match s.sig {
            LogicSignature::ContractAccount => (None, None),
            LogicSignature::DelegatedSig(sig) => (Some(sig), None),
            LogicSignature::DelegatedMultiSig(msig) => (None, Some(msig)),
        };
        ApiSignedLogic {
            logic: s.logic.0,
            sig,
            msig,
            args: s.args.into_iter().map(ApiSignedLogicArg).collect(),
        }
    }
}

/// See [ApiTransaction] doc
fn num_as_api_option<T: Num>(n: T) -> Option<T> {
    if n.is_zero() {
        None
    } else {
        Some(n)
    }
}

/// See [ApiTransaction] doc
fn bool_as_api_option(b: bool) -> Option<bool> {
    if b {
        Some(b)
    } else {
        None
    }
}

/// See [ApiTransaction] doc
fn vec_as_api_option<T>(v: Vec<T>) -> Option<Vec<T>> {
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}

/// See [ApiTransaction] doc
fn str_as_api_option(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// See [ApiTransaction] doc
fn num_from_api_option<T: Num>(opt: Option<T>) -> T {
    opt.unwrap_or_else(T::zero)
}

/// See [ApiTransaction] doc
fn bool_from_api_option(opt: Option<bool>) -> bool {
    opt.unwrap_or(false)
}

fn application_call_on_complete_to_int(call: &ApplicationCallOnComplete) -> u32 {
    match call {
        ApplicationCallOnComplete::NoOp => 0,
        ApplicationCallOnComplete::OptIn => 1,
        ApplicationCallOnComplete::CloseOut => 2,
        ApplicationCallOnComplete::ClearState => 3,
        ApplicationCallOnComplete::UpdateApplication => 4,
        ApplicationCallOnComplete::DeleteApplication => 5,
    }
}

fn state_proof_type_to_int(stpf: &StateProofType) -> u8 {
    match stpf {
        StateProofType::StateProofBasic => 0,
    }
}

fn int_to_application_call_on_complete(
    i: u32,
) -> Result<ApplicationCallOnComplete, TransactionError> {
    match i {
        0 => Ok(ApplicationCallOnComplete::NoOp),
        1 => Ok(ApplicationCallOnComplete::OptIn),
        2 => Ok(ApplicationCallOnComplete::CloseOut),
        3 => Ok(ApplicationCallOnComplete::ClearState),
        4 => Ok(ApplicationCallOnComplete::UpdateApplication),
        5 => Ok(ApplicationCallOnComplete::DeleteApplication),
        _ => Err(TransactionError::Deserialization(format!(
            "Invalid on complete value: {}",
            i
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_signed_logic_contract_account() {
        let program = CompiledTeal(vec![
            0x01, 0x20, 0x01, 0x01, 0x22, // int 1
        ]);
        let args = vec![vec![1, 2, 3], vec![4, 5, 6]];
        let lsig = SignedLogic {
            logic: program.clone(),
            args,
            sig: LogicSignature::ContractAccount,
        };

        // TODO generic utility to test serialization. Test all api structs.
        // https://github.com/manuelmauro/algonaut/issues/67
        let api_lsig: ApiSignedLogic = lsig.clone().into();
        let serialized = rmp_serde::to_vec_named(&api_lsig).unwrap();
        let deserialized: ApiSignedLogic = rmp_serde::from_slice(&serialized).unwrap();
        let lsig_deserialized: SignedLogic = deserialized.try_into().unwrap();

        assert_eq!(lsig, lsig_deserialized);
    }

    #[test]
    fn test_serialize_signed_logic_contract_account_no_args() {
        let program = CompiledTeal(vec![
            0x01, 0x20, 0x01, 0x01, 0x22, // int 1
        ]);
        let args = vec![];
        let lsig = SignedLogic {
            logic: program.clone(),
            args,
            sig: LogicSignature::ContractAccount,
        };

        // TODO generic utility to test serialization. Test all api structs.
        // https://github.com/manuelmauro/algonaut/issues/67
        let api_lsig: ApiSignedLogic = lsig.clone().into();
        let serialized = rmp_serde::to_vec_named(&api_lsig).unwrap();
        let deserialized: ApiSignedLogic = rmp_serde::from_slice(&serialized).unwrap();
        let lsig_deserialized: SignedLogic = deserialized.try_into().unwrap();

        assert_eq!(lsig, lsig_deserialized);
    }
}
