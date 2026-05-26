//! Code generation for the `contract!` macro.
//!
//! Generates a typed contract struct with methods for each ABI method, the
//! named ARC-56 structs those methods use, optional network-specific
//! constructors, ARC-28 events, state accessors, and a `deploy` constructor.
//! The work is split by concern across the submodules below; `generate_contract`
//! stitches their output together.

mod client;
mod deploy;
mod events;
mod methods;
mod naming;
mod state;
mod structs;

use crate::contract::parse::AbiContract;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

/// Generate all code for a contract from its parsed JSON.
pub fn generate_contract(contract: &AbiContract) -> Result<TokenStream, String> {
    let struct_ident = Ident::new(&naming::to_pascal_case(&contract.name), Span::call_site());

    // Resolve which ARC-56 named structs can be fully generated. Methods whose
    // arguments the macro can't model (unsupported struct, tuple, array,
    // reference, transaction, …) are omitted rather than failing the build.
    let supported_structs = structs::resolve_supported_structs(&contract.structs);

    // The omitted methods, surfaced in the client's doc comment so a real-world
    // spec yields a usable partial client instead of a `compile_error!`.
    let omitted_methods = methods::unsupported_methods(contract, &supported_structs);

    let structs_def = structs::generate_structs(&contract.structs, &supported_structs)?;
    let struct_def = client::generate_struct(&struct_ident, &omitted_methods);
    let impl_block = client::generate_impl(contract, &struct_ident, &supported_structs);
    let network_constructors = client::generate_network_constructors(contract, &struct_ident);
    let builders = methods::generate_builders(contract, &struct_ident, &supported_structs);
    let events_def = events::generate_events(contract, &struct_ident);
    let state_def = state::generate_state_accessors(contract, &struct_ident, &contract.structs);
    let deploy_def = deploy::generate_deploy(contract, &struct_ident);

    Ok(quote! {
        #structs_def
        #struct_def
        #impl_block
        #network_constructors
        #builders
        #events_def
        #state_def
        #deploy_def
    })
}
