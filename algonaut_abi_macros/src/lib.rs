//! Proc-macros for `algonaut_abi`.
//!
//! - [`macro@contract`] — `contract!("path/to/app.json")` reads an ARC-4 ABI or
//!   ARC-56 app-spec JSON file at compile time and generates a typed client
//!   struct with a method builder for each ABI method.
//!
//! The macro is re-exported from `algonaut_abi`, so call sites write
//! `algonaut_abi::contract!` (or `algonaut::contract!`). The signature/type
//! grammar it shares with the runtime parser lives in [`algonaut_abi_sig`].

mod contract;

use proc_macro::TokenStream;

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
