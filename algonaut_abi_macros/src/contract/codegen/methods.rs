//! Generation of the per-method functions and their call builders, including
//! argument encoding, literal defaults, lifecycle setters, and the scan for
//! methods the macro can't model.

use super::naming::{is_rust_keyword, to_pascal_case, to_snake_case};
use crate::contract::parse::{AbiContract, AbiMethod, AbiMethodArg};
use crate::contract::type_map::{arg_encode_expr, rust_param_type};
use algonaut_abi_sig::{ArgClass, parse_signature, parse_type};
use base64::Engine;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use std::collections::BTreeSet;

/// The parameter declaration and the argument-encoding expression for one
/// method argument.
struct ArgSpec {
    /// `name: Type` for the generated method's parameter list, or `None` when
    /// the argument is supplied automatically (e.g. a literal default) and so
    /// takes no parameter.
    param: Option<TokenStream>,
    /// An expression producing the argument's [`AbiValue`].
    encode: TokenStream,
}

/// The methods the macro can't model yet, paired with why — so a real-world
/// spec produces a usable partial client and the gaps stay discoverable.
pub(super) fn unsupported_methods(
    contract: &AbiContract,
    supported_structs: &BTreeSet<String>,
) -> Vec<(String, String)> {
    contract
        .methods
        .iter()
        .filter_map(|method| {
            method_arg_specs(method, supported_structs)
                .err()
                .map(|reason| (method.name.clone(), reason))
        })
        .collect()
}

/// If an argument has a `literal` default value, build the expression that
/// decodes that constant into an [`AbiValue`] at run time, so the caller can
/// omit it. The base64 payload is decoded at macro-expansion time (failing
/// fast on malformed input); the ABI decode happens at run time via the type
/// the contract declares. Returns `None` for arguments with no literal default
/// — including the other `defaultValue` sources (box/global/local/method),
/// which need a runtime read and remain required parameters for now.
fn literal_default_encode(model_arg: &AbiMethodArg) -> Option<TokenStream> {
    let default_value = model_arg.default_value.as_ref()?;
    if default_value.source != "literal" {
        return None;
    }

    // The default's own `type` wins; otherwise the value is encoded as the
    // argument's ABI type. AVM types (e.g. "AVMUint64") do not parse as ABI
    // types, so such defaults fall through and the argument stays required.
    let type_str = default_value
        .type_
        .as_deref()
        .unwrap_or(model_arg.type_.as_str());
    parse_type(type_str).ok()?;

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&default_value.data)
        .ok()?;

    Some(quote! {
        {
            let __ty: ::algonaut_abi::abi_type::AbiType = #type_str
                .parse()
                .expect("contract!: ABI type validated at macro expansion");
            __ty
                .decode(&[#(#bytes),*])
                .expect("contract!: literal default value")
        }
    })
}

/// Build the per-argument specs for a method, or an error naming the first
/// unsupported argument.
///
/// Shared by [`generate_method`] (which emits the function), [`generate_builders`]
/// and [`unsupported_methods`], so they can never disagree about which methods
/// exist.
fn method_arg_specs(
    method: &AbiMethod,
    supported_structs: &BTreeSet<String>,
) -> Result<Vec<ArgSpec>, String> {
    let signature = method.get_signature();
    let parsed = parse_signature(&signature).map_err(|e| e.reason)?;

    let mut specs = Vec::with_capacity(parsed.args.len());

    for (i, arg_class) in parsed.args.iter().enumerate() {
        let model_arg = method.args.get(i);

        let arg_name = model_arg
            .and_then(|a| a.name.as_ref())
            .map(|n| to_snake_case(n))
            .unwrap_or_else(|| format!("arg{i}"));

        // Escape Rust keywords
        let arg_ident = if is_rust_keyword(&arg_name) {
            format_ident!("r#{}", arg_name)
        } else {
            Ident::new(&arg_name, Span::call_site())
        };

        // A literal default value lets the caller omit the argument entirely.
        if let Some(encode) = model_arg.and_then(literal_default_encode) {
            specs.push(ArgSpec {
                param: None,
                encode,
            });
            continue;
        }

        // ARC-56 named struct argument: use the generated Rust struct as the
        // parameter type and encode it as an ABI tuple.
        if let Some(struct_name) = model_arg.and_then(|a| a.struct_.as_ref()) {
            if !supported_structs.contains(struct_name) {
                return Err(format!(
                    "struct argument `{struct_name}` has unsupported field types"
                ));
            }
            let ty = Ident::new(&to_pascal_case(struct_name), Span::call_site());
            specs.push(ArgSpec {
                param: Some(quote! { #arg_ident: #ty }),
                encode: quote! { #arg_ident.abi_encode() },
            });
            continue;
        }

        match arg_class {
            ArgClass::Value(ty) => {
                let rust_type = rust_param_type(ty)?;
                let value = quote! { #arg_ident };
                let encode = arg_encode_expr(ty, &value, 0)?;
                specs.push(ArgSpec {
                    param: Some(quote! { #arg_ident: #rust_type }),
                    encode,
                });
            }
            ArgClass::Transaction(tx_type) => {
                return Err(format!("transaction argument `{tx_type}`"));
            }
            ArgClass::Reference(ref_type) => {
                // ARC-4 reference argument: the value flows as a plain
                // `AbiValue` through `MethodInvocation`. At group-build time the
                // method-call encoder reclassifies it by the method signature,
                // appends it to the transaction's foreign accounts/assets/apps
                // array, and encodes the `uint8` index as the ABI argument — so
                // here the macro only needs a typed parameter and the right
                // `AbiValue`.
                let (rust_type, encode) = match ref_type.as_str() {
                    "account" => (
                        quote! { ::algonaut_core::Address },
                        quote! { ::algonaut_abi::abi_type::AbiValue::Address(#arg_ident) },
                    ),
                    "asset" => (
                        quote! { ::algonaut_core::AssetId },
                        quote! {
                            ::algonaut_abi::abi_type::AbiValue::Int(
                                ::num_bigint::BigUint::from(#arg_ident.0)
                            )
                        },
                    ),
                    "application" => (
                        quote! { ::algonaut_core::AppId },
                        quote! {
                            ::algonaut_abi::abi_type::AbiValue::Int(
                                ::num_bigint::BigUint::from(#arg_ident.0)
                            )
                        },
                    ),
                    other => return Err(format!("reference argument `{other}`")),
                };

                specs.push(ArgSpec {
                    param: Some(quote! { #arg_ident: #rust_type }),
                    encode,
                });
            }
        }
    }

    Ok(specs)
}

/// Generate a single method function.
pub(super) fn generate_method(
    method: &AbiMethod,
    struct_ident: &Ident,
    supported_structs: &BTreeSet<String>,
) -> Result<TokenStream, String> {
    let specs = method_arg_specs(method, supported_structs)?;
    let signature = method.get_signature();

    let params = specs.iter().filter_map(|s| s.param.as_ref());
    let encode_calls = specs.iter().map(|s| &s.encode);

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
                on_complete:
                    ::algonaut::transaction::transaction::ApplicationCallOnComplete::NoOp,
            }
        }
    })
}

/// Generate builder structs for each supported method.
pub(super) fn generate_builders(
    contract: &AbiContract,
    struct_ident: &Ident,
    supported_structs: &BTreeSet<String>,
) -> TokenStream {
    let mut builders = Vec::new();

    for method in &contract.methods {
        // Skip methods with unsupported arguments (they are omitted from the
        // impl block too). Sharing `method_arg_specs` keeps this in lockstep
        // with `generate_method`.
        if method_arg_specs(method, supported_structs).is_err() {
            continue;
        }

        let method_name = &method.name;
        let builder_ident = format_ident!("{}{}", struct_ident, to_pascal_case(method_name));

        let doc = format!("Builder for the `{}` method.", method_name);
        let action_setters = lifecycle_setters(method);

        builders.push(quote! {
            #[doc = #doc]
            pub struct #builder_ident<'a> {
                contract: &'a #struct_ident,
                invocation: ::algonaut_abi::MethodInvocation,
                on_complete:
                    ::algonaut::transaction::transaction::ApplicationCallOnComplete,
            }

            impl<'a> #builder_ident<'a> {
                #action_setters

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
                    .on_complete(self.on_complete)
                    .build(params)
                }

                /// Dry-run this single-method call through algod's simulate
                /// endpoint and return the outcome, including the decoded ABI
                /// return value (`method_results[0].return_value`).
                ///
                /// This is the read path for read-only (ARC-22) methods: it
                /// signs with placeholder signatures and submits nothing, so it
                /// neither charges fees nor changes state.
                pub async fn simulate(
                    self,
                    algod: &::algonaut::Algod,
                    params: &::algonaut_model::algod::SuggestedParams,
                ) -> ::core::result::Result<
                    ::algonaut::atomic::SimulateOutcome,
                    ::algonaut::Error,
                > {
                    let call = self.build(params);
                    ::algonaut::atomic::AtomicGroupBuilder::new()
                        .add_method_call(call)
                        .build()?
                        .simulate(algod)
                        .await
                }
            }
        });
    }

    quote! { #(#builders)* }
}

/// Generate the on-completion setters a method's builder should expose, gated
/// by the method's declared ARC-56 `call` actions.
///
/// `NoOp` is the default and needs no setter. Each other declared action gets
/// an ergonomic setter (`opt_in`, `close_out`, `update`, `delete`) so a caller
/// invokes, say, an opt-in method with `client.method(..).opt_in().build(..)`.
/// Methods with no declared actions (e.g. plain ARC-4 methods) get none and
/// stay NoOp-only.
fn lifecycle_setters(method: &AbiMethod) -> TokenStream {
    let actions = match &method.actions {
        Some(actions) => actions,
        None => return TokenStream::new(),
    };

    let mut setters = Vec::new();
    for action in &actions.call {
        let (setter, variant) = match action.as_str() {
            "OptIn" => ("opt_in", "OptIn"),
            "CloseOut" => ("close_out", "CloseOut"),
            "UpdateApplication" => ("update", "UpdateApplication"),
            "DeleteApplication" => ("delete", "DeleteApplication"),
            // NoOp is the default; anything unknown is ignored.
            _ => continue,
        };
        let setter_ident = Ident::new(setter, Span::call_site());
        let variant_ident = Ident::new(variant, Span::call_site());
        let doc = format!("Invoke this method with the `{action}` on-completion action.");
        setters.push(quote! {
            #[doc = #doc]
            pub fn #setter_ident(mut self) -> Self {
                self.on_complete =
                    ::algonaut::transaction::transaction::ApplicationCallOnComplete::#variant_ident;
                self
            }
        });
    }

    quote! { #(#setters)* }
}
