// Lets the re-exported `abi_call!`/`abi_method!` macros — whose expansion
// names `::algonaut_abi::…` — resolve those paths inside this crate too (tests,
// doctests), exactly as a downstream crate would.
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

/// Compile-time checked ABI method-call macros. `abi_method!("…")` validates a
/// signature literal and yields an [`abi_interactions::AbiMethod`];
/// `abi_call!("…", args…)` additionally type-checks the arguments and yields a
/// [`MethodInvocation`]. Re-exported from the `algonaut_abi_macros` crate so
/// the path stays `algonaut_abi::abi_call!` (and `algonaut::abi::abi_call!`).
///
/// Both validate the signature literal at compile time:
///
/// ```
/// let _ = algonaut_abi::abi_method!("add(uint64,uint64)uint64");
/// let _ = algonaut_abi::abi_call!("add(uint64,uint64)uint64", 2u64, 3u64);
/// ```
///
/// A misspelled type is a build error, not a runtime one:
///
/// ```compile_fail
/// // `unt64` is not an ABI type — rejected by `cargo build`.
/// let _ = algonaut_abi::abi_method!("add(unt64,uint64)uint64");
/// ```
///
/// So is the wrong argument count:
///
/// ```compile_fail
/// // Two arguments expected, one supplied — a `format!`-style arity error.
/// let _ = algonaut_abi::abi_call!("add(uint64,uint64)uint64", 2u64);
/// ```
///
/// And so is an argument of the wrong type:
///
/// ```compile_fail
/// // `&str` cannot stand in for `uint64`: no `AbiArg<Uint<64>>` impl.
/// let _ = algonaut_abi::abi_call!("add(uint64,uint64)uint64", "two", 3u64);
/// ```
pub use algonaut_abi_macros::{abi_call, abi_method, contract};

#[doc(inline)]
pub use macro_support::MethodInvocation;

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
