//! `contract!` macro implementation.
//!
//! Reads an ARC-4 ABI JSON file at compile time and generates a type-safe
//! Rust struct with methods for each ABI method.

mod codegen;
mod parse;
mod type_map;

use quote::quote;
use syn::{LitStr, parse_macro_input};

/// Expand the `contract!` macro.
///
/// Input: a string literal path to an ARC-4 ABI JSON file, relative to
/// `CARGO_MANIFEST_DIR`.
///
/// Output: a struct named after the contract with methods for each ABI method.
pub fn expand(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let lit = parse_macro_input!(input as LitStr);
    let path_str = lit.value();

    // Resolve the path relative to CARGO_MANIFEST_DIR
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_owned());
    let full_path = std::path::Path::new(&manifest_dir).join(&path_str);

    // Read the file content
    let json_content = match std::fs::read_to_string(&full_path) {
        Ok(content) => content,
        Err(e) => {
            return syn::Error::new(
                lit.span(),
                format!("failed to read ABI file `{}`: {}", full_path.display(), e),
            )
            .to_compile_error()
            .into();
        }
    };

    // Parse the JSON
    let contract = match parse::parse_contract_json(&json_content) {
        Ok(c) => c,
        Err(e) => {
            return syn::Error::new(lit.span(), e).to_compile_error().into();
        }
    };

    // Generate code
    let generated = match codegen::generate_contract(&contract) {
        Ok(tokens) => tokens,
        Err(e) => {
            return syn::Error::new(lit.span(), e).to_compile_error().into();
        }
    };

    // Emit include_str! to establish a dependency on the JSON file for rebuilds
    let path_str_lit = path_str.as_str();
    let output = quote! {
        // Establish file dependency for incremental compilation
        const _: &str = ::core::include_str!(::core::concat!(
            ::core::env!("CARGO_MANIFEST_DIR"),
            "/",
            #path_str_lit
        ));

        #generated
    };

    output.into()
}
