//! Generation of the `deploy` associated function: program acquisition (TEAL
//! `source` compiled through algod, or precompiled `byteCode` used directly),
//! template substitution, ABI create-method or bare create, declared create
//! actions, foreign references, and automatic extra-program-page sizing.

use super::methods::{ArgEncode, ArgSpec, method_arg_specs};
use super::naming::{is_rust_keyword, to_snake_case};
use crate::contract::parse::{AbiContract, AbiMethod, TemplateVariable};
use base64::Engine;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use std::collections::BTreeSet;

/// How the approval/clear programs are obtained at deploy time.
enum Programs {
    /// TEAL `source` (text), substituted for template variables and compiled
    /// through algod's `compile` endpoint.
    Source { approval: String, clear: String },
    /// Precompiled `byteCode` (raw program bytes), used as-is — no compile
    /// round-trip, so no node call to obtain the programs.
    ByteCode { approval: Vec<u8>, clear: Vec<u8> },
}

/// Map an AVM OnComplete name to the `ApplicationCallOnComplete` variant ident.
/// Returns `None` for an unknown name (the create stays NoOp).
fn on_complete_variant(action: &str) -> Option<&'static str> {
    match action {
        "NoOp" => Some("NoOp"),
        "OptIn" => Some("OptIn"),
        "CloseOut" => Some("CloseOut"),
        "UpdateApplication" => Some("UpdateApplication"),
        "DeleteApplication" => Some("DeleteApplication"),
        _ => None,
    }
}

/// Generate a `deploy` associated function when the contract carries usable
/// programs (TEAL `source` or precompiled `byteCode`).
///
/// `deploy` obtains the approval and clear programs (compiling `source` through
/// algod, or decoding `byteCode` directly), submits an app-create transaction
/// with the declared state schema — bare, or through the contract's ABI create
/// method with its typed constructor arguments — waits for confirmation, and
/// returns a client bound to the newly created application id. The number of
/// extra program pages is sized automatically from the compiled program length.
///
/// A contract with neither `source` nor `byteCode` (or with malformed base64)
/// gets no `deploy`. A `byteCode`-only contract that also declares template
/// variables gets none either: templates substitute into TEAL text, which a
/// precompiled program no longer carries.
pub(super) fn generate_deploy(
    contract: &AbiContract,
    struct_ident: &Ident,
    supported_structs: &BTreeSet<String>,
) -> TokenStream {
    let engine = base64::engine::general_purpose::STANDARD;

    // Prefer TEAL `source` (it supports template substitution and is what the
    // existing path compiled); fall back to precompiled `byteCode`.
    let programs = if let Some(source) = &contract.source {
        let approval = match engine.decode(&source.approval) {
            Ok(bytes) => bytes,
            Err(_) => return TokenStream::new(),
        };
        let clear = match engine.decode(&source.clear) {
            Ok(bytes) => bytes,
            Err(_) => return TokenStream::new(),
        };
        // TEAL source is text; template variables are substituted into it at
        // deploy time, so carry it as a string rather than raw bytes.
        let approval = match String::from_utf8(approval) {
            Ok(src) => src,
            Err(_) => return TokenStream::new(),
        };
        let clear = match String::from_utf8(clear) {
            Ok(src) => src,
            Err(_) => return TokenStream::new(),
        };
        Programs::Source { approval, clear }
    } else if let Some(byte_code) = &contract.byte_code {
        // A precompiled program cannot host `TMPL_<name>` tokens; refuse to
        // generate a `deploy` that would silently ignore declared templates.
        if !contract.template_variables.is_empty() {
            return TokenStream::new();
        }
        let approval = match engine.decode(&byte_code.approval) {
            Ok(bytes) => bytes,
            Err(_) => return TokenStream::new(),
        };
        let clear = match engine.decode(&byte_code.clear) {
            Ok(bytes) => bytes,
            Err(_) => return TokenStream::new(),
        };
        Programs::ByteCode { approval, clear }
    } else {
        return TokenStream::new();
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

    // One typed `deploy` parameter per declared template variable, substituted
    // for its `TMPL_<name>` token in the source before compiling. TEAL integers
    // are uint64, so integer template variables map to `u64`; a contract using a
    // non-integer template variable gets no generated `deploy` (it cannot be
    // substituted safely yet). Template substitution only applies to the
    // `source` path; the `byteCode` path is gated above to have none.
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

    // How the app is created. Most real contracts create through an ABI method
    // (e.g. `createApplication(...)`) rather than a bare call. We pick the first
    // method whose declared `create` actions include a usable OnComplete, encode
    // its selector and (typed) constructor arguments as the create transaction's
    // app arguments, and set the OnComplete from the declared create action.
    // With no such method we fall back to a bare create.
    let create_method = pick_create_method(contract, supported_structs);

    let create = match &create_method {
        Some((method, specs, on_complete)) => build_method_create(method, specs, on_complete),
        None => build_bare_create(),
    };

    let create_params: Vec<TokenStream> = create_method
        .as_ref()
        .map(|(_, specs, _)| specs.iter().filter_map(|s| s.param.clone()).collect())
        .unwrap_or_default();

    let create_doc = match &create_method {
        Some((method, _, _)) => format!(
            "creates through the ABI method `{}` (its selector and the typed \
             constructor arguments are the create transaction's app arguments)",
            method.name
        ),
        None => "submits a bare app-create".to_owned(),
    };

    let programs_doc = match &programs {
        Programs::Source { .. } => {
            "Compiles the approval and clear programs from the contract's TEAL `source` \
             through algod, then "
        }
        Programs::ByteCode { .. } => {
            "Uses the contract's precompiled `byteCode` directly (no compile round-trip), \
             then "
        }
    };

    let mut full_doc = String::from(programs_doc);
    full_doc.push_str(&create_doc);
    full_doc.push_str(
        ". The number of extra program pages is sized automatically from the program length. \
         Waits for confirmation and returns a client bound to the newly created application id.",
    );
    if !tmpl_docs.is_empty() {
        full_doc.push_str("\n\nTemplate variables: ");
        full_doc.push_str(&tmpl_docs.join(" "));
    }

    // The `tmpl_replaces` only apply to the `source` arms; for `byteCode` the
    // vec is empty (no template variables), so referencing it is harmless.
    let prepare_programs = match &programs {
        Programs::Source { approval, clear } => quote! {
            let __approval_src = ::std::string::String::from(#approval) #(#tmpl_replaces)*;
            let __clear_src = ::std::string::String::from(#clear) #(#tmpl_replaces)*;
            let __approval = algod
                .teal_compile(__approval_src.as_bytes(), ::algonaut::SourceMap::Skip)
                .await?;
            let __clear = algod
                .teal_compile(__clear_src.as_bytes(), ::algonaut::SourceMap::Skip)
                .await?;
        },
        Programs::ByteCode { approval, clear } => {
            let approval_bytes = approval.as_slice();
            let clear_bytes = clear.as_slice();
            quote! {
                let __approval = ::algonaut_core::CompiledTeal(::std::vec![#(#approval_bytes),*]);
                let __clear = ::algonaut_core::CompiledTeal(::std::vec![#(#clear_bytes),*]);
            }
        }
    };

    quote! {
        impl #struct_ident {
            #[doc = "Deploy a new instance of this contract."]
            #[doc = ""]
            #[doc = #full_doc]
            pub async fn deploy(
                algod: &::algonaut::Algod,
                sender: ::algonaut_core::Address,
                signer: ::std::sync::Arc<dyn ::algonaut_transaction::Signer>,
                params: &::algonaut_model::algod::SuggestedParams
                #(, #tmpl_params)*
                #(, #create_params)*
            ) -> ::core::result::Result<Self, ::algonaut::Error> {
                #prepare_programs

                // One free program page; each 2048-byte slab beyond the first
                // needs an extra page. `(total - 1) / 2048` is the count of
                // extra pages (0 when the programs fit in a single page).
                let __program_len = __approval.0.len() + __clear.0.len();
                let __extra_pages: u32 = if __program_len == 0 {
                    0
                } else {
                    (((__program_len - 1) / 2048) as u32)
                };

                let __global_schema = ::algonaut::transaction::transaction::StateSchema {
                    number_ints: #global_ints,
                    number_byteslices: #global_bytes,
                };
                let __local_schema = ::algonaut::transaction::transaction::StateSchema {
                    number_ints: #local_ints,
                    number_byteslices: #local_bytes,
                };

                #create

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

/// Pick the ABI create method to deploy through, if any: the first method whose
/// declared `create` actions include a usable OnComplete *and* whose arguments
/// the macro can model (so its constructor args become typed `deploy`
/// parameters). Returns the method, its argument specs, and the OnComplete
/// variant ident for the chosen create action.
fn pick_create_method<'a>(
    contract: &'a AbiContract,
    supported_structs: &BTreeSet<String>,
) -> Option<(&'a AbiMethod, Vec<ArgSpec>, Ident)> {
    contract.methods.iter().find_map(|method| {
        let actions = method.actions.as_ref()?;
        // Prefer NoOp when offered; otherwise the first recognized create action.
        let action = actions
            .create
            .iter()
            .find(|a| a.as_str() == "NoOp")
            .or_else(|| {
                actions
                    .create
                    .iter()
                    .find(|a| on_complete_variant(a).is_some())
            })?;
        let variant = on_complete_variant(action)?;
        let specs = method_arg_specs(method, supported_structs).ok()?;
        // A create method whose arguments include a *sourced* default (a
        // box/global/local/method read) cannot be deployed: those resolve by
        // reading the app's own state, which does not exist yet at create time.
        // Skip it so `deploy` is still generated for another usable create
        // method (or omitted) rather than emitting code that reads a
        // not-yet-created app.
        if specs
            .iter()
            .any(|s| matches!(s.encode, ArgEncode::SourcedDefault(_)))
        {
            return None;
        }
        Some((method, specs, Ident::new(variant, Span::call_site())))
    })
}

/// Build the create transaction through an ABI method call (`app_id == 0`),
/// honoring its typed/encoded arguments, foreign references, the declared
/// create OnComplete, the state schema, the programs, and the auto-sized extra
/// pages. Produces a `let __outcome = ...;` statement.
fn build_method_create(method: &AbiMethod, specs: &[ArgSpec], on_complete: &Ident) -> TokenStream {
    let signature = method.get_signature();
    let invocation_args = specs.iter().map(|s| match &s.encode {
        ArgEncode::Value(v) => quote! { ::algonaut::atomic::AbiArgValue::AbiValue(#v) },
        ArgEncode::Arg(a) => a.clone(),
        // `pick_create_method` rejects create methods with sourced defaults
        // (they read the not-yet-created app), so this never occurs here; keep
        // the match exhaustive with a clear signal if that ever changes.
        ArgEncode::SourcedDefault(_) => quote! {
            compile_error!(
                "contract!: a create/deploy method argument with a sourced default is not supported"
            )
        },
    });

    quote! {
        let __method =
            ::algonaut_abi::abi_interactions::AbiMethod::from_signature(#signature)
                .expect("contract!: create-method signature validated at macro expansion");
        let __args: ::std::vec::Vec<::algonaut::atomic::AbiArgValue> =
            ::std::vec![ #(#invocation_args),* ];
        let __invocation = ::algonaut::atomic::Invocation::new(__method, __args);

        // `app_id == 0` is the application-creation form of a method call: it
        // carries the approval/clear programs, the state schema, and the extra
        // pages, and encodes the method selector + arguments as app arguments.
        let __call = ::algonaut::atomic::MethodCall::builder(
                ::algonaut_core::AppId(0),
                sender,
                ::std::sync::Arc::clone(&signer),
            )
            .invoke(__invocation)
            .on_complete(
                ::algonaut::transaction::transaction::ApplicationCallOnComplete::#on_complete,
            )
            .approval_program(__approval)
            .clear_program(__clear)
            .global_schema(__global_schema)
            .local_schema(__local_schema)
            .extra_pages(__extra_pages)
            .build(params);

        let __outcome = ::algonaut::atomic::AtomicGroupBuilder::new()
            .add_method_call(__call)
            .build()?
            .sign()
            .await?
            .execute(algod)
            .await?;
    }
}

/// Build a bare (non-ABI) app-create transaction with the programs, schema, and
/// auto-sized extra pages. Produces a `let __outcome = ...;` statement.
fn build_bare_create() -> TokenStream {
    quote! {
        let __txn = ::algonaut::transaction::CreateApplication::new(
                sender,
                __approval,
                __clear,
                __global_schema,
                __local_schema,
            )
            .extra_pages(__extra_pages)
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
    }
}
