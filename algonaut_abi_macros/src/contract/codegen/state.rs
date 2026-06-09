//! Generation of the contract's state read accessors.
//!
//! Four families of getter are generated from the contract's declared ARC-56
//! `state`:
//!
//! - `global_<key>` — a fixed global-state key (no extra argument).
//! - `local_<key>(account)` — a fixed local-state key, per account.
//! - `box_<key>` — a fixed box, read through algod's box endpoint.
//! - `<class>_<map>(key)` — a dynamic map entry (global/local/box), keyed by a
//!   runtime value encoded per the map's declared `keyType`.
//!
//! All getters return the decoded value as an [`AbiValue`] wrapped in
//! `Option` (absent key/box/account → `None`) and are `async` because they all
//! hit algod.

use super::naming::to_snake_case;
use super::structs::struct_abi_tuple_type;
use crate::contract::parse::{AbiContract, StorageKey, StorageMap, StructField};
use crate::contract::type_map::{arg_encode_expr, rust_param_type};
use algonaut_abi_sig::{ArgClass, classify_arg};
use base64::Engine;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::collections::HashMap;

/// Generate read accessors for every declared state location the macro can
/// decode: fixed global/local/box keys and global/local/box maps.
///
/// Each generated getter fetches the relevant state from algod and returns the
/// decoded value (as an [`AbiValue`]) for the location, or `None` when it is
/// absent. AVM-typed values map directly; ABI- and struct-typed values are
/// ABI-decoded. Locations whose value type the macro can't decode are skipped.
pub(super) fn generate_state_accessors(
    contract: &AbiContract,
    struct_ident: &Ident,
    structs: &HashMap<String, Vec<StructField>>,
) -> TokenStream {
    let state = match &contract.state {
        Some(state) => state,
        None => return TokenStream::new(),
    };

    let mut getters = Vec::new();

    // Fixed global keys.
    for (name, key) in sorted(&state.keys.global) {
        if let Some(g) = global_key_getter(name, key, structs) {
            getters.push(g);
        }
    }
    // Fixed local keys (per account).
    for (name, key) in sorted(&state.keys.local) {
        if let Some(g) = local_key_getter(name, key, structs) {
            getters.push(g);
        }
    }
    // Fixed boxes.
    for (name, key) in sorted(&state.keys.box_) {
        if let Some(g) = box_key_getter(name, key, structs) {
            getters.push(g);
        }
    }
    // Maps, keyed by a runtime value.
    for (name, map) in sorted(&state.maps.global) {
        if let Some(g) = map_getter(name, map, structs, MapClass::Global) {
            getters.push(g);
        }
    }
    for (name, map) in sorted(&state.maps.local) {
        if let Some(g) = map_getter(name, map, structs, MapClass::Local) {
            getters.push(g);
        }
    }
    for (name, map) in sorted(&state.maps.box_) {
        if let Some(g) = map_getter(name, map, structs, MapClass::Box) {
            getters.push(g);
        }
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

/// Sort a name→entry map by name for stable codegen output.
fn sorted<T>(map: &HashMap<String, T>) -> Vec<(&String, &T)> {
    let mut entries: Vec<_> = map.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    entries
}

/// A `global_<key>` getter: look the key up in the application's global state.
fn global_key_getter(
    name: &str,
    key: &StorageKey,
    structs: &HashMap<String, Vec<StructField>>,
) -> Option<TokenStream> {
    let decode = state_value_decode_expr(&key.value_type, structs)?;
    let fn_ident = format_ident!("global_{}", to_snake_case(name));
    let doc = format!("Read the `{name}` global-state value, decoded per its ARC-56 type.");
    Some(key_value_scan_getter(
        &fn_ident,
        &doc,
        key.key.as_str(),
        &decode,
        KvSource::Global,
    ))
}

/// A `local_<key>(account)` getter: look the key up in the account's local
/// state for this application.
fn local_key_getter(
    name: &str,
    key: &StorageKey,
    structs: &HashMap<String, Vec<StructField>>,
) -> Option<TokenStream> {
    let decode = state_value_decode_expr(&key.value_type, structs)?;
    let fn_ident = format_ident!("local_{}", to_snake_case(name));
    let doc =
        format!("Read the `{name}` local-state value for `account`, decoded per its ARC-56 type.");
    Some(key_value_scan_getter(
        &fn_ident,
        &doc,
        key.key.as_str(),
        &decode,
        KvSource::Local,
    ))
}

/// Which state source a key/value scan reads from.
enum KvSource {
    Global,
    /// Local state, per account address (parameter name `account`).
    Local,
}

/// Build a getter that fetches a key→value list and returns the decoded value
/// of the entry whose base64 key matches `key_b64`.
fn key_value_scan_getter(
    fn_ident: &Ident,
    doc: &str,
    key_b64: &str,
    decode: &TokenStream,
    source: KvSource,
) -> TokenStream {
    let abi = quote! { ::algonaut_abi::abi_type::AbiValue };
    let err = quote! { ::algonaut::Error };
    let opt_ret = quote! {
        ::core::result::Result<::core::option::Option<#abi>, #err>
    };
    let scan = quote! {
        for __kv in __entries {
            if __kv.key == #key_b64 {
                let __tv = &__kv.value;
                return ::core::result::Result::Ok(::core::option::Option::Some(#decode));
            }
        }
    };
    match source {
        KvSource::Global => quote! {
            #[doc = #doc]
            pub async fn #fn_ident(
                &self,
                algod: &::algonaut::Algod,
            ) -> #opt_ret {
                let __app = algod.app(self.app_id).await?;
                if let ::core::option::Option::Some(__entries) = &__app.params.global_state {
                    #scan
                }
                ::core::result::Result::Ok(::core::option::Option::None)
            }
        },
        KvSource::Local => quote! {
            #[doc = #doc]
            pub async fn #fn_ident(
                &self,
                algod: &::algonaut::Algod,
                account: &::algonaut_core::Address,
            ) -> #opt_ret {
                let __info = algod.clone().account_app(account, self.app_id).await?;
                if let ::core::option::Option::Some(__state) = &__info.app_local_state {
                    if let ::core::option::Option::Some(__entries) = &__state.key_value {
                        #scan
                    }
                }
                ::core::result::Result::Ok(::core::option::Option::None)
            }
        },
    }
}

/// A `box_<key>` getter: read the fixed box through algod's box endpoint and
/// decode its contents per the declared value type.
fn box_key_getter(
    name: &str,
    key: &StorageKey,
    structs: &HashMap<String, Vec<StructField>>,
) -> Option<TokenStream> {
    let decode = box_value_decode_expr(&key.value_type, structs)?;
    let fn_ident = format_ident!("box_{}", to_snake_case(name));
    let doc = format!("Read the `{name}` box value, decoded per its ARC-56 type.");
    // The box endpoint takes a goal-encoded name; `b64:` carries the raw key
    // bytes (already base64 in the spec) without re-encoding.
    let goal_name = format!("b64:{}", key.key);
    Some(quote! {
        #[doc = #doc]
        pub async fn #fn_ident(
            &self,
            algod: &::algonaut::Algod,
        ) -> ::core::result::Result<
            ::core::option::Option<::algonaut_abi::abi_type::AbiValue>,
            ::algonaut::Error,
        > {
            match algod.app_box(self.app_id, #goal_name).await {
                ::core::result::Result::Ok(__box) => {
                    let __bytes: &[u8] = &__box.value.0;
                    ::core::result::Result::Ok(::core::option::Option::Some(#decode))
                }
                // A missing box surfaces as a 404; treat it as absent rather
                // than an error so callers can probe for existence.
                ::core::result::Result::Err(__e) => {
                    if __e.is_404() {
                        ::core::result::Result::Ok(::core::option::Option::None)
                    } else {
                        ::core::result::Result::Err(__e)
                    }
                }
            }
        }
    })
}

/// Which storage class a map lives in.
#[derive(Clone, Copy)]
enum MapClass {
    Global,
    Local,
    Box,
}

impl MapClass {
    fn prefix(self) -> &'static str {
        match self {
            MapClass::Global => "global",
            MapClass::Local => "local",
            MapClass::Box => "box",
        }
    }
}

/// Build a map getter. The runtime `key` argument is encoded to bytes per the
/// map's `keyType`, the optional prefix is prepended, and the resulting bytes
/// locate the entry:
///
/// - global/local: the bytes are base64-encoded and matched against the state
///   key (which algod reports base64-encoded);
/// - box: the bytes become the goal-encoded box name.
fn map_getter(
    name: &str,
    map: &StorageMap,
    structs: &HashMap<String, Vec<StructField>>,
    class: MapClass,
) -> Option<TokenStream> {
    let (key_param, key_bytes_expr) = map_key_encode(&map.key_type)?;
    let decode = match class {
        MapClass::Box => box_value_decode_expr(&map.value_type, structs)?,
        _ => state_value_decode_expr(&map.value_type, structs)?,
    };

    let fn_ident = format_ident!("{}_{}", class.prefix(), to_snake_case(name));
    let class_doc = class.prefix();
    let doc = format!(
        "Read the `{name}` {class_doc}-map entry for `key`, decoded per its ARC-56 value type."
    );

    // Prefix bytes (base64 in the spec) prepended to the encoded key. The
    // base64 is decoded at macro-expansion time (failing fast on malformed
    // input), so generated code carries only the raw bytes.
    let prefix_prepend = match &map.prefix {
        Some(p) => {
            let bytes = base64::engine::general_purpose::STANDARD.decode(p).ok()?;
            quote! {
                {
                    let mut __full: ::std::vec::Vec<u8> = ::std::vec![#(#bytes),*];
                    __full.extend_from_slice(&__key_bytes);
                    __full
                }
            }
        }
        None => quote! { __key_bytes },
    };

    let abi = quote! { ::algonaut_abi::abi_type::AbiValue };
    let err = quote! { ::algonaut::Error };
    let opt_ret = quote! {
        ::core::result::Result<::core::option::Option<#abi>, #err>
    };

    let body = match class {
        MapClass::Global => quote! {
            let __full_key = #prefix_prepend;
            let __key_b64 = ::algonaut_abi::macro_support::b64_encode(&__full_key);
            let __app = algod.app(self.app_id).await?;
            if let ::core::option::Option::Some(__entries) = &__app.params.global_state {
                for __kv in __entries {
                    if __kv.key == __key_b64 {
                        let __tv = &__kv.value;
                        return ::core::result::Result::Ok(::core::option::Option::Some(#decode));
                    }
                }
            }
            ::core::result::Result::Ok(::core::option::Option::None)
        },
        MapClass::Local => quote! {
            let __full_key = #prefix_prepend;
            let __key_b64 = ::algonaut_abi::macro_support::b64_encode(&__full_key);
            let __info = algod.clone().account_app(account, self.app_id).await?;
            if let ::core::option::Option::Some(__state) = &__info.app_local_state {
                if let ::core::option::Option::Some(__entries) = &__state.key_value {
                    for __kv in __entries {
                        if __kv.key == __key_b64 {
                            let __tv = &__kv.value;
                            return ::core::result::Result::Ok(
                                ::core::option::Option::Some(#decode),
                            );
                        }
                    }
                }
            }
            ::core::result::Result::Ok(::core::option::Option::None)
        },
        MapClass::Box => quote! {
            let __full_key = #prefix_prepend;
            let __goal_name = ::std::format!(
                "b64:{}",
                ::algonaut_abi::macro_support::b64_encode(&__full_key),
            );
            match algod.app_box(self.app_id, &__goal_name).await {
                ::core::result::Result::Ok(__box) => {
                    let __bytes: &[u8] = &__box.value.0;
                    ::core::result::Result::Ok(::core::option::Option::Some(#decode))
                }
                ::core::result::Result::Err(__e) => {
                    if __e.is_404() {
                        ::core::result::Result::Ok(::core::option::Option::None)
                    } else {
                        ::core::result::Result::Err(__e)
                    }
                }
            }
        },
    };

    // Local maps need the account address; the others don't.
    let extra_params = match class {
        MapClass::Local => quote! { account: &::algonaut_core::Address, },
        _ => quote! {},
    };

    Some(quote! {
        #[doc = #doc]
        pub async fn #fn_ident(
            &self,
            algod: &::algonaut::Algod,
            #extra_params
            key: #key_param,
        ) -> #opt_ret {
            let __key_bytes: ::std::vec::Vec<u8> = #key_bytes_expr;
            #body
        }
    })
}

/// The `key` parameter's Rust type and the expression that turns it (bound as
/// `key`) into the raw key bytes, per a map's declared `keyType`. Returns
/// `None` when the key type can't be encoded.
fn map_key_encode(key_type: &str) -> Option<(TokenStream, TokenStream)> {
    match key_type {
        // AVM-native key encodings: raw, no ABI framing.
        "AVMString" => Some((quote! { &str }, quote! { key.as_bytes().to_vec() })),
        "AVMUint64" => Some((quote! { u64 }, quote! { key.to_be_bytes().to_vec() })),
        "AVMBytes" => Some((quote! { &[u8] }, quote! { key.to_vec() })),
        // ABI-typed keys: encode the value with its ABI type.
        other => {
            let class = classify_arg(other).ok()?;
            let sig = match class {
                ArgClass::Value(ty) => ty,
                // reference/transaction types are not storage keys.
                _ => return None,
            };
            let rust_type = rust_param_type(&sig).ok()?;
            let encode = arg_encode_expr(&sig, &quote! { key }, 0).ok()?;
            let type_str = other;
            let bytes = quote! {
                {
                    let __ty: ::algonaut_abi::abi_type::AbiType = #type_str
                        .parse()
                        .expect("contract!: map key type validated at macro expansion");
                    __ty
                        .encode(#encode)
                        .expect("contract!: map key encoding")
                }
            };
            Some((rust_type, bytes))
        }
    }
}

/// The expression that decodes a global/local-state [`TealValue`] (bound as
/// `__tv`) into an [`AbiValue`], for a declared ARC-56 value type — or `None`
/// if the type can't be decoded (so the accessor is skipped).
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
            let type_str = abi_value_type_str(other, structs)?;
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

/// The expression that decodes raw box bytes (bound as `__bytes: &[u8]`) into an
/// [`AbiValue`], per a declared ARC-56 value type — or `None` if undecodable.
///
/// A box is a flat byte buffer with no TEAL type tag, so AVM types map to the
/// raw bytes directly (uint64 from 8 big-endian bytes), and ABI/struct types
/// are ABI-decoded.
fn box_value_decode_expr(
    value_type: &str,
    structs: &HashMap<String, Vec<StructField>>,
) -> Option<TokenStream> {
    let abi = quote! { ::algonaut_abi::abi_type::AbiValue };
    match value_type {
        "AVMUint64" => Some(quote! {
            {
                let mut __buf = [0u8; 8];
                let __n = ::core::cmp::min(8, __bytes.len());
                __buf[8 - __n..].copy_from_slice(&__bytes[__bytes.len() - __n..]);
                #abi::from(u64::from_be_bytes(__buf))
            }
        }),
        "AVMBytes" => Some(quote! { #abi::from(__bytes.to_vec()) }),
        "AVMString" => Some(quote! {
            #abi::String(::std::string::String::from_utf8_lossy(__bytes).into_owned())
        }),
        other => {
            let type_str = abi_value_type_str(other, structs)?;
            Some(quote! {
                {
                    let __ty: ::algonaut_abi::abi_type::AbiType = #type_str
                        .parse()
                        .expect("contract!: box value type validated at macro expansion");
                    __ty.decode(__bytes)?
                }
            })
        }
    }
}

/// Resolve a non-AVM value type to its canonical ABI tuple-type string (a named
/// struct expands to its tuple), validating it parses. `None` if undecodable.
fn abi_value_type_str(
    value_type: &str,
    structs: &HashMap<String, Vec<StructField>>,
) -> Option<String> {
    if structs.contains_key(value_type) {
        struct_abi_tuple_type(value_type, structs)
    } else {
        algonaut_abi_sig::parse_type(value_type).ok()?;
        Some(value_type.to_owned())
    }
}
