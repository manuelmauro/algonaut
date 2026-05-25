//! Code generation for the `contract!` macro.
//!
//! Generates a typed contract struct with methods for each ABI method,
//! plus optional network-specific constructors.

use crate::contract::parse::{AbiContract, AbiMethod, genesis_to_network};
use crate::contract::type_map::{abi_marker_type, rust_param_type};
use algonaut_abi_sig::{ArgClass, parse_signature};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};

/// Generate all code for a contract from its parsed JSON.
pub fn generate_contract(contract: &AbiContract) -> Result<TokenStream, String> {
    let struct_name = to_pascal_case(&contract.name);
    let struct_ident = Ident::new(&struct_name, Span::call_site());

    // Generate the struct definition
    let struct_def = generate_struct(&struct_ident);

    // Generate the impl block with new() and method functions
    let impl_block = generate_impl(contract, &struct_ident)?;

    // Generate network-specific constructors if networks are present
    let network_constructors = generate_network_constructors(contract, &struct_ident);

    // Generate builder structs for each method
    let builders = generate_builders(contract, &struct_ident)?;

    Ok(quote! {
        #struct_def
        #impl_block
        #network_constructors
        #builders
    })
}

/// Generate the main contract struct definition.
fn generate_struct(struct_ident: &Ident) -> TokenStream {
    quote! {
        #[doc = "Generated contract client."]
        pub struct #struct_ident {
            app_id: ::algonaut_core::AppId,
            sender: ::algonaut_core::Address,
            signer: ::std::sync::Arc<dyn ::algonaut_transaction::Signer>,
        }
    }
}

/// Generate the main impl block with new() and all method functions.
fn generate_impl(contract: &AbiContract, struct_ident: &Ident) -> Result<TokenStream, String> {
    let mut methods = Vec::new();

    for method in &contract.methods {
        match generate_method(method, struct_ident) {
            Ok(m) => methods.push(m),
            Err(e) => {
                // Generate a compile_error! for unsupported methods
                let method_name = &method.name;
                let error_msg =
                    format!("method `{method_name}` has unsupported argument type: {e}");
                let method_ident = Ident::new(&to_snake_case(method_name), Span::call_site());
                methods.push(quote! {
                    #[doc = "This method has unsupported argument types."]
                    pub fn #method_ident(&self) {
                        ::core::compile_error!(#error_msg);
                    }
                });
            }
        }
    }

    Ok(quote! {
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
    })
}

/// Generate a single method function.
fn generate_method(method: &AbiMethod, struct_ident: &Ident) -> Result<TokenStream, String> {
    let signature = method.get_signature();

    // Parse and validate the signature using algonaut_abi_sig
    let parsed = parse_signature(&signature).map_err(|e| e.reason)?;

    // Generate parameter list
    let mut params = Vec::new();
    let mut param_idents = Vec::new();
    let mut encode_calls = Vec::new();

    for (i, arg_class) in parsed.args.iter().enumerate() {
        let arg_name = method
            .args
            .get(i)
            .and_then(|a| a.name.as_ref())
            .map(|n| to_snake_case(n))
            .unwrap_or_else(|| format!("arg{i}"));

        // Escape Rust keywords
        let arg_ident = if is_rust_keyword(&arg_name) {
            format_ident!("r#{}", arg_name)
        } else {
            Ident::new(&arg_name, Span::call_site())
        };

        match arg_class {
            ArgClass::Value(ty) => {
                let rust_type = rust_param_type(ty)?;
                let marker = abi_marker_type(ty)?;

                params.push(quote! { #arg_ident: #rust_type });
                encode_calls.push(quote! {
                    ::algonaut_abi::macro_support::AbiArg::<#marker>::encode(#arg_ident)
                });
            }
            ArgClass::Transaction(tx_type) => {
                return Err(format!("transaction argument `{tx_type}`"));
            }
            ArgClass::Reference(ref_type) => {
                return Err(format!("reference argument `{ref_type}`"));
            }
        }

        param_idents.push(arg_ident);
    }

    let method_name_str = &method.name;
    let method_ident = Ident::new(&to_snake_case(method_name_str), Span::call_site());
    let builder_ident = format_ident!("{}{}", struct_ident, to_pascal_case(method_name_str));

    let doc = method
        .desc
        .as_ref()
        .map(|d| quote! { #[doc = #d] })
        .unwrap_or_default();

    Ok(quote! {
        #doc
        pub fn #method_ident(&self, #(#params),*) -> #builder_ident<'_> {
            let method = ::algonaut_abi::abi_interactions::AbiMethod::from_signature(#signature)
                .expect("contract!: signature validated at macro expansion");
            let args: ::std::vec::Vec<::algonaut_abi::abi_type::AbiValue> =
                ::std::vec![ #(#encode_calls),* ];
            let invocation = ::algonaut_abi::MethodInvocation::new(method, args);
            #builder_ident {
                contract: self,
                invocation,
            }
        }
    })
}

/// Generate network-specific constructors (e.g., testnet(), mainnet()).
fn generate_network_constructors(contract: &AbiContract, struct_ident: &Ident) -> TokenStream {
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

/// Generate builder structs for each supported method.
fn generate_builders(contract: &AbiContract, struct_ident: &Ident) -> Result<TokenStream, String> {
    let mut builders = Vec::new();

    for method in &contract.methods {
        let signature = method.get_signature();

        // Skip methods with unsupported types (they get compile_error! in the method)
        let parsed = match parse_signature(&signature) {
            Ok(p) => p,
            Err(_) => continue,
        };

        let has_unsupported = parsed.args.iter().any(|arg| {
            matches!(arg, ArgClass::Transaction(_) | ArgClass::Reference(_))
                || match arg {
                    ArgClass::Value(ty) => rust_param_type(ty).is_err(),
                    _ => false,
                }
        });

        if has_unsupported {
            continue;
        }

        let method_name = &method.name;
        let builder_ident = format_ident!("{}{}", struct_ident, to_pascal_case(method_name));

        let doc = format!("Builder for the `{}` method.", method_name);

        builders.push(quote! {
            #[doc = #doc]
            pub struct #builder_ident<'a> {
                contract: &'a #struct_ident,
                invocation: ::algonaut_abi::MethodInvocation,
            }

            impl<'a> #builder_ident<'a> {
                /// Build the method call with the given suggested parameters.
                pub fn build(
                    self,
                    params: &::algonaut_model::algod::SuggestedParams,
                ) -> ::algonaut::atomic::MethodCall {
                    ::algonaut::atomic::MethodCall::builder(
                        self.contract.app_id,
                        self.contract.sender,
                        ::std::sync::Arc::clone(&self.contract.signer),
                    )
                    .invoke(self.invocation)
                    .build(params)
                }
            }
        });
    }

    Ok(quote! { #(#builders)* })
}

/// Convert a string to PascalCase.
fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// Convert a string to snake_case.
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();

    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else if c == '-' || c == ' ' {
            result.push('_');
        } else {
            result.push(c);
        }
    }

    result
}

/// Check if a string is a Rust keyword.
fn is_rust_keyword(s: &str) -> bool {
    matches!(
        s,
        "as" | "async"
            | "await"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "try"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
    )
}

/// Sanitize a string to be a valid Rust identifier.
fn sanitize_identifier(s: &str) -> String {
    let sanitized: String = s
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    // Ensure it doesn't start with a digit
    if sanitized.starts_with(|c: char| c.is_ascii_digit()) {
        format!("network_{sanitized}")
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("calculator"), "Calculator");
        assert_eq!(to_pascal_case("add_liquidity"), "AddLiquidity");
        assert_eq!(to_pascal_case("myContract"), "MyContract");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("addLiquidity"), "add_liquidity");
        assert_eq!(to_snake_case("getBalance"), "get_balance");
        assert_eq!(to_snake_case("add"), "add");
    }

    #[test]
    fn test_is_rust_keyword() {
        assert!(is_rust_keyword("type"));
        assert!(is_rust_keyword("fn"));
        assert!(!is_rust_keyword("add"));
    }
}
