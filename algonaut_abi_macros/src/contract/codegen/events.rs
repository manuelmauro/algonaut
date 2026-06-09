//! Generation of the ARC-28 event enum and the `decode_events` decoder.

use super::naming::to_pascal_case;
use crate::contract::parse::AbiContract;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use sha2::{Digest, Sha512_256};

/// The 4-byte ARC-28 event prefix: the first four bytes of the SHA-512/256
/// digest of the event signature (the same construction ARC-4 uses for method
/// selectors).
fn event_selector(signature: &str) -> [u8; 4] {
    let digest = Sha512_256::digest(signature.as_bytes());
    [digest[0], digest[1], digest[2], digest[3]]
}

/// Generate an event enum and a `decode_events` associated function that turns
/// a transaction's logs into typed events, when the contract declares any.
///
/// Each variant carries the decoded ABI tuple of the event's arguments. The
/// 4-byte selector is computed at macro-expansion time and matched against each
/// log's prefix; the remaining bytes are ABI-decoded as the argument tuple.
pub(super) fn generate_events(contract: &AbiContract, struct_ident: &Ident) -> TokenStream {
    if contract.events.is_empty() {
        return TokenStream::new();
    }

    let enum_ident = format_ident!("{}Event", struct_ident);

    let mut variants = Vec::new();
    let mut arms = Vec::new();

    for event in &contract.events {
        let variant_ident = Ident::new(&to_pascal_case(&event.name), Span::call_site());
        let arg_types: Vec<&str> = event.args.iter().map(|a| a.type_.as_str()).collect();
        let signature = format!("{}({})", event.name, arg_types.join(","));
        let [b0, b1, b2, b3] = event_selector(&signature);

        variants.push(quote! {
            #[doc = #signature]
            #variant_ident(::algonaut_abi::abi_type::AbiValue)
        });

        if event.args.is_empty() {
            // No payload: an empty tuple value, no decode needed.
            arms.push(quote! {
                [#b0, #b1, #b2, #b3] => {
                    out.push(#enum_ident::#variant_ident(
                        ::algonaut_abi::abi_type::AbiValue::Array(::std::vec::Vec::new()),
                    ));
                }
            });
        } else {
            let tuple_type = format!("({})", arg_types.join(","));
            arms.push(quote! {
                [#b0, #b1, #b2, #b3] => {
                    let __ty: ::algonaut_abi::abi_type::AbiType = #tuple_type
                        .parse()
                        .expect("contract!: event tuple type validated at macro expansion");
                    if let ::core::result::Result::Ok(__value) = __ty.decode(body) {
                        out.push(#enum_ident::#variant_ident(__value));
                    }
                }
            });
        }
    }

    quote! {
        #[doc = "ARC-28 events that the contract may emit."]
        #[derive(::core::fmt::Debug, ::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq)]
        pub enum #enum_ident {
            #(#variants),*
        }

        impl #struct_ident {
            /// Decode ARC-28 events from a transaction's logs, in log order.
            ///
            /// Logs that don't match a declared event selector (or that fail to
            /// decode) are skipped.
            pub fn decode_events(
                logs: &[::std::vec::Vec<u8>],
            ) -> ::std::vec::Vec<#enum_ident> {
                let mut out = ::std::vec::Vec::new();
                for log in logs {
                    if log.len() < 4 {
                        continue;
                    }
                    let (selector, body) = log.split_at(4);
                    match selector {
                        #(#arms)*
                        _ => {}
                    }
                }
                out
            }
        }
    }
}
