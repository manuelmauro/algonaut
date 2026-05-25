//! Compile-time checked ARC-4 ABI method-call macros.
//!
//! These proc-macros model an ABI method call on the `format!`/`println!`
//! family: the method signature is a *format string* whose argument **types**
//! are the specifiers, and the trailing macro arguments are the values that
//! fill them. Validation that `format!` does at compile time — the spec is
//! well-formed, the argument count matches, each argument has the right type —
//! happens here too, with spans on the offending token.
//!
//! - [`macro@abi_call`] — `abi_call!("add(uint64,uint64)uint64", 2u64, 3u64)`
//!   validates the signature, checks arity, and type-checks each argument via
//!   a per-type `AbiArg<T>` bound, expanding to an `algonaut_abi::MethodInvocation`.
//! - [`macro@abi_method`] — `abi_method!("add(uint64,uint64)uint64")` is the
//!   signature-only base: validate the literal and expand to an `AbiMethod`.
//! - [`macro@contract`] — `contract!("path/to/contract.json")` reads an ARC-4
//!   ABI JSON file and generates a typed struct with methods for each ABI
//!   method.
//!
//! All macros are re-exported from `algonaut_abi`, so call sites write
//! `algonaut_abi::abi_call!` (or `algonaut::abi::abi_call!`). The grammar is
//! the one in [`algonaut_abi_sig`], the same crate `from_signature` parses
//! with at run time — so the macros and the runtime parser cannot disagree.

mod contract;

use algonaut_abi_sig::{ArgClass, SigType};
use proc_macro::TokenStream;
use proc_macro2::{Literal, Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Expr, LitStr, Token, parse_macro_input};

/// `abi_call!("signature", arg0, arg1, …)`: a compile-time checked ABI method
/// invocation.
///
/// The signature literal is validated against the ARC-4 grammar, the number of
/// trailing arguments is checked against it, and each argument is type-checked
/// through the `AbiArg<T>` trait. The macro expands to an
/// `algonaut_abi::MethodInvocation` — the method plus its encoded value
/// arguments — which `MethodCall`'s builder accepts via `.invoke(...)`.
///
/// # Supported argument types
///
/// The first cut checks the value types with a canonical Rust representation:
/// `uintN` (native `u8`/`u16`/`u32`/`u64`/`u128` for the matching or wider
/// width, `BigUint` for any width), `byte`, `bool`, `address`, `string`, and
/// `byte[]`. Transaction args (`pay`, `txn`, …), reference args (`account`,
/// `asset`, `application`), `ufixed`, and compound array/tuple types are not
/// yet bound by the macro; build those calls with the dynamic path
/// (`MethodCall::builder(..).invoke(Invocation::new(AbiMethod::from_signature(..)?, [..]))`).
#[proc_macro]
pub fn abi_call(input: TokenStream) -> TokenStream {
    let AbiCall { sig, args } = parse_macro_input!(input as AbiCall);
    let sig_str = sig.value();

    let parsed = match algonaut_abi_sig::parse_signature(&sig_str) {
        Ok(p) => p,
        Err(e) => return signature_error(sig.span(), &e.reason, &e.input),
    };

    // Arity: every signature slot consumes exactly one trailing argument,
    // matching the positional `format!` model.
    if parsed.args.len() != args.len() {
        return syn::Error::new(
            Span::call_site(),
            format!(
                "this ABI signature takes {} argument{} but {} {} supplied",
                parsed.args.len(),
                plural(parsed.args.len()),
                args.len(),
                were_was(args.len()),
            ),
        )
        .to_compile_error()
        .into();
    }

    // Per-argument type check: emit `AbiArg::<Marker>::encode(arg)` with the
    // argument's own span, so a type mismatch reads as an unsatisfied
    // `AbiArg<…>` bound pointed at that argument.
    let mut encoded = Vec::with_capacity(args.len());
    for (slot, expr) in parsed.args.iter().zip(args.iter()) {
        let marker = match marker_for(slot) {
            Ok(marker) => marker,
            Err(reason) => {
                return syn::Error::new(expr.span(), reason)
                    .to_compile_error()
                    .into();
            }
        };
        encoded.push(quote_spanned! {expr.span()=>
            ::algonaut_abi::macro_support::AbiArg::<#marker>::encode(#expr)
        });
    }

    quote! {{
        let method = ::algonaut_abi::abi_interactions::AbiMethod::from_signature(#sig)
            .expect("abi_call!: signature validated at macro expansion");
        let args: ::std::vec::Vec<::algonaut_abi::abi_type::AbiValue> =
            ::std::vec![ #(#encoded),* ];
        ::algonaut_abi::MethodInvocation::new(method, args)
    }}
    .into()
}

/// `abi_method!("signature")`: validate the signature literal at compile time
/// and expand to the `AbiMethod` it describes. The signature-only counterpart
/// to [`macro@abi_call`], for when the method is needed as a value (passed
/// around, or with arguments supplied dynamically).
#[proc_macro]
pub fn abi_method(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as LitStr);
    let sig_str = lit.value();

    if let Err(e) = algonaut_abi_sig::parse_signature(&sig_str) {
        return signature_error(lit.span(), &e.reason, &e.input);
    }

    quote! {
        ::algonaut_abi::abi_interactions::AbiMethod::from_signature(#lit)
            .expect("abi_method!: signature validated at macro expansion")
    }
    .into()
}

/// Parsed `abi_call!` input: a signature string literal followed by zero or
/// more comma-separated argument expressions (a trailing comma is allowed).
struct AbiCall {
    sig: LitStr,
    args: Vec<Expr>,
}

impl Parse for AbiCall {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let sig: LitStr = input.parse()?;
        let mut args = Vec::new();
        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break; // trailing comma
            }
            args.push(input.parse()?);
        }
        Ok(AbiCall { sig, args })
    }
}

/// The marker type for an argument slot, used as the `T` in `AbiArg<T>`. Errors
/// (with a guiding message) for argument kinds the macro does not yet bind.
fn marker_for(slot: &ArgClass) -> Result<TokenStream2, String> {
    match slot {
        ArgClass::Value(ty) => value_marker(ty),
        ArgClass::Transaction(name) => Err(unsupported(&format!("transaction argument `{name}`"))),
        ArgClass::Reference(name) => Err(unsupported(&format!("reference argument `{name}`"))),
    }
}

fn value_marker(ty: &SigType) -> Result<TokenStream2, String> {
    match ty {
        SigType::UInt { bit_size } => {
            let bits = Literal::u16_unsuffixed(*bit_size);
            Ok(quote! { ::algonaut_abi::macro_support::Uint<#bits> })
        }
        SigType::Byte => Ok(quote! { ::algonaut_abi::macro_support::Byte }),
        SigType::Bool => Ok(quote! { ::algonaut_abi::macro_support::Bool }),
        SigType::Address => Ok(quote! { ::algonaut_abi::macro_support::Address }),
        SigType::String => Ok(quote! { ::algonaut_abi::macro_support::AbiString }),
        // `byte[]` is the one compound type with a canonical Rust rep (`Vec<u8>`).
        SigType::DynamicArray { child_type } if matches!(**child_type, SigType::Byte) => {
            Ok(quote! { ::algonaut_abi::macro_support::Bytes })
        }
        other => Err(unsupported(&format!("`{}` argument", describe(other)))),
    }
}

/// A short, human-readable name for a type the macro can't yet bind, used in
/// the diagnostic.
fn describe(ty: &SigType) -> &'static str {
    match ty {
        SigType::UFixed { .. } => "ufixed",
        SigType::StaticArray { .. } => "static array",
        SigType::DynamicArray { .. } => "dynamic array",
        SigType::Tuple { .. } => "tuple",
        // The scalar cases are handled by `value_marker`; reaching here would
        // be a bug, but name them rather than panic.
        SigType::UInt { .. } => "uint",
        SigType::Byte => "byte",
        SigType::Bool => "bool",
        SigType::Address => "address",
        SigType::String => "string",
    }
}

fn unsupported(what: &str) -> String {
    format!(
        "abi_call! does not yet bind {what}; build this call with the dynamic path: \
         MethodCall::builder(app_id, sender, signer)\
         .invoke(Invocation::new(AbiMethod::from_signature(<signature>)?, [<args>]))"
    )
}

fn signature_error(span: Span, reason: &str, input: &str) -> TokenStream {
    syn::Error::new(
        span,
        format!("invalid ABI signature: {reason} (in `{input}`)"),
    )
    .to_compile_error()
    .into()
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

fn were_was(n: usize) -> &'static str {
    if n == 1 { "was" } else { "were" }
}

/// `contract!("path/to/contract.json")`: generate a typed contract client from
/// an ARC-4 ABI JSON file.
///
/// The macro reads the ABI JSON at compile time and generates a struct named
/// after the contract (PascalCase) with methods for each ABI method. The path
/// is resolved relative to `CARGO_MANIFEST_DIR`, matching `include_str!` behavior.
///
/// # Example
///
/// ```ignore
/// algonaut::contract!("contracts/calculator.json");
///
/// // Use the generated struct
/// let client = Calculator::new(AppId(5), alice.address(), signer);
/// let call = client.add(2u64, 3u64).build(&params);
/// ```
///
/// # Generated Code
///
/// For a contract with methods `add(uint64,uint64)uint64` and `subtract(uint64,uint64)uint64`:
///
/// ```ignore
/// pub struct Calculator {
///     app_id: AppId,
///     sender: Address,
///     signer: Arc<dyn Signer>,
/// }
///
/// impl Calculator {
///     pub fn new(app_id, sender, signer) -> Self { ... }
///     pub fn add(&self, a: u64, b: u64) -> CalculatorAddBuilder { ... }
///     pub fn subtract(&self, a: u64, b: u64) -> CalculatorSubtractBuilder { ... }
/// }
/// ```
///
/// If the ABI JSON contains a `networks` field, named constructors are generated
/// for known networks (testnet, mainnet, betanet).
///
/// # Supported Types
///
/// The initial implementation supports scalar types with canonical Rust
/// representations: `uint8`-`uint512`, `bool`, `address`, `string`, `byte[]`.
/// Methods with transaction args, reference args, or compound types (arrays,
/// tuples) generate a compile error with guidance to use the dynamic path.
///
/// # Feature Requirements
///
/// The generated code uses `MethodCall` from the `algonaut::atomic` module,
/// which requires the `algod` feature (included in default features). Ensure
/// your `algonaut` dependency includes this feature.
#[proc_macro]
pub fn contract(input: TokenStream) -> TokenStream {
    contract::expand(input)
}
