// Lets `::algonaut_abi::…` paths — e.g. those named by the re-exported
// `contract!` macro's expansion, or by in-crate doctests — resolve inside this
// crate too, exactly as a downstream crate would.
extern crate self as algonaut_abi;

mod abi_encode;
mod abi_encode_test;
pub mod abi_error;
mod abi_interaction_tests;
pub mod abi_interactions;
mod abi_json_tests;
pub mod abi_type;
mod abi_type_test;
mod biguint_ext;
pub mod macro_support;
pub mod sourcemap;

use crate::abi_error::AbiError;
use abi_type::AbiType;

/// The [`contract!`](macro@contract) macro: generate a typed contract client
/// from an ARC-4 ABI or ARC-56 app-spec JSON file at compile time. Re-exported
/// from the `algonaut_abi_macros` crate so the path stays
/// `algonaut_abi::contract!` (and `algonaut::contract!`).
pub use algonaut_abi_macros::contract;

/// MakeTupleType makes tuple ABI type by taking an array of tuple element types as argument.
pub fn make_tuple_type(argument_types: &[AbiType]) -> Result<AbiType, AbiError> {
    if argument_types.is_empty() {
        return Err(AbiError::TypeParse {
            input: "()".to_owned(),
            reason: "tuple must contain at least one type".to_owned(),
        });
    }

    if argument_types.len() >= u16::MAX as usize {
        return Err(AbiError::TypeParse {
            input: format!("tuple with {} types", argument_types.len()),
            reason: "tuple type child type count exceeds uint16 maximum".to_owned(),
        });
    }

    let mut strs = vec![];
    for arg in argument_types {
        strs.push(arg.to_string())
    }

    let str_tuple = format!("({})", strs.join(","));
    str_tuple.parse()
}
