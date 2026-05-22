//! Encoding an ABI [`MethodCall`] into its application-call transaction.
//!
//! [`process_method_call`] is the entry point used by
//! [`AtomicGroupBuilder::build`](super::AtomicGroupBuilder::build): it turns a
//! method call into an application-call transaction, packs reference
//! arguments into the call's foreign arrays, wraps any overflow past the
//! 15-argument limit into a tuple, and appends the result (plus any
//! transaction-typed arguments) to the group. [`validate_tx`] is the shared
//! per-transaction check applied to every slot, method call or not.
//!
//! Two helper structs carry the state that used to be threaded through
//! long argument lists: [`ForeignArrays`] accumulates the foreign
//! account/asset/app references, and [`EncodedArgs`] accumulates the
//! encoded ABI argument types and values.

use std::collections::HashMap;

use algonaut_abi::{
    abi_error::AbiError,
    abi_interactions::{AbiArgType, AbiMethod, ReferenceArgType, TransactionArgType},
    abi_type::{AbiType, AbiValue},
    make_tuple_type,
};
use algonaut_core::{Address, AppId, AssetId};
use algonaut_transaction::{
    Transaction, TransactionType,
    transaction::{ApplicationCallTransaction, to_tx_type_enum},
};
use num_bigint::BigUint;
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
    // Destructure once so every field is moved exactly where it is needed,
    // rather than cloned out of a borrowed `call`.
    let MethodCall {
        app_id,
        mut method,
        method_args,
        fee,
        sender,
        first_valid,
        last_valid,
        genesis_hash,
        genesis_id,
        on_complete,
        approval_program,
        clear_program,
        global_schema,
        local_schema,
        extra_pages,
        note,
        lease,
        rekey_to,
        signer,
        boxes,
    } = call;

    if method_args.len() != method.args.len() {
        return Err(Error::AbiArgumentCountMismatch {
            expected: method.args.len(),
            actual: method_args.len(),
        });
    }
    if txs.len() + method.get_tx_count() > MAX_ATOMIC_GROUP_SIZE {
        return Err(Error::ComposerGroupFull {
            max: MAX_ATOMIC_GROUP_SIZE,
        });
    }

    let mut foreign = ForeignArrays::default();
    let mut args = EncodedArgs::default();
    let mut tx_args: Vec<TransactionWithSigner> = Vec::new();

    for (arg_spec, arg_value) in method.args.iter_mut().zip(method_args) {
        match arg_spec.type_()? {
            AbiArgType::Tx(expected_type) => {
                let tx_with_signer = match arg_value {
                    AbiArgValue::TxWithSigner(tx_with_signer) => *tx_with_signer,
                    AbiArgValue::AbiValue(_) => return Err(Error::ExpectedTransactionArgument),
                };
                validate_tx(&tx_with_signer.tx, expected_type)?;
                tx_args.push(tx_with_signer);
            }
            AbiArgType::Ref(ref_type) => {
                let index = foreign.add_ref(&ref_type, &arg_value, sender, app_id)?;
                args.push_ref_index(index)?;
            }
            AbiArgType::AbiObj(abi_type) => match arg_value {
                AbiArgValue::AbiValue(value) => args.push(abi_type, value),
                AbiArgValue::TxWithSigner(_) => {
                    return Err(Error::InvalidAbiArgument {
                        expected: "ABI value",
                        actual: "transaction".to_owned(),
                    });
                }
            },
        }
    }

    args.wrap_overflow()?;
    let app_arguments = args.encode(method.get_selector()?)?;

    let ForeignArrays {
        accounts,
        assets,
        apps,
    } = foreign;

    let app_call = TransactionType::ApplicationCallTransaction(ApplicationCallTransaction {
        sender,
        app_id: Some(app_id),
        on_complete,
        accounts: Some(accounts),
        approval_program,
        app_arguments: Some(app_arguments),
        clear_state_program: clear_program,
        foreign_apps: Some(apps),
        foreign_assets: Some(assets),
        global_state_schema: global_schema,
        local_state_schema: local_schema,
        extra_pages,
        // The builder uses an empty vec for "no boxes"; the transaction
        // model wants `None`.
        boxes: (!boxes.is_empty()).then_some(boxes),
    });

    let tx = Transaction {
        fee,
        first_valid,
        genesis_hash,
        last_valid,
        txn_type: app_call,
        genesis_id: Some(genesis_id),
        group: None,
        lease,
        note: (!note.is_empty()).then_some(note),
        rekey_to,
    };

    txs.append(&mut tx_args);
    txs.push(TransactionWithSigner {
        tx,
        signer: Some(signer),
    });
    method_map.insert(txs.len() - 1, method);

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
        return Err(Error::TransactionAlreadyGrouped);
    }

    let actual_type = to_tx_type_enum(&tx.txn_type);
    if expected_type != TransactionArgType::Any
        && expected_type != TransactionArgType::One(actual_type.clone())
    {
        return Err(Error::TransactionTypeMismatch {
            expected: format!("{expected_type:?}"),
            actual: format!("{actual_type:?}"),
        });
    }

    Ok(())
}

/// Accumulates foreign-array references during ABI method encoding. The
/// three arrays correspond to an application call's `accounts`,
/// `foreign_assets`, and `foreign_apps`.
#[derive(Default)]
struct ForeignArrays {
    accounts: Vec<Address>,
    assets: Vec<AssetId>,
    apps: Vec<AppId>,
}

impl ForeignArrays {
    /// Add a reference argument to its respective foreign array and return
    /// the index that can be used to reference it (in TEAL).
    fn add_ref(
        &mut self,
        arg_type: &ReferenceArgType,
        arg_value: &AbiArgValue,
        sender: Address,
        app_id: AppId,
    ) -> Result<usize, Error> {
        match arg_type {
            ReferenceArgType::Account => {
                let address = Address::try_from(arg_value)?;
                Ok(populate_foreign_array(
                    address,
                    &mut self.accounts,
                    Some(sender),
                ))
            }
            ReferenceArgType::Asset => {
                let asset_id = ref_arg_u64(arg_value)?;
                Ok(populate_foreign_array(
                    AssetId(asset_id),
                    &mut self.assets,
                    None,
                ))
            }
            ReferenceArgType::Application => {
                let referenced_app = ref_arg_u64(arg_value)?;
                Ok(populate_foreign_array(
                    AppId(referenced_app),
                    &mut self.apps,
                    Some(app_id),
                ))
            }
        }
    }
}

/// Extract a `u64` foreign-array index (asset or app id) from an
/// integer-typed ABI argument.
fn ref_arg_u64(arg_value: &AbiArgValue) -> Result<u64, Error> {
    let int = BigUint::try_from(arg_value)?;
    int.to_u64()
        .ok_or_else(|| AbiError::ValueOutOfRange {
            abi_type: "uint64".to_owned(),
            reason: format!("value {int} exceeds u64 capacity"),
        })
        .map_err(Error::from)
}

/// Accumulates encoded ABI argument types and values during method
/// encoding, then encodes them (prefixed by the method selector) into the
/// application call's `app_arguments`.
#[derive(Default)]
struct EncodedArgs {
    types: Vec<AbiType>,
    values: Vec<AbiValue>,
}

impl EncodedArgs {
    /// Append a plain ABI argument.
    fn push(&mut self, ty: AbiType, value: AbiValue) {
        self.types.push(ty);
        self.values.push(value);
    }

    /// Append a foreign-array index argument (a `uint8` referencing a slot
    /// in `accounts`/`foreign_assets`/`foreign_apps`).
    fn push_ref_index(&mut self, index: usize) -> Result<(), Error> {
        self.types.push(AbiType::uint(FOREIGN_OBJ_ABI_UINT_SIZE)?);
        self.values.push(AbiValue::Int(index.into()));
        Ok(())
    }

    /// If more than 15 ABI arguments were collected, pack everything from
    /// index 14 onward into a single trailing tuple, leaving 14 direct
    /// arguments plus the tuple — the 15 application-argument slots the
    /// 4-byte selector does not occupy. Mirrors py-algorand-sdk's
    /// `AtomicTransactionComposer.add_method_call`.
    fn wrap_overflow(&mut self) -> Result<(), Error> {
        if self.values.len() <= MAX_ABI_ARG_TYPE_LEN {
            return Ok(());
        }

        // `split_off` truncates the direct arguments to the first 14 and
        // hands back the overflow to fold into the tuple — without this
        // truncation the encoded call would carry more than 15 app args.
        let split_at = MAX_ABI_ARG_TYPE_LEN - 1;
        let wrapped_types = self.types.split_off(split_at);
        let wrapped_values = self.values.split_off(split_at);

        let tuple_type = make_tuple_type(&wrapped_types)?;
        self.types.push(tuple_type);
        self.values.push(AbiValue::Array(wrapped_values));
        Ok(())
    }

    /// Encode the collected arguments, prefixed by the 4-byte method
    /// selector, into the `app_arguments` byte vectors. Consumes `self`,
    /// moving each value into [`AbiType::encode`] rather than cloning it.
    fn encode(self, selector: [u8; 4]) -> Result<Vec<Vec<u8>>, Error> {
        let mut encoded = Vec::with_capacity(self.values.len() + 1);
        encoded.push(selector.to_vec());
        for (ty, value) in self.types.iter().zip(self.values) {
            encoded.push(ty.encode(value)?);
        }
        Ok(encoded)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// More than 15 ABI arguments must collapse to 14 direct args plus one
    /// trailing tuple holding the overflow — 15 application-argument slots,
    /// leaving the 16th for the method selector. (The selector is prepended
    /// later, by `EncodedArgs::encode`.) Matches py-algorand-sdk's
    /// `AtomicTransactionComposer.add_method_call`.
    #[test]
    fn wrap_overflow_packs_args_past_15_into_a_trailing_tuple() {
        let mut args = EncodedArgs::default();
        for i in 0..16u8 {
            args.push(AbiType::Byte, AbiValue::Byte(i));
        }

        args.wrap_overflow().unwrap();

        assert_eq!(args.types.len(), 15, "must be 14 direct args + 1 tuple");
        assert_eq!(args.values.len(), 15);

        // The first 14 slots are the original args, untouched and in order.
        for (i, value) in args.values.iter().take(14).enumerate() {
            assert_eq!(*value, AbiValue::Byte(i as u8));
        }

        // The trailing slot is a 2-element tuple holding original args 14
        // and 15.
        match (&args.types[14], &args.values[14]) {
            (AbiType::Tuple { len, child_types }, AbiValue::Array(items)) => {
                assert_eq!(*len, 2);
                assert_eq!(child_types.len(), 2);
                assert_eq!(items, &vec![AbiValue::Byte(14), AbiValue::Byte(15)]);
            }
            other => panic!("expected a trailing 2-tuple, got {other:?}"),
        }
    }

    /// Exactly 15 arguments fit without wrapping: no tuple is introduced.
    #[test]
    fn wrap_overflow_leaves_15_or_fewer_args_untouched() {
        let mut args = EncodedArgs::default();
        for i in 0..15u8 {
            args.push(AbiType::Byte, AbiValue::Byte(i));
        }

        args.wrap_overflow().unwrap();

        assert_eq!(args.values.len(), 15);
        assert!(
            args.values.iter().all(|v| matches!(v, AbiValue::Byte(_))),
            "no tuple should be introduced at the boundary"
        );
    }
}
