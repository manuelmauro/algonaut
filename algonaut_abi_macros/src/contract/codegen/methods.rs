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
    /// How this argument contributes to the invocation's argument vector.
    encode: ArgEncode,
}

/// How an argument contributes to the invocation's argument vector.
enum ArgEncode {
    /// An expression producing an `AbiValue` (scalars, structs, literal
    /// defaults).
    Value(TokenStream),
    /// An expression producing an `AbiArgValue` directly (transaction args,
    /// which occupy their own slot in the atomic group).
    Arg(TokenStream),
    /// An `async` expression producing an `AbiValue` by reading from algod at
    /// call time — a sourced (non-literal) `defaultValue`. Its presence makes
    /// the generated method `async` and gives it an `&Algod` parameter.
    SourcedDefault(TokenStream),
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
/// — the other `defaultValue` sources (box/global/local/method) need a runtime
/// read and are handled by [`sourced_default_resolve`] instead.
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

/// The ABI type string to decode a sourced default's storage value with: the
/// default's own `type` wins, else the argument's type. Returns `None` when
/// neither is a decodable ABI type (e.g. AVM types), so the default falls
/// through and the argument stays required.
fn sourced_default_type(model_arg: &AbiMethodArg, default_type: Option<&str>) -> Option<String> {
    let type_str = default_type.unwrap_or(model_arg.type_.as_str());
    parse_type(type_str).ok()?;
    Some(type_str.to_owned())
}

/// If an argument has a *sourced* (non-literal) default value, build the
/// `async` expression that reads it at call time and yields an `AbiValue`, so
/// the caller can omit it. Reads run against the `algod` parameter the method
/// gains; `global`/`box` reads are keyed by the default's base64 `data`,
/// `local` reads use the client's configured `self.sender`, and `method`
/// simulates the named read-only method and takes its return value.
///
/// Returns `None` for a `literal` default (handled by
/// [`literal_default_encode`]) or for a sourced default the macro can't model
/// (an undecodable value type, or an unknown source) — leaving the argument a
/// required parameter.
fn sourced_default_resolve(model_arg: &AbiMethodArg) -> Option<TokenStream> {
    let default_value = model_arg.default_value.as_ref()?;
    let data = default_value.data.as_str();

    let abi = quote! { ::algonaut_abi::abi_type::AbiValue };

    // A `global`/`box`/`local` storage read decodes raw bytes with this type.
    let decode_storage = |bytes_var: TokenStream| -> Option<TokenStream> {
        let type_str = sourced_default_type(model_arg, default_value.type_.as_deref())?;
        Some(quote! {
            {
                let __ty: ::algonaut_abi::abi_type::AbiType = #type_str
                    .parse()
                    .expect("contract!: default value type validated at macro expansion");
                __ty.decode(#bytes_var)?
            }
        })
    };

    match default_value.source.as_str() {
        "literal" => None,
        "global" => {
            let decode = decode_storage(quote! { &__tv.bytes })?;
            Some(quote! {
                {
                    let __app = algod.app(self.app_id).await?;
                    let mut __value: ::core::option::Option<#abi> = ::core::option::Option::None;
                    if let ::core::option::Option::Some(__entries) = &__app.params.global_state {
                        for __kv in __entries {
                            if __kv.key == #data {
                                let __tv = &__kv.value;
                                __value = ::core::option::Option::Some(#decode);
                                break;
                            }
                        }
                    }
                    __value.ok_or_else(|| ::algonaut::Error::Internal(
                        ::std::format!("contract!: default global key {} not found", #data)
                    ))?
                }
            })
        }
        "local" => {
            let decode = decode_storage(quote! { &__tv.bytes })?;
            Some(quote! {
                {
                    let __info = algod.clone().account_app(&self.sender, self.app_id).await?;
                    let mut __value: ::core::option::Option<#abi> = ::core::option::Option::None;
                    if let ::core::option::Option::Some(__state) = &__info.app_local_state {
                        if let ::core::option::Option::Some(__entries) = &__state.key_value {
                            for __kv in __entries {
                                if __kv.key == #data {
                                    let __tv = &__kv.value;
                                    __value = ::core::option::Option::Some(#decode);
                                    break;
                                }
                            }
                        }
                    }
                    __value.ok_or_else(|| ::algonaut::Error::Internal(
                        ::std::format!("contract!: default local key {} not found", #data)
                    ))?
                }
            })
        }
        "box" => {
            let decode = decode_storage(quote! { &__box.value.0 })?;
            let goal_name = format!("b64:{data}");
            Some(quote! {
                {
                    let __box = algod.app_box(self.app_id, #goal_name).await?;
                    #decode
                }
            })
        }
        "method" => {
            // `data` is the signature of a read-only method; simulate it with
            // no arguments and take its return value. The signature is
            // validated at macro-expansion time.
            parse_signature(data).ok()?;
            Some(quote! {
                {
                    let __method = ::algonaut_abi::abi_interactions::AbiMethod::from_signature(#data)
                        .expect("contract!: default method signature validated at macro expansion");
                    let __invocation = ::algonaut::atomic::Invocation::new(
                        __method,
                        ::std::vec::Vec::<::algonaut::atomic::AbiArgValue>::new(),
                    );
                    let __params = algod.suggested_params().await?;
                    let __call = ::algonaut::atomic::MethodCall::builder(
                        self.app_id,
                        self.sender,
                        ::std::sync::Arc::clone(&self.signer),
                    )
                    .invoke(__invocation)
                    .build(&__params);
                    let __outcome = ::algonaut::atomic::AtomicGroupBuilder::new()
                        .add_method_call(__call)
                        .build()?
                        .simulate(algod)
                        .await?;
                    match &__outcome.method_results[0].return_value {
                        ::core::result::Result::Ok(
                            ::algonaut::atomic::AbiMethodReturnValue::Some(__v)
                        ) => __v.clone(),
                        _ => return ::core::result::Result::Err(::algonaut::Error::Internal(
                            ::std::format!("contract!: default method {} returned no value", #data)
                        )),
                    }
                }
            })
        }
        _ => None,
    }
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
                encode: ArgEncode::Value(encode),
            });
            continue;
        }

        // A sourced (non-literal) default is resolved by a runtime read; the
        // method gains an `&Algod` parameter and becomes `async`. The argument
        // takes no parameter, exactly like a literal default.
        if let Some(resolve) = model_arg.and_then(sourced_default_resolve) {
            specs.push(ArgSpec {
                param: None,
                encode: ArgEncode::SourcedDefault(resolve),
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
                encode: ArgEncode::Value(quote! { #arg_ident.abi_encode() }),
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
                    encode: ArgEncode::Value(encode),
                });
            }
            ArgClass::Transaction(_tx_type) => {
                // A caller-supplied transaction the builder places immediately
                // before this method call in the atomic group. Any transaction
                // type is accepted as a `TransactionWithSigner`; the AVM checks
                // the precise type.
                specs.push(ArgSpec {
                    param: Some(quote! { #arg_ident: ::algonaut::atomic::TransactionWithSigner }),
                    encode: ArgEncode::Arg(
                        quote! { ::algonaut::atomic::AbiArgValue::from(#arg_ident) },
                    ),
                });
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
                    encode: ArgEncode::Value(encode),
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

    // A sourced default must be resolved with an `await`, so build the argument
    // vector through per-slot `let` bindings rather than an inline `vec!`. This
    // also makes the whole method `async` and gives it an `&Algod` parameter.
    let has_sourced_default = specs
        .iter()
        .any(|s| matches!(s.encode, ArgEncode::SourcedDefault(_)));

    let mut arg_bindings = Vec::new();
    let mut arg_idents = Vec::new();
    for (i, spec) in specs.iter().enumerate() {
        let ident = format_ident!("__arg{i}");
        let bind = match &spec.encode {
            ArgEncode::Value(v) => {
                quote! { let #ident = ::algonaut::atomic::AbiArgValue::AbiValue(#v); }
            }
            ArgEncode::Arg(a) => quote! { let #ident = #a; },
            ArgEncode::SourcedDefault(r) => {
                quote! { let #ident = ::algonaut::atomic::AbiArgValue::AbiValue(#r); }
            }
        };
        arg_bindings.push(bind);
        arg_idents.push(ident);
    }

    let method_name_str = &method.name;
    let method_ident = Ident::new(&to_snake_case(method_name_str), Span::call_site());
    let builder_ident = format_ident!("{}{}", struct_ident, to_pascal_case(method_name_str));

    let doc = method
        .desc
        .as_ref()
        .map(|d| quote! { #[doc = #d] })
        .unwrap_or_default();

    let build_body = quote! {
        let method = ::algonaut_abi::abi_interactions::AbiMethod::from_signature(#signature)
            .expect("contract!: signature validated at macro expansion");
        #(#arg_bindings)*
        let args: ::std::vec::Vec<::algonaut::atomic::AbiArgValue> =
            ::std::vec![ #(#arg_idents),* ];
        let invocation = ::algonaut::atomic::Invocation::new(method, args);
        #builder_ident {
            contract: self,
            invocation,
            on_complete:
                ::algonaut::transaction::transaction::ApplicationCallOnComplete::NoOp,
            boxes: ::std::vec::Vec::new(),
        }
    };

    if has_sourced_default {
        // Reads at call time: the method is `async` and returns a `Result`
        // because a default read can fail. An extra doc line flags the shape so
        // the divergence from the sync builder methods is discoverable.
        let async_doc = " \n\nThis method has one or more sourced (non-literal) default arguments, \
             so it reads them from algod at call time: it takes an `&Algod` and is `async`, \
             returning the builder once the defaults resolve. `local` defaults read the client's \
             sender.";
        Ok(quote! {
            #doc
            #[doc = #async_doc]
            pub async fn #method_ident(
                &self,
                algod: &::algonaut::Algod,
                #(#params),*
            ) -> ::core::result::Result<#builder_ident<'_>, ::algonaut::Error> {
                ::core::result::Result::Ok({ #build_body })
            }
        })
    } else {
        Ok(quote! {
            #doc
            pub fn #method_ident(&self, #(#params),*) -> #builder_ident<'_> {
                #build_body
            }
        })
    }
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
                invocation: ::algonaut::atomic::Invocation,
                on_complete:
                    ::algonaut::transaction::transaction::ApplicationCallOnComplete,
                boxes: ::std::vec::Vec<
                    ::algonaut::transaction::transaction::BoxReference,
                >,
            }

            impl<'a> #builder_ident<'a> {
                #action_setters

                /// Attach a reference to a box of THIS application by its raw key bytes.
                /// Required for methods that read or write box storage; the AVM rejects a
                /// box access whose reference is absent from the transaction.
                pub fn box_ref(
                    mut self,
                    name: impl ::core::convert::Into<::std::vec::Vec<u8>>,
                ) -> Self {
                    self.boxes.push(::algonaut::transaction::transaction::BoxReference {
                        app_id: ::core::option::Option::None,
                        name: name.into(),
                    });
                    self
                }

                /// Attach a reference to a box owned by another application.
                pub fn box_ref_of(
                    mut self,
                    app_id: ::algonaut_core::AppId,
                    name: impl ::core::convert::Into<::std::vec::Vec<u8>>,
                ) -> Self {
                    self.boxes.push(::algonaut::transaction::transaction::BoxReference {
                        app_id: ::core::option::Option::Some(app_id),
                        name: name.into(),
                    });
                    self
                }

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
                    .boxes(self.boxes)
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
