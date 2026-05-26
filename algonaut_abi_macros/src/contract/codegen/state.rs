//! Generation of the `global_<key>` state read accessors.

use super::naming::to_snake_case;
use super::structs::struct_abi_tuple_type;
use crate::contract::parse::{AbiContract, StructField};
use algonaut_abi_sig::parse_type;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::collections::HashMap;

/// Generate read accessors for the contract's declared global-state keys.
///
/// Each generated `global_<key>` method fetches the application from algod and
/// returns the decoded value (as an [`AbiValue`]) for the declared key, or
/// `None` if the key is absent. AVM-typed values map directly; ABI- and
/// struct-typed values are ABI-decoded. Local state, boxes, and maps (which
/// need an account address or a map key) are not generated here.
pub(super) fn generate_state_accessors(
    contract: &AbiContract,
    struct_ident: &Ident,
    structs: &HashMap<String, Vec<StructField>>,
) -> TokenStream {
    let state = match &contract.state {
        Some(state) => state,
        None => return TokenStream::new(),
    };

    let mut keys: Vec<_> = state.keys.global.iter().collect();
    keys.sort_by(|a, b| a.0.cmp(b.0));

    let mut getters = Vec::new();
    for (name, storage_key) in keys {
        let decode = match state_value_decode_expr(&storage_key.value_type, structs) {
            Some(decode) => decode,
            None => continue,
        };
        let fn_ident = format_ident!("global_{}", to_snake_case(name));
        let key_b64 = storage_key.key.as_str();
        let doc = format!("Read the `{name}` global-state value, decoded per its ARC-56 type.");

        getters.push(quote! {
            #[doc = #doc]
            pub async fn #fn_ident(
                &self,
                algod: &::algonaut::Algod,
            ) -> ::core::result::Result<
                ::core::option::Option<::algonaut_abi::abi_type::AbiValue>,
                ::algonaut::Error,
            > {
                let __app = algod.app(self.app_id).await?;
                if let ::core::option::Option::Some(__entries) = &__app.params.global_state {
                    for __kv in __entries {
                        if __kv.key == #key_b64 {
                            let __tv = &__kv.value;
                            return ::core::result::Result::Ok(
                                ::core::option::Option::Some(#decode),
                            );
                        }
                    }
                }
                ::core::result::Result::Ok(::core::option::Option::None)
            }
        });
    }

    if getters.is_empty() {
        return TokenStream::new();
    }

    quote! {
        impl #struct_ident {
            #(#getters)*
        }
    }
}

/// The expression that decodes a global-state [`TealValue`] (bound as `__tv`)
/// into an [`AbiValue`], for a declared ARC-56 value type — or `None` if the
/// type cannot be decoded (so the accessor is skipped).
fn state_value_decode_expr(
    value_type: &str,
    structs: &HashMap<String, Vec<StructField>>,
) -> Option<TokenStream> {
    let abi = quote! { ::algonaut_abi::abi_type::AbiValue };
    match value_type {
        // AVM-native values are already typed in the TealValue.
        "AVMUint64" => Some(quote! { #abi::from(__tv.uint) }),
        "AVMBytes" => Some(quote! { #abi::from(__tv.bytes.clone()) }),
        "AVMString" => Some(quote! {
            #abi::String(::std::string::String::from_utf8_lossy(&__tv.bytes).into_owned())
        }),
        // ABI- and struct-typed values are ABI-decoded from the raw bytes.
        other => {
            let type_str = if structs.contains_key(other) {
                struct_abi_tuple_type(other, structs)?
            } else {
                parse_type(other).ok()?;
                other.to_owned()
            };
            Some(quote! {
                {
                    let __ty: ::algonaut_abi::abi_type::AbiType = #type_str
                        .parse()
                        .expect("contract!: state value type validated at macro expansion");
                    __ty.decode(&__tv.bytes)?
                }
            })
        }
    }
}
