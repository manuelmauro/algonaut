//! Generation of the named ARC-56 structs (and their `abi_encode`), plus the
//! support resolution and ABI tuple-type helpers the rest of codegen relies on.

use super::naming::{is_rust_keyword, is_valid_ident, to_pascal_case, to_snake_case};
use crate::contract::parse::{StructField, StructFieldType};
use crate::contract::type_map::{arg_decode_expr, arg_encode_expr, rust_param_type};
use algonaut_abi_sig::parse_type;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use std::collections::{BTreeSet, HashMap};

/// Resolve which named structs can be fully generated.
///
/// A struct is supported when every field is supported: a leaf field whose
/// type maps to a Rust type, or a field referencing another supported struct.
/// Inline nested structs are supported when their own fields are. Computed to a
/// fixpoint so a struct that references one defined later is still resolved;
/// reference cycles (which would be infinitely sized, and so are invalid)
/// simply stay unsupported.
pub(super) fn resolve_supported_structs(
    structs: &HashMap<String, Vec<StructField>>,
) -> BTreeSet<String> {
    let mut supported = BTreeSet::new();

    loop {
        let mut changed = false;
        for (name, fields) in structs {
            if supported.contains(name) {
                continue;
            }
            // Some tools emit a struct whose *name* is an inline type literal
            // (e.g. `{ foo: uint16; bar: uint16 }`), which is not a valid Rust
            // identifier; such structs (and any method referencing them) are
            // left unsupported rather than panicking `Ident::new`.
            if !is_valid_ident(&to_pascal_case(name)) {
                continue;
            }
            if fields
                .iter()
                .all(|f| field_supported(&f.type_, structs, &supported))
            {
                supported.insert(name.clone());
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    supported
}

/// Whether a struct field's type can be generated, given the structs already
/// known to be supported.
fn field_supported(
    field_type: &StructFieldType,
    structs: &HashMap<String, Vec<StructField>>,
    supported: &BTreeSet<String>,
) -> bool {
    match field_type {
        StructFieldType::Type(s) => {
            if structs.contains_key(s) {
                // A reference to another named struct.
                supported.contains(s)
            } else {
                // A leaf ABI type: supported iff it maps to a Rust type.
                parse_type(s)
                    .ok()
                    .map(|t| rust_param_type(&t).is_ok())
                    .unwrap_or(false)
            }
        }
        // An inline nested struct is supported when all of its own fields are;
        // it is generated as its own Rust sub-struct (see `emit_struct_def`).
        StructFieldType::Nested(sub_fields) => sub_fields
            .iter()
            .all(|f| field_supported(&f.type_, structs, supported)),
    }
}

/// Generate the Rust struct definitions (and their `abi_encode`) for the
/// supported ARC-56 structs, in a deterministic (name-sorted) order.
pub(super) fn generate_structs(
    structs: &HashMap<String, Vec<StructField>>,
    supported: &BTreeSet<String>,
) -> Result<TokenStream, String> {
    let mut names: Vec<&String> = structs.keys().filter(|n| supported.contains(*n)).collect();
    names.sort();

    let mut defs = Vec::new();
    for name in names {
        emit_struct_def(&to_pascal_case(name), &structs[name], structs, &mut defs)?;
    }
    Ok(quote! { #(#defs)* })
}

/// Emit the Rust struct (and its `abi_encode`) for `name_pascal`, pushing it
/// onto `defs`. Inline nested struct fields are emitted recursively as their
/// own sub-structs named `<Parent><Field>`.
fn emit_struct_def(
    name_pascal: &str,
    fields: &[StructField],
    structs: &HashMap<String, Vec<StructField>>,
    defs: &mut Vec<TokenStream>,
) -> Result<(), String> {
    let struct_ident = Ident::new(name_pascal, Span::call_site());
    let mut field_defs = Vec::new();
    let mut field_encodes = Vec::new();
    // Each entry decodes one positional tuple element (bound as `__fields[i]`,
    // moved out) into its field's Rust value, producing `field: value`.
    let mut field_decode_binds = Vec::new();
    let mut field_idents = Vec::new();

    for (i, field) in fields.iter().enumerate() {
        let field_name = to_snake_case(&field.name);
        let field_ident = if is_rust_keyword(&field_name) {
            format_ident!("r#{}", field_name)
        } else {
            Ident::new(&field_name, Span::call_site())
        };

        let idx = proc_macro2::Literal::usize_unsuffixed(i);
        let elem = quote! { __take(&mut __fields, #idx)? };

        let (ty, encode, decode) = match &field.type_ {
            StructFieldType::Type(s) if structs.contains_key(s) => {
                // Reference to another named struct.
                let ty = Ident::new(&to_pascal_case(s), Span::call_site());
                (
                    quote! { #ty },
                    quote! { self.#field_ident.abi_encode() },
                    quote! { #ty::abi_decode(#elem)? },
                )
            }
            StructFieldType::Type(s) => {
                let sig = parse_type(s).map_err(|e| e.reason)?;
                let rust_type = rust_param_type(&sig)?;
                // Route the leaf field through the shared encoder so array
                // fields (now supported by `rust_param_type`) also encode
                // correctly; scalars keep their `AbiArg<Marker>` path.
                let encode = arg_encode_expr(&sig, &quote! { self.#field_ident }, 0)?;
                let decode = arg_decode_expr(&sig, &elem, 0)?;
                (quote! { #rust_type }, encode, quote! { (#decode)? })
            }
            StructFieldType::Nested(sub_fields) => {
                // Synthesize a sub-struct for the anonymous inline fields.
                let sub_name = format!("{name_pascal}{}", to_pascal_case(&field.name));
                emit_struct_def(&sub_name, sub_fields, structs, defs)?;
                let ty = Ident::new(&sub_name, Span::call_site());
                (
                    quote! { #ty },
                    quote! { self.#field_ident.abi_encode() },
                    quote! { #ty::abi_decode(#elem)? },
                )
            }
        };
        field_defs.push(quote! { pub #field_ident: #ty });
        field_encodes.push(encode);
        field_decode_binds.push(quote! { let #field_ident = #decode; });
        field_idents.push(field_ident);
    }

    let field_count = proc_macro2::Literal::usize_unsuffixed(fields.len());
    let doc = format!("Generated ARC-56 struct `{name_pascal}`.");
    defs.push(quote! {
        #[doc = #doc]
        #[derive(Debug, Clone)]
        pub struct #struct_ident {
            #(#field_defs),*
        }

        impl #struct_ident {
            /// Encode this struct as its ARC-4 ABI tuple value.
            pub fn abi_encode(self) -> ::algonaut_abi::abi_type::AbiValue {
                ::algonaut_abi::abi_type::AbiValue::Array(::std::vec![
                    #(#field_encodes),*
                ])
            }

            /// Decode this struct from its ARC-4 ABI tuple value (the reverse of
            /// [`abi_encode`](Self::abi_encode)). Errors on a shape mismatch
            /// (wrong arity or element type) or an out-of-range integer.
            pub fn abi_decode(
                value: ::algonaut_abi::abi_type::AbiValue,
            ) -> ::core::result::Result<Self, ::algonaut_abi::macro_support::AbiDecodeError> {
                fn __take(
                    fields: &mut [::core::option::Option<
                        ::algonaut_abi::abi_type::AbiValue,
                    >],
                    i: usize,
                ) -> ::core::result::Result<
                    ::algonaut_abi::abi_type::AbiValue,
                    ::algonaut_abi::macro_support::AbiDecodeError,
                > {
                    fields
                        .get_mut(i)
                        .and_then(::core::option::Option::take)
                        .ok_or_else(|| ::algonaut_abi::macro_support::AbiDecodeError::new(
                            ::std::format!("missing tuple element {}", i),
                        ))
                }

                let __items = ::algonaut_abi::macro_support::decode_array_items(value)?;
                if __items.len() != #field_count {
                    return ::core::result::Result::Err(
                        ::algonaut_abi::macro_support::AbiDecodeError::new(::std::format!(
                            "expected {} tuple elements, got {}", #field_count, __items.len(),
                        )),
                    );
                }
                let mut __fields: ::std::vec::Vec<::core::option::Option<
                    ::algonaut_abi::abi_type::AbiValue,
                >> = __items.into_iter().map(::core::option::Option::Some).collect();
                #(#field_decode_binds)*
                ::core::result::Result::Ok(Self { #(#field_idents),* })
            }
        }
    });
    Ok(())
}

/// Build the canonical ABI tuple-type string for a named struct (e.g.
/// `"(uint64,address)"`), recursing into struct-typed fields. Returns `None`
/// if a field type is not a decodable ABI type.
pub(super) fn struct_abi_tuple_type(
    name: &str,
    structs: &HashMap<String, Vec<StructField>>,
) -> Option<String> {
    fields_abi_tuple_type(structs.get(name)?, structs)
}

/// The canonical ABI tuple-type string for a list of struct fields, recursing
/// into named-struct references and inline nested structs alike.
fn fields_abi_tuple_type(
    fields: &[StructField],
    structs: &HashMap<String, Vec<StructField>>,
) -> Option<String> {
    let mut parts = Vec::with_capacity(fields.len());
    for field in fields {
        match &field.type_ {
            StructFieldType::Type(s) if structs.contains_key(s) => {
                parts.push(fields_abi_tuple_type(&structs[s], structs)?);
            }
            StructFieldType::Type(s) => {
                parse_type(s).ok()?;
                parts.push(s.clone());
            }
            StructFieldType::Nested(sub_fields) => {
                parts.push(fields_abi_tuple_type(sub_fields, structs)?);
            }
        }
    }
    Some(format!("({})", parts.join(",")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaf(name: &str, ty: &str) -> StructField {
        StructField {
            name: name.to_owned(),
            type_: StructFieldType::Type(ty.to_owned()),
        }
    }

    #[test]
    fn scalar_struct_is_supported() {
        let mut structs = HashMap::new();
        structs.insert(
            "Pair".to_owned(),
            vec![leaf("first", "uint64"), leaf("second", "uint64")],
        );
        let supported = resolve_supported_structs(&structs);
        assert!(supported.contains("Pair"));
    }

    #[test]
    fn struct_with_array_field_is_supported() {
        // `rust_param_type` accepts arrays, so a struct with a `uint64[]` (or a
        // static `uint64[3]`) field is supported and encodes via the shared
        // `arg_encode_expr` path — consistent with the support resolution.
        let mut structs = HashMap::new();
        structs.insert(
            "Bag".to_owned(),
            vec![leaf("items", "uint64[]"), leaf("triple", "uint64[3]")],
        );
        let supported = resolve_supported_structs(&structs);
        assert!(supported.contains("Bag"));
        // It also emits without error (every field encodes).
        assert!(generate_structs(&structs, &supported).is_ok());
    }

    #[test]
    fn struct_referencing_another_struct_resolves() {
        // `Wrapper` references `Pair`, defined in the same map; the fixpoint
        // resolves both regardless of iteration order.
        let mut structs = HashMap::new();
        structs.insert("Wrapper".to_owned(), vec![leaf("p", "Pair")]);
        structs.insert(
            "Pair".to_owned(),
            vec![leaf("first", "uint64"), leaf("second", "uint64")],
        );
        let supported = resolve_supported_structs(&structs);
        assert!(supported.contains("Pair"));
        assert!(supported.contains("Wrapper"));
    }

    #[test]
    fn struct_with_unsupported_field_is_unsupported() {
        // A non-standard-width uint (e.g. `uint24`) has no canonical Rust type.
        // (`ufixed` is now supported via the `Ufixed` newtype, so it no longer
        // serves as the unsupported example here.)
        let mut structs = HashMap::new();
        structs.insert("Bad".to_owned(), vec![leaf("x", "uint24")]);
        let supported = resolve_supported_structs(&structs);
        assert!(!supported.contains("Bad"));

        // An inline nested struct is supported when its sub-fields are (it is
        // generated as its own sub-struct).
        let mut nested = HashMap::new();
        nested.insert(
            "Outer".to_owned(),
            vec![StructField {
                name: "inner".to_owned(),
                type_: StructFieldType::Nested(vec![leaf("a", "uint64")]),
            }],
        );
        assert!(resolve_supported_structs(&nested).contains("Outer"));

        // ...but not when a nested sub-field is itself unsupported.
        let mut nested_bad = HashMap::new();
        nested_bad.insert(
            "OuterBad".to_owned(),
            vec![StructField {
                name: "inner".to_owned(),
                type_: StructFieldType::Nested(vec![leaf("x", "uint24")]),
            }],
        );
        assert!(!resolve_supported_structs(&nested_bad).contains("OuterBad"));
    }

    #[test]
    fn struct_with_ufixed_or_tuple_field_is_supported() {
        // `rust_param_type` now accepts `ufixed` (via the `Ufixed` newtype) and
        // anonymous tuples, so a struct with such fields is supported and emits.
        let mut structs = HashMap::new();
        structs.insert(
            "Mixed".to_owned(),
            vec![leaf("price", "ufixed64x2"), leaf("pair", "(uint64,uint64)")],
        );
        let supported = resolve_supported_structs(&structs);
        assert!(supported.contains("Mixed"));
        assert!(generate_structs(&structs, &supported).is_ok());
    }
}
