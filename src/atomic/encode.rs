//! Encoding an ABI [`MethodCall`] into its application-call transaction.
//!
//! [`process_method_call`] is the entry point used by
//! [`AtomicGroupBuilder::build`](super::AtomicGroupBuilder::build): it turns a
//! method call into an application-call transaction, packs reference
//! arguments into the call's foreign arrays, wraps any overflow past the
//! 15-argument limit into a tuple, and appends the result (plus any
//! transaction-typed arguments) to the group. [`validate_tx`] is the shared
//! per-transaction check applied to every slot, method call or not.

use std::collections::HashMap;

use algonaut_abi::{
    abi_error::AbiError,
    abi_interactions::{AbiArgType, AbiMethod, ReferenceArgType, TransactionArgType},
    abi_type::{AbiType, AbiValue},
    make_tuple_type,
};
use algonaut_core::{Address, AppId, AssetId, Round};
use algonaut_transaction::{
    Transaction, TransactionType,
    builder::TransactionParams,
    transaction::{ApplicationCallTransaction, to_tx_type_enum},
};
use num_traits::ToPrimitive;

use crate::Error;

use super::{AbiArgValue, MAX_ATOMIC_GROUP_SIZE, MethodCall, TransactionWithSigner};

// if the abi type argument number > 15, then the abi types after 14th should be wrapped in a tuple
const MAX_ABI_ARG_TYPE_LEN: usize = 15;

const FOREIGN_OBJ_ABI_UINT_SIZE: usize = 8;

/// Encode an ABI method call into its application-call transaction (plus
/// any transaction-typed arguments) and append the result to `txs`,
/// recording the method at the app-call's index in `method_map`.
pub(super) fn process_method_call(
    call: MethodCall,
    txs: &mut Vec<TransactionWithSigner>,
    method_map: &mut HashMap<usize, AbiMethod>,
) -> Result<(), Error> {
    if call.method_args.len() != call.method.args.len() {
        return Err(Error::Msg(format!(
            "incorrect number of arguments were provided: {} != {}",
            call.method_args.len(),
            call.method.args.len()
        )));
    }
    if txs.len() + call.method.get_tx_count() > MAX_ATOMIC_GROUP_SIZE {
        return Err(Error::ComposerGroupFull {
            max: MAX_ATOMIC_GROUP_SIZE,
        });
    }

    let mut method_types = vec![];
    let mut method_args: Vec<AbiValue> = vec![];
    let mut txs_with_signer = vec![];
    let mut foreign_accounts = vec![];
    let mut foreign_assets = vec![];
    let mut foreign_apps = vec![];

    for (arg_type, arg_value) in call.method.args.iter().zip(&call.method_args) {
        let mut arg_type = arg_type.clone();

        match arg_type.type_()? {
            AbiArgType::Tx(type_) => {
                add_tx_arg_type_to_method_call(arg_value, type_, &mut txs_with_signer)?
            }
            AbiArgType::Ref(type_) => add_ref_arg_to_method_call(
                &type_,
                arg_value,
                &mut foreign_accounts,
                &mut foreign_assets,
                &mut foreign_apps,
                &mut method_types,
                &mut method_args,
                call.sender,
                call.app_id,
            )?,
            AbiArgType::AbiObj(type_) => {
                add_abi_obj_arg_to_method_call(
                    &type_,
                    arg_value,
                    &mut method_types,
                    &mut method_args,
                )?;
            }
        };
    }

    if method_args.len() > MAX_ABI_ARG_TYPE_LEN {
        let (type_, value) = wrap_overflowing_values(&method_types, &method_args)?;
        method_types.push(type_);
        method_args.push(value);
    }

    let mut encoded_abi_args = vec![call.method.get_selector()?.into()];
    for (method_type, method_arg) in method_types.iter().zip(&method_args) {
        encoded_abi_args.push(method_type.encode(method_arg.clone())?);
    }

    let app_call = TransactionType::ApplicationCallTransaction(ApplicationCallTransaction {
        sender: call.sender,
        app_id: Some(call.app_id),
        on_complete: call.on_complete.clone(),
        accounts: Some(foreign_accounts),
        approval_program: call.approval_program.clone(),
        app_arguments: Some(encoded_abi_args),
        clear_state_program: call.clear_program.clone(),
        foreign_apps: Some(foreign_apps),
        foreign_assets: Some(foreign_assets),
        global_state_schema: call.global_schema.clone(),
        local_state_schema: call.local_schema.clone(),
        extra_pages: call.extra_pages,
        boxes: call.boxes.clone(),
    });

    let sp = &call.suggested_params;
    let tx = Transaction {
        fee: call.fee,
        first_valid: Round(sp.last_round()),
        genesis_hash: sp.genesis_hash(),
        last_valid: Round(sp.last_round() + 1000),
        txn_type: app_call,
        genesis_id: Some(sp.genesis_id().clone()),
        group: None,
        lease: call.lease,
        note: call.note.clone(),
        rekey_to: call.rekey_to,
    };

    txs.append(&mut txs_with_signer);
    txs.push(TransactionWithSigner {
        tx,
        signer: Some(call.signer.clone()),
    });
    method_map.insert(txs.len() - 1, call.method);

    Ok(())
}

/// Shared per-transaction validity check: the transaction must carry no
/// group id yet, and (when `expected_type` is not
/// [`TransactionArgType::Any`]) must match the expected transaction type.
pub(super) fn validate_tx(
    tx: &Transaction,
    expected_type: TransactionArgType,
) -> Result<(), Error> {
    if tx.group.is_some() {
        return Err(Error::Msg("Expected empty group id".to_owned()));
    }

    if expected_type != TransactionArgType::Any
        && expected_type != TransactionArgType::One(to_tx_type_enum(&tx.txn_type))
    {
        return Err(Error::Msg(format!(
            "expected transaction with type {expected_type:?}, but got type {:?}",
            tx.txn_type
        )));
    }

    Ok(())
}

fn add_tx_arg_type_to_method_call(
    arg_value: &AbiArgValue,
    expected_type: TransactionArgType,
    txs_with_signer: &mut Vec<TransactionWithSigner>,
) -> Result<(), Error> {
    let txn_and_signer = match arg_value {
        AbiArgValue::TxWithSigner(tx_with_signer) => tx_with_signer,
        _ => {
            return Err(Error::Msg(
                "invalid arg value, expected transaction".to_owned(),
            ));
        }
    };

    validate_tx(&txn_and_signer.tx, expected_type)?;
    txs_with_signer.push(txn_and_signer.to_owned());

    Ok(())
}

fn add_abi_obj_arg_to_method_call(
    abi_type: &AbiType,
    arg_value: &AbiArgValue,
    method_types: &mut Vec<AbiType>,
    method_args: &mut Vec<AbiValue>,
) -> Result<(), Error> {
    match arg_value {
        AbiArgValue::AbiValue(value) => {
            method_types.push(abi_type.clone());
            method_args.push(value.clone());
        }
        AbiArgValue::TxWithSigner(_) => {
            return Err(Error::Msg(
                "Invalid state: shouldn't be here with a tx with signer value type".to_owned(),
            ));
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn add_ref_arg_to_method_call(
    arg_type: &ReferenceArgType,
    arg_value: &AbiArgValue,

    foreign_accounts: &mut Vec<Address>,
    foreign_assets: &mut Vec<AssetId>,
    foreign_apps: &mut Vec<AppId>,

    method_types: &mut Vec<AbiType>,
    method_args: &mut Vec<AbiValue>,

    sender: Address,
    app_id: AppId,
) -> Result<(), Error> {
    let index = add_to_foreign_array(
        arg_type,
        arg_value,
        foreign_accounts,
        foreign_assets,
        foreign_apps,
        sender,
        app_id,
    )?;

    method_types.push(AbiType::uint(FOREIGN_OBJ_ABI_UINT_SIZE)?);
    method_args.push(AbiValue::Int(index.into()));

    Ok(())
}

/// Adds arg value to its respective foreign array
/// Returns index that can be used to reference `arg_value` in its foreign array (in TEAL).
fn add_to_foreign_array(
    arg_type: &ReferenceArgType,
    arg_value: &AbiArgValue,
    foreign_accounts: &mut Vec<Address>,
    foreign_assets: &mut Vec<AssetId>,
    foreign_apps: &mut Vec<AppId>,
    sender: Address,
    app_id: AppId,
) -> Result<usize, Error> {
    match arg_type {
        ReferenceArgType::Account => match arg_value.address() {
            Some(address) => Ok(populate_foreign_array(
                address,
                foreign_accounts,
                Some(sender),
            )),
            _ => Err(Error::Msg(format!(
                "Invalid value type: {arg_value:?} for arg type: {arg_type:?}"
            ))),
        },
        ReferenceArgType::Asset => match arg_value.int() {
            Some(int) => {
                let intu64 = int.to_u64().ok_or_else(|| {
                    AbiError::ValueOutOfRange {
                        abi_type: "uint64".to_owned(),
                        reason: format!("value {int} exceeds u64 capacity"),
                    }
                })?;

                Ok(populate_foreign_array(
                    AssetId(intu64),
                    foreign_assets,
                    None,
                ))
            }
            _ => Err(Error::Msg(format!(
                "Invalid value type: {arg_value:?} for arg type: {arg_type:?}"
            ))),
        },
        ReferenceArgType::Application => match arg_value.int() {
            Some(int) => {
                let intu64 = int.to_u64().ok_or_else(|| {
                    AbiError::ValueOutOfRange {
                        abi_type: "uint64".to_owned(),
                        reason: format!("value {int} exceeds u64 capacity"),
                    }
                })?;

                Ok(populate_foreign_array(
                    AppId(intu64),
                    foreign_apps,
                    Some(app_id),
                ))
            }
            _ => Err(Error::Msg(format!(
                "Invalid value type: {arg_value:?} for arg type: {arg_type:?}"
            ))),
        },
    }
}

fn wrap_overflowing_values(
    method_types: &[AbiType],
    method_args: &[AbiValue],
) -> Result<(AbiType, AbiValue), Error> {
    let mut wrapped_abi_types = vec![];
    let mut wrapped_value_list = vec![];

    for i in (MAX_ABI_ARG_TYPE_LEN - 1)..method_args.len() {
        wrapped_abi_types.push(method_types[i].clone());
        wrapped_value_list.push(method_args[i].clone());
    }

    let tuple_type = make_tuple_type(&wrapped_abi_types)?;

    Ok((tuple_type, AbiValue::Array(wrapped_value_list)))
}

/// Add a value to an application call's foreign array. The addition will be as compact as possible,
/// and this function will return an index that can be used to reference `object_to_add` in `obj_array`.
///
/// # Arguments
///
/// * `obj_to_add` - The value to add to the array. If this value is already present in the array,
///   it will not be added again. Instead, the existing index will be returned.
/// * `obj_array` - The existing foreign array. This input may be modified to append `obj_to_add`.
/// * `zeroth_obj` - If provided, this value indicated two things: the 0 value is special for this
///   array, so all indexes into `obj_array` must start at 1; additionally, if `obj_to_add` equals
///   `zeroth_obj`, then `obj_to_add` will not be added to the array, and instead the 0 indexes will be returned.
///
/// Returns an index that can be used to reference `obj_to_add` in `obj_array`.
fn populate_foreign_array<T: Eq>(
    obj_to_add: T,
    obj_array: &mut Vec<T>,
    zeroth_obj: Option<T>,
) -> usize {
    if let Some(o) = &zeroth_obj
        && &obj_to_add == o
    {
        return 0;
    }

    let start_from: usize = zeroth_obj.map(|_| 1).unwrap_or(0);
    let search_in_vec_index = obj_array.iter().position(|o| o == &obj_to_add);
    if let Some(index) = search_in_vec_index {
        start_from + index
    } else {
        obj_array.push(obj_to_add);
        obj_array.len() - 1 + start_from
    }
}
