//! Generation of the `deploy` associated function: TEAL compilation, template
//! substitution, ABI-method or bare create, and the app-create transaction.

use super::naming::{is_rust_keyword, to_snake_case};
use crate::contract::parse::{AbiContract, TemplateVariable};
use base64::Engine;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use sha2::{Digest, Sha512_256};

/// Generate a `deploy` associated function when the contract carries TEAL
/// `source`.
///
/// `deploy` compiles the approval and clear programs through algod, submits a
/// single app-create transaction with the declared state schema, and returns a
/// client bound to the newly created application id. The TEAL source is
/// base64-decoded at macro-expansion time; a contract without `source` (or with
/// malformed base64) gets no `deploy`.
pub(super) fn generate_deploy(contract: &AbiContract, struct_ident: &Ident) -> TokenStream {
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
