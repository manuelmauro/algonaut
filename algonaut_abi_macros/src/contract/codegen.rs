//! Code generation for the `contract!` macro.
//!
//! Generates a typed contract struct with methods for each ABI method, the
//! named ARC-56 structs those methods use, and optional network-specific
//! constructors.

use crate::contract::parse::{
    AbiContract, AbiMethod, AbiMethodArg, StructField, StructFieldType, TemplateVariable,
    genesis_to_network,
};
use crate::contract::type_map::{abi_marker_type, rust_param_type};
use algonaut_abi_sig::{ArgClass, parse_signature, parse_type};
use base64::Engine;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use sha2::{Digest, Sha512_256};
use std::collections::{BTreeSet, HashMap};

/// Generate all code for a contract from its parsed JSON.
pub fn generate_contract(contract: &AbiContract) -> Result<TokenStream, String> {
    let struct_name = to_pascal_case(&contract.name);
    let struct_ident = Ident::new(&struct_name, Span::call_site());

    // Resolve which ARC-56 named structs can be fully generated. Methods whose
    // arguments the macro can't model (unsupported struct, tuple, array,
    // reference, transaction, …) are omitted rather than failing the build.
    let supported_structs = resolve_supported_structs(&contract.structs);

    // The omitted methods, surfaced in the client's doc comment so a real-world
    // spec yields a usable partial client instead of a `compile_error!`.
    let omitted_methods = unsupported_methods(contract, &supported_structs);

    // Generate the Rust structs for the supported ARC-56 structs.
    let structs = generate_structs(&contract.structs, &supported_structs)?;

    // Generate the struct definition
    let struct_def = generate_struct(&struct_ident, &omitted_methods);

    // Generate the impl block with new() and method functions
    let impl_block = generate_impl(contract, &struct_ident, &supported_structs);

    // Generate network-specific constructors if networks are present
    let network_constructors = generate_network_constructors(contract, &struct_ident);

    // Generate builder structs for each supported method
    let builders = generate_builders(contract, &struct_ident, &supported_structs);

    // Generate the ARC-28 event enum and decoder, if the contract has events.
    let events = generate_events(contract, &struct_ident);

    // Generate global-state read accessors, if the contract declares state.
    let state = generate_state_accessors(contract, &struct_ident, &contract.structs);

    // Generate a `deploy` constructor, if the contract carries TEAL source.
    let deploy = generate_deploy(contract, &struct_ident);

    Ok(quote! {
        #structs
        #struct_def
        #impl_block
        #network_constructors
        #builders
        #events
        #state
        #deploy
    })
}

/// The methods the macro can't model yet, paired with why — so a real-world
/// spec produces a usable partial client and the gaps stay discoverable.
fn unsupported_methods(
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

/// Generate the main contract struct definition. Any methods the macro omits
/// (unsupported argument types) are listed in the doc comment.
fn generate_struct(struct_ident: &Ident, omitted_methods: &[(String, String)]) -> TokenStream {
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

/// Generate the main impl block with new() and all method functions.
fn generate_impl(
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
/// Shared by [`generate_method`] (which emits the function) and
/// [`generate_builders`] (which only needs to know whether the method is
/// supported), so the two can never disagree about which methods exist.
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
                let marker = abi_marker_type(ty)?;

                specs.push(ArgSpec {
                    param: Some(quote! { #arg_ident: #rust_type }),
                    encode: quote! {
                        ::algonaut_abi::macro_support::AbiArg::<#marker>::encode(#arg_ident)
                    },
                });
            }
            ArgClass::Transaction(tx_type) => {
                return Err(format!("transaction argument `{tx_type}`"));
            }
            ArgClass::Reference(ref_type) => {
                return Err(format!("reference argument `{ref_type}`"));
            }
        }
    }

    Ok(specs)
}

/// Generate a single method function.
fn generate_method(
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
fn generate_builders(
    contract: &AbiContract,
    struct_ident: &Ident,
    supported_structs: &BTreeSet<String>,
) -> TokenStream {
    let mut builders = Vec::new();

    for method in &contract.methods {
        // Skip methods with unsupported arguments (they get a compile_error! in
        // the impl block). Sharing `method_arg_specs` keeps this in lockstep
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

// ===========================================================================
// ARC-28 events
// ===========================================================================

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
fn generate_events(contract: &AbiContract, struct_ident: &Ident) -> TokenStream {
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

// ===========================================================================
// ARC-56 named structs
// ===========================================================================

/// Resolve which named structs can be fully generated.
///
/// A struct is supported when every field is supported: a leaf field whose
/// type maps to a Rust type, or a field referencing another supported struct.
/// Inline nested structs and unsupported leaf types (arrays, tuples, `ufixed`)
/// make a struct unsupported. Computed to a fixpoint so a struct that
/// references one defined later is still resolved; reference cycles (which
/// would be infinitely sized, and so are invalid) simply stay unsupported.
fn resolve_supported_structs(structs: &HashMap<String, Vec<StructField>>) -> BTreeSet<String> {
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
        // Inline nested structs are not generated in this phase.
        StructFieldType::Nested(_) => false,
    }
}

/// Generate the Rust struct definitions (and their `abi_encode`) for the
/// supported ARC-56 structs, in a deterministic (name-sorted) order.
fn generate_structs(
    structs: &HashMap<String, Vec<StructField>>,
    supported: &BTreeSet<String>,
) -> Result<TokenStream, String> {
    let mut names: Vec<&String> = structs.keys().filter(|n| supported.contains(*n)).collect();
    names.sort();

    let mut defs = Vec::new();

    for name in names {
        let fields = &structs[name];
        let struct_ident = Ident::new(&to_pascal_case(name), Span::call_site());

        let mut field_defs = Vec::new();
        let mut field_encodes = Vec::new();

        for field in fields {
            let field_name = to_snake_case(&field.name);
            let field_ident = if is_rust_keyword(&field_name) {
                format_ident!("r#{}", field_name)
            } else {
                Ident::new(&field_name, Span::call_site())
            };

            let (ty, encode) = struct_field_type_and_encode(&field.type_, structs, &field_ident)?;
            field_defs.push(quote! { pub #field_ident: #ty });
            field_encodes.push(encode);
        }

        let doc = format!("Generated ARC-56 struct `{name}`.");

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
            }
        });
    }

    Ok(quote! { #(#defs)* })
}

/// The Rust type and the encode expression for a single struct field, where
/// the encode expression reads `self.<field_ident>`.
fn struct_field_type_and_encode(
    field_type: &StructFieldType,
    structs: &HashMap<String, Vec<StructField>>,
    field_ident: &Ident,
) -> Result<(TokenStream, TokenStream), String> {
    match field_type {
        StructFieldType::Type(s) => {
            if structs.contains_key(s) {
                // Reference to another generated struct.
                let ty = Ident::new(&to_pascal_case(s), Span::call_site());
                Ok((quote! { #ty }, quote! { self.#field_ident.abi_encode() }))
            } else {
                let sig = parse_type(s).map_err(|e| e.reason)?;
                let rust_type = rust_param_type(&sig)?;
                let marker = abi_marker_type(&sig)?;
                Ok((
                    quote! { #rust_type },
                    quote! {
                        ::algonaut_abi::macro_support::AbiArg::<#marker>::encode(self.#field_ident)
                    },
                ))
            }
        }
        StructFieldType::Nested(_) => Err("inline nested struct".to_owned()),
    }
}

// ===========================================================================
// ARC-56 global state
// ===========================================================================

/// Generate read accessors for the contract's declared global-state keys.
///
/// Each generated `global_<key>` method fetches the application from algod and
/// returns the decoded value (as an [`AbiValue`]) for the declared key, or
/// `None` if the key is absent. AVM-typed values map directly; ABI- and
/// struct-typed values are ABI-decoded. Local state, boxes, and maps (which
/// need an account address or a map key) are not generated here.
fn generate_state_accessors(
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

/// Build the canonical ABI tuple-type string for a named struct (e.g.
/// `"(uint64,address)"`), recursing into struct-typed fields. Returns `None`
/// if a field type is not a decodable ABI type (e.g. an inline nested struct).
fn struct_abi_tuple_type(
    name: &str,
    structs: &HashMap<String, Vec<StructField>>,
) -> Option<String> {
    let fields = structs.get(name)?;
    let mut parts = Vec::with_capacity(fields.len());
    for field in fields {
        match &field.type_ {
            StructFieldType::Type(s) => {
                if structs.contains_key(s) {
                    parts.push(struct_abi_tuple_type(s, structs)?);
                } else {
                    parse_type(s).ok()?;
                    parts.push(s.clone());
                }
            }
            StructFieldType::Nested(_) => return None,
        }
    }
    Some(format!("({})", parts.join(",")))
}

// ===========================================================================
// ARC-56 deploy
// ===========================================================================

/// Generate a `deploy` associated function when the contract carries TEAL
/// `source`.
///
/// `deploy` compiles the approval and clear programs through algod, submits a
/// single app-create transaction with the declared state schema, and returns a
/// client bound to the newly created application id. The TEAL source is
/// base64-decoded at macro-expansion time; a contract without `source` (or with
/// malformed base64) gets no `deploy`.
fn generate_deploy(contract: &AbiContract, struct_ident: &Ident) -> TokenStream {
    let source = match &contract.source {
        Some(source) => source,
        None => return TokenStream::new(),
    };

    let engine = base64::engine::general_purpose::STANDARD;
    let approval = match engine.decode(&source.approval) {
        Ok(bytes) => bytes,
        Err(_) => return TokenStream::new(),
    };
    let clear = match engine.decode(&source.clear) {
        Ok(bytes) => bytes,
        Err(_) => return TokenStream::new(),
    };

    let (global_ints, global_bytes, local_ints, local_bytes) = contract
        .state
        .as_ref()
        .map(|state| {
            (
                state.schema.global.ints,
                state.schema.global.bytes,
                state.schema.local.ints,
                state.schema.local.bytes,
            )
        })
        .unwrap_or((0, 0, 0, 0));

    // TEAL source is text; template variables are substituted into it at deploy
    // time, so carry it as a string rather than raw bytes.
    let approval_src = match String::from_utf8(approval) {
        Ok(src) => src,
        Err(_) => return TokenStream::new(),
    };
    let clear_src = match String::from_utf8(clear) {
        Ok(src) => src,
        Err(_) => return TokenStream::new(),
    };

    // One typed `deploy` parameter per declared template variable, substituted
    // for its `TMPL_<name>` token in the source before compiling. TEAL integers
    // are uint64, so integer template variables map to `u64`; a contract using a
    // non-integer template variable gets no generated `deploy` (it cannot be
    // substituted safely yet).
    let mut template_vars: Vec<(&String, &TemplateVariable)> =
        contract.template_variables.iter().collect();
    template_vars.sort_by(|a, b| a.0.cmp(b.0));
    let mut tmpl_params = Vec::new();
    let mut tmpl_replaces = Vec::new();
    let mut tmpl_docs = Vec::new();
    for (name, var) in template_vars {
        if !var.type_.starts_with("uint") {
            return TokenStream::new();
        }
        let param_name = to_snake_case(name);
        let param_ident = if is_rust_keyword(&param_name) {
            format_ident!("r#{}", param_name)
        } else {
            Ident::new(&param_name, Span::call_site())
        };
        let token = format!("TMPL_{name}");
        tmpl_params.push(quote! { #param_ident: u64 });
        // Chained on the source literal, so no `mut` is needed when empty.
        tmpl_replaces.push(quote! { .replace(#token, &#param_ident.to_string()) });
        tmpl_docs.push(format!(
            "`{param_name}` sets the `{token}` template variable."
        ));
    }
    let tmpl_doc = if tmpl_docs.is_empty() {
        String::new()
    } else {
        format!("\n\nTemplate variables: {}", tmpl_docs.join(" "))
    };

    // How the app is created. Most real contracts create through an ABI method
    // (e.g. `createApplication()`, `bareActions.create: []`) rather than a bare
    // call; for a no-arg create method we pass its 4-byte selector as the create
    // transaction's only app argument. With no such method we fall back to a
    // bare NoOp create (what hand-written contracts like the example use).
    let create_method = contract.methods.iter().find(|m| {
        m.args.is_empty()
            && m.actions
                .as_ref()
                .is_some_and(|a| a.create.iter().any(|c| c == "NoOp"))
    });
    let create_args = match create_method {
        Some(method) => {
            let digest = Sha512_256::digest(method.get_signature().as_bytes());
            let (b0, b1, b2, b3) = (digest[0], digest[1], digest[2], digest[3]);
            quote! { .app_arguments(::std::vec![::std::vec![#b0, #b1, #b2, #b3]]) }
        }
        None => quote! {},
    };
    let create_doc = match create_method {
        Some(method) => format!(
            "creates via the ABI method `{}` (its selector is the create \
             transaction's app argument)",
            method.name
        ),
        None => "submits a bare app-create".to_owned(),
    };

    quote! {
        impl #struct_ident {
            #[doc = "Deploy a new instance of this contract."]
            #[doc = ""]
            #[doc = "Compiles the approval and clear programs through algod,"]
            #[doc = #create_doc]
            #[doc = "with the declared state schema, waits for confirmation, and"]
            #[doc = "returns a client bound to the newly created application id."]
            #[doc = #tmpl_doc]
            pub async fn deploy(
                algod: &::algonaut::Algod,
                sender: ::algonaut_core::Address,
                signer: ::std::sync::Arc<dyn ::algonaut_transaction::Signer>,
                params: &::algonaut_model::algod::SuggestedParams
                #(, #tmpl_params)*
            ) -> ::core::result::Result<Self, ::algonaut::Error> {
                let __approval_src = ::std::string::String::from(#approval_src) #(#tmpl_replaces)*;
                let __clear_src = ::std::string::String::from(#clear_src) #(#tmpl_replaces)*;
                let __approval = algod
                    .teal_compile(__approval_src.as_bytes(), ::algonaut::SourceMap::Skip)
                    .await?;
                let __clear = algod
                    .teal_compile(__clear_src.as_bytes(), ::algonaut::SourceMap::Skip)
                    .await?;

                let __txn = ::algonaut::transaction::CreateApplication::new(
                    sender,
                    __approval,
                    __clear,
                    ::algonaut::transaction::transaction::StateSchema {
                        number_ints: #global_ints,
                        number_byteslices: #global_bytes,
                    },
                    ::algonaut::transaction::transaction::StateSchema {
                        number_ints: #local_ints,
                        number_byteslices: #local_bytes,
                    },
                )
                #create_args
                .build(params)?;

                let __outcome = ::algonaut::atomic::AtomicGroupBuilder::new()
                    .add_transaction(::algonaut::atomic::TransactionWithSigner::new(
                        __txn,
                        ::std::sync::Arc::clone(&signer),
                    ))
                    .build()?
                    .sign()
                    .await?
                    .execute(algod)
                    .await?;

                let __app_id = __outcome.created_app_id.ok_or_else(|| {
                    ::algonaut::Error::Msg(
                        "deploy: confirmed transaction did not create an application".to_owned(),
                    )
                })?;

                ::core::result::Result::Ok(Self::new(__app_id, sender, signer))
            }
        }
    }
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

/// Whether `s` is a valid, non-keyword Rust identifier — so it can name a
/// generated type without panicking `Ident::new`. Real-world specs sometimes
/// carry struct names that are inline type literals, not identifiers.
fn is_valid_ident(s: &str) -> bool {
    syn::parse_str::<syn::Ident>(s).is_ok()
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

    fn leaf(name: &str, ty: &str) -> StructField {
        StructField {
            name: name.to_owned(),
            type_: StructFieldType::Type(ty.to_owned()),
        }
    }

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
        // A `ufixed` field has no canonical Rust type.
        let mut structs = HashMap::new();
        structs.insert("Bad".to_owned(), vec![leaf("x", "ufixed64x2")]);
        let supported = resolve_supported_structs(&structs);
        assert!(!supported.contains("Bad"));

        // Inline nested structs are not generated in this phase.
        let mut nested = HashMap::new();
        nested.insert(
            "Outer".to_owned(),
            vec![StructField {
                name: "inner".to_owned(),
                type_: StructFieldType::Nested(vec![leaf("a", "uint64")]),
            }],
        );
        assert!(!resolve_supported_structs(&nested).contains("Outer"));
    }
}
