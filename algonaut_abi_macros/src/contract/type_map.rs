//! Maps ABI type strings to Rust types for generated method parameters.
//!
//! Covers the scalar types with canonical Rust representations, `byte[]`,
//! arrays (`T[]`/`T[N]`), anonymous tuples (`(T1,T2,…)` → Rust tuples), and
//! `ufixedNxM` (the [`Ufixed`](::algonaut_abi::macro_support::Ufixed) newtype).
//! A type the macro still can't model (e.g. a non-standard uint width) returns
//! an error with guidance to use the dynamic path.

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
        // Non-byte dynamic arrays map to `Vec<R>` over the element's Rust type.
        SigType::DynamicArray { child_type } => {
            let inner = rust_param_type(child_type)?;
            Ok(quote! { ::std::vec::Vec<#inner> })
        }
        // Static arrays map to `[R; N]` over the element's Rust type.
        SigType::StaticArray { len, child_type } => {
            let inner = rust_param_type(child_type)?;
            let n = proc_macro2::Literal::usize_unsuffixed(*len as usize);
            Ok(quote! { [#inner; #n] })
        }
        // ufixedNxM maps to the `Ufixed<N, M>` value newtype, which carries the
        // scale in the type and encodes on the wire exactly like `uintN`.
        SigType::UFixed {
            bit_size,
            precision,
        } => {
            let bits = proc_macro2::Literal::u16_unsuffixed(*bit_size);
            let prec = proc_macro2::Literal::u16_unsuffixed(*precision);
            Ok(quote! { ::algonaut_abi::macro_support::Ufixed<#bits, #prec> })
        }
        // Anonymous tuples `(T1,T2,…)` map to Rust tuple types `(R1,R2,…)` over
        // the element mappings. The empty tuple `()` maps to Rust's unit `()`.
        SigType::Tuple { child_types } => {
            let inners = child_types
                .iter()
                .map(rust_param_type)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(quote! { ( #(#inners,)* ) })
        }
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
        SigType::UFixed {
            bit_size,
            precision,
        } => {
            let bits = proc_macro2::Literal::u16_unsuffixed(*bit_size);
            let prec = proc_macro2::Literal::u16_unsuffixed(*precision);
            Ok(quote! { ::algonaut_abi::macro_support::UFixed<#bits, #prec> })
        }
        other => Err(unsupported_type_message(other)),
    }
}

/// Build the expression that encodes `value` (an owned Rust value of
/// [`rust_param_type`] shape) into its [`AbiValue`]. Scalars defer to the
/// `AbiArg<Marker>` impls; arrays map each element recursively and wrap them in
/// `AbiValue::Array`, the representation `algonaut_abi` decodes both static and
/// dynamic arrays from. `depth` keeps the per-level closure binding unique.
pub fn arg_encode_expr(
    ty: &SigType,
    value: &TokenStream,
    depth: usize,
) -> Result<TokenStream, String> {
    match ty {
        // byte[] keeps its canonical Vec<u8> path via the Bytes marker.
        SigType::DynamicArray { child_type } if matches!(**child_type, SigType::Byte) => {
            Ok(quote! {
                ::algonaut_abi::macro_support::AbiArg::<::algonaut_abi::macro_support::Bytes>::encode(#value)
            })
        }
        SigType::DynamicArray { child_type } => array_encode(child_type, value, depth),
        SigType::StaticArray { child_type, .. } => array_encode(child_type, value, depth),
        SigType::Tuple { child_types } => tuple_encode(child_types, value, depth),
        scalar => {
            let marker = abi_marker_type(scalar)?;
            Ok(quote! { ::algonaut_abi::macro_support::AbiArg::<#marker>::encode(#value) })
        }
    }
}

/// Encode a Rust tuple value as its ARC-4 ABI tuple — the same
/// `AbiValue::Array` representation named ARC-56 structs encode to (a struct is
/// a tuple under the hood; see `codegen/structs.rs::abi_encode`). Each element
/// is mapped recursively and accessed by its tuple index. The value is bound
/// once to a fresh local so the argument expression is evaluated exactly once,
/// even for nested tuples; `depth` keeps that binding unique per level.
fn tuple_encode(
    children: &[SigType],
    value: &TokenStream,
    depth: usize,
) -> Result<TokenStream, String> {
    // The empty tuple `()` (Rust's unit) is an empty ABI array; still bind the
    // value so the argument expression is consumed (and not warned as unused).
    if children.is_empty() {
        return Ok(quote! {
            {
                let _ = #value;
                ::algonaut_abi::abi_type::AbiValue::Array(::std::vec![])
            }
        });
    }
    let tup = quote::format_ident!("__tuple{depth}");
    let mut element_encodes = Vec::with_capacity(children.len());
    for (i, child) in children.iter().enumerate() {
        let index = proc_macro2::Literal::usize_unsuffixed(i);
        let field = quote! { #tup.#index };
        element_encodes.push(arg_encode_expr(child, &field, depth + 1)?);
    }
    Ok(quote! {
        {
            let #tup = #value;
            ::algonaut_abi::abi_type::AbiValue::Array(::std::vec![ #(#element_encodes),* ])
        }
    })
}

fn array_encode(child: &SigType, value: &TokenStream, depth: usize) -> Result<TokenStream, String> {
    let elem = quote::format_ident!("__elem{depth}");
    let elem_expr = quote! { #elem };
    let inner = arg_encode_expr(child, &elem_expr, depth + 1)?;
    Ok(quote! {
        ::algonaut_abi::abi_type::AbiValue::Array(
            ::std::iter::IntoIterator::into_iter(#value).map(|#elem| #inner).collect()
        )
    })
}

/// Build the expression that decodes `value` (a `TokenStream` producing an
/// owned [`AbiValue`]) into a `Result<#rust_type, AbiDecodeError>`, the reverse
/// of [`arg_encode_expr`]. Scalars defer to the `AbiDecode<Marker>` impls;
/// dynamic arrays map each element through the same path and `collect` into a
/// `Vec<R>`; static arrays additionally check the element count and build
/// `[R; N]`. `depth` keeps each level's binding unique.
pub fn arg_decode_expr(
    ty: &SigType,
    value: &TokenStream,
    depth: usize,
) -> Result<TokenStream, String> {
    match ty {
        // byte[] keeps its canonical Vec<u8> path via the Bytes marker.
        SigType::DynamicArray { child_type } if matches!(**child_type, SigType::Byte) => {
            Ok(quote! {
                ::algonaut_abi::macro_support::AbiDecode::<::algonaut_abi::macro_support::Bytes>::decode(#value)
            })
        }
        SigType::DynamicArray { child_type } => {
            let elem = quote::format_ident!("__delem{depth}");
            let inner = arg_decode_expr(child_type, &quote! { #elem }, depth + 1)?;
            Ok(quote! {
                ::algonaut_abi::macro_support::decode_array_items(#value)
                    .and_then(|__items| {
                        __items
                            .into_iter()
                            .map(|#elem| #inner)
                            .collect::<::core::result::Result<
                                ::std::vec::Vec<_>,
                                ::algonaut_abi::macro_support::AbiDecodeError,
                            >>()
                    })
            })
        }
        SigType::StaticArray { len, child_type } => {
            let elem = quote::format_ident!("__delem{depth}");
            let inner = arg_decode_expr(child_type, &quote! { #elem }, depth + 1)?;
            let n = proc_macro2::Literal::usize_unsuffixed(*len as usize);
            Ok(quote! {
                ::algonaut_abi::macro_support::decode_array_items(#value)
                    .and_then(|__items| {
                        let __decoded = __items
                            .into_iter()
                            .map(|#elem| #inner)
                            .collect::<::core::result::Result<
                                ::std::vec::Vec<_>,
                                ::algonaut_abi::macro_support::AbiDecodeError,
                            >>()?;
                        <[_; #n]>::try_from(__decoded).map_err(|__v: ::std::vec::Vec<_>| {
                            ::algonaut_abi::macro_support::AbiDecodeError::new(format!(
                                "expected {} elements, got {}", #n, __v.len(),
                            ))
                        })
                    })
            })
        }
        SigType::Tuple { child_types } => tuple_decode(child_types, value, depth),
        scalar => {
            let marker = abi_marker_type(scalar)?;
            Ok(quote! {
                ::algonaut_abi::macro_support::AbiDecode::<#marker>::decode(#value)
            })
        }
    }
}

/// Decode an ARC-4 ABI tuple value into a Rust tuple `(R1, R2, …)` — the reverse
/// of [`tuple_encode`]. Checks the element count, then decodes each element in
/// order; the empty tuple `()` decodes from an empty array. `depth` keeps each
/// level's binding unique.
fn tuple_decode(
    children: &[SigType],
    value: &TokenStream,
    depth: usize,
) -> Result<TokenStream, String> {
    if children.is_empty() {
        return Ok(quote! {
            ::algonaut_abi::macro_support::decode_array_items(#value).and_then(|__items| {
                if !__items.is_empty() {
                    return ::core::result::Result::Err(
                        ::algonaut_abi::macro_support::AbiDecodeError::new(::std::format!(
                            "expected 0 tuple elements, got {}", __items.len(),
                        )),
                    );
                }
                ::core::result::Result::Ok(())
            })
        });
    }
    let n = proc_macro2::Literal::usize_unsuffixed(children.len());
    let it = quote::format_ident!("__dtuple{depth}");
    let mut binds = Vec::with_capacity(children.len());
    let mut idents = Vec::with_capacity(children.len());
    for (i, child) in children.iter().enumerate() {
        let elem_ident = quote::format_ident!("__dtuple{depth}_{i}");
        let idx = proc_macro2::Literal::usize_unsuffixed(i);
        let next = quote! {
            #it.next().ok_or_else(|| ::algonaut_abi::macro_support::AbiDecodeError::new(
                ::std::format!("missing tuple element {}", #idx),
            ))?
        };
        let inner = arg_decode_expr(child, &next, depth + 1)?;
        binds.push(quote! { let #elem_ident = (#inner)?; });
        idents.push(elem_ident);
    }
    Ok(quote! {
        ::algonaut_abi::macro_support::decode_array_items(#value).and_then(|__items| {
            if __items.len() != #n {
                return ::core::result::Result::Err(
                    ::algonaut_abi::macro_support::AbiDecodeError::new(::std::format!(
                        "expected {} tuple elements, got {}", #n, __items.len(),
                    )),
                );
            }
            let mut #it = ::std::iter::IntoIterator::into_iter(__items);
            #(#binds)*
            ::core::result::Result::Ok(( #(#idents,)* ))
        })
    })
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
