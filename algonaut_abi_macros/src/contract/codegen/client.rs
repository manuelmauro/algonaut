//! Generation of the client struct itself: its definition (with the
//! omitted-methods doc), the `impl` block of methods, and the network-specific
//! constructors.

use super::methods::generate_method;
use super::naming::sanitize_identifier;
use crate::contract::parse::{AbiContract, genesis_to_network};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::collections::BTreeSet;

/// Generate the main contract struct definition. Any methods the macro omits
/// (unsupported argument types) are listed in the doc comment.
pub(super) fn generate_struct(
    struct_ident: &Ident,
    omitted_methods: &[(String, String)],
) -> TokenStream {
    let mut doc = String::from("Generated contract client.");
    if !omitted_methods.is_empty() {
        doc.push_str(
            "\n\n# Omitted methods\n\nThese methods are not generated because the macro \
             does not support their argument types:\n",
        );
        for (name, reason) in omitted_methods {
            doc.push_str(&format!("\n- `{name}`: {reason}"));
        }
    }
    quote! {
        #[doc = #doc]
        pub struct #struct_ident {
            app_id: ::algonaut_core::AppId,
            sender: ::algonaut_core::Address,
            signer: ::std::sync::Arc<dyn ::algonaut_transaction::Signer>,
        }
    }
}

/// Generate the main impl block with `new()` and all method functions.
pub(super) fn generate_impl(
    contract: &AbiContract,
    struct_ident: &Ident,
    supported_structs: &BTreeSet<String>,
) -> TokenStream {
    let mut methods = Vec::new();

    for method in &contract.methods {
        // Omit methods with unsupported argument types (they are listed in the
        // client struct's doc comment). `generate_builders` skips the same set
        // via `method_arg_specs`, so the two stay in lockstep.
        if let Ok(m) = generate_method(method, struct_ident, supported_structs) {
            methods.push(m);
        }
    }

    quote! {
        impl #struct_ident {
            /// Create a new contract client.
            pub fn new(
                app_id: ::algonaut_core::AppId,
                sender: ::algonaut_core::Address,
                signer: ::std::sync::Arc<dyn ::algonaut_transaction::Signer>,
            ) -> Self {
                Self { app_id, sender, signer }
            }

            /// Returns the application ID.
            pub fn app_id(&self) -> ::algonaut_core::AppId {
                self.app_id
            }

            /// Returns the sender address.
            pub fn sender(&self) -> ::algonaut_core::Address {
                self.sender
            }

            #(#methods)*
        }
    }
}

/// Generate network-specific constructors (e.g., `testnet()`, `mainnet()`).
pub(super) fn generate_network_constructors(
    contract: &AbiContract,
    struct_ident: &Ident,
) -> TokenStream {
    let mut constructors = Vec::new();

    for (genesis_hash, network_info) in &contract.networks {
        let network_name = genesis_to_network(genesis_hash)
            .map(|s| s.to_owned())
            .unwrap_or_else(|| sanitize_identifier(genesis_hash));

        let method_ident = Ident::new(&network_name, Span::call_site());
        let app_id = network_info.app_id;

        let doc = format!(
            "Create a client for the {} deployment (app ID {}).",
            network_name, app_id
        );

        constructors.push(quote! {
            #[doc = #doc]
            pub fn #method_ident(
                sender: ::algonaut_core::Address,
                signer: ::std::sync::Arc<dyn ::algonaut_transaction::Signer>,
            ) -> Self {
                Self::new(::algonaut_core::AppId(#app_id), sender, signer)
            }
        });
    }

    if constructors.is_empty() {
        TokenStream::new()
    } else {
        quote! {
            impl #struct_ident {
                #(#constructors)*
            }
        }
    }
}
