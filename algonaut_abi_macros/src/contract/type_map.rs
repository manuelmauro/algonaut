//! Maps ABI type strings to Rust types for generated method parameters.
//!
//! The initial implementation covers scalar types that have canonical Rust
//! representations. Unsupported types (transaction/reference args, compound
//! types) return an error with guidance to use the dynamic path.

use algonaut_abi_sig::SigType;
use proc_macro2::TokenStream;
use quote::quote;

/// Maps an ABI type to the Rust type to use in generated method parameters.
/// Returns the type as a TokenStream for use in code generation.
pub fn rust_param_type(ty: &SigType) -> Result<TokenStream, String> {
    match ty {
        SigType::UInt { bit_size } => uint_to_rust(*bit_size),
        SigType::Byte => Ok(quote! { u8 }),
        SigType::Bool => Ok(quote! { bool }),
        SigType::Address => Ok(quote! { ::algonaut_core::Address }),
        SigType::String => Ok(quote! { ::std::string::String }),
        // byte[] is the one compound type with a canonical Rust representation
        SigType::DynamicArray { child_type } if matches!(**child_type, SigType::Byte) => {
            Ok(quote! { ::std::vec::Vec<u8> })
        }
        SigType::UFixed {
            bit_size,
            precision,
        } => Err(format!(
            "ufixed{bit_size}x{precision} (no canonical Rust type)"
        )),
        SigType::StaticArray { .. } => Err("static array".to_owned()),
        SigType::DynamicArray { .. } => Err("dynamic array".to_owned()),
        SigType::Tuple { .. } => Err("tuple".to_owned()),
    }
}

/// Maps a uint bit size to the appropriate Rust unsigned integer type.
fn uint_to_rust(bit_size: u16) -> Result<TokenStream, String> {
    match bit_size {
        8 => Ok(quote! { u8 }),
        16 => Ok(quote! { u16 }),
        32 => Ok(quote! { u32 }),
        64 => Ok(quote! { u64 }),
        128 => Ok(quote! { u128 }),
        // uint256+ uses BigUint
        n if n > 128 && n <= 512 => Ok(quote! { ::num_bigint::BigUint }),
        n => Err(format!("uint{n} (unsupported bit size)")),
    }
}

/// Maps an ABI type to the marker type for use with AbiArg trait bounds.
/// This is used to generate the encoding call for method arguments.
pub fn abi_marker_type(ty: &SigType) -> Result<TokenStream, String> {
    match ty {
        SigType::UInt { bit_size } => {
            let bits = proc_macro2::Literal::u16_unsuffixed(*bit_size);
            Ok(quote! { ::algonaut_abi::macro_support::Uint<#bits> })
        }
        SigType::Byte => Ok(quote! { ::algonaut_abi::macro_support::Byte }),
        SigType::Bool => Ok(quote! { ::algonaut_abi::macro_support::Bool }),
        SigType::Address => Ok(quote! { ::algonaut_abi::macro_support::Address }),
        SigType::String => Ok(quote! { ::algonaut_abi::macro_support::AbiString }),
        SigType::DynamicArray { child_type } if matches!(**child_type, SigType::Byte) => {
            Ok(quote! { ::algonaut_abi::macro_support::Bytes })
        }
        other => Err(unsupported_type_message(other)),
    }
}

fn unsupported_type_message(ty: &SigType) -> String {
    let type_name = match ty {
        SigType::UFixed { .. } => "ufixed",
        SigType::StaticArray { .. } => "static array",
        SigType::DynamicArray { .. } => "dynamic array",
        SigType::Tuple { .. } => "tuple",
        _ => "unsupported type",
    };
    format!("{type_name}; use MethodCall::builder().invoke(Invocation::new(...)) for this method")
}
