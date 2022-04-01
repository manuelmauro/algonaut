mod abi_encode;
mod abi_encode_test;
pub mod abi_error;
mod abi_interaction_tests;
pub mod abi_interactions;
mod abi_json_tests;
pub mod abi_type;
mod abi_type_test;
mod biguint_ext;

use crate::abi_error::AbiError;
use abi_type::AbiType;

/// MakeTupleType makes tuple ABI type by taking an array of tuple element types as argument.
pub fn make_tuple_type(argument_types: &[AbiType]) -> Result<AbiType, AbiError> {
    if argument_types.is_empty() {
        return Err(AbiError::Msg(
            "tuple must contain at least one type".to_string(),
        ));
    }

    if argument_types.len() >= u16::MAX as usize {
        return Err(AbiError::Msg(
            "tuple type child type number larger than maximum uint16 error".to_string(),
        ));
    }

    let mut strs = vec![];
    for arg in argument_types {
        strs.push(arg.string()?)
    }

    let str_tuple = format!("({})", strs.join(","));
    str_tuple.parse()
}
