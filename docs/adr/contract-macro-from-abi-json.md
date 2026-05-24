---
id: contract-macro-from-abi-json
title: Generate typed contract clients from ARC-4 ABI JSON files
abstract: Add a contract!("path/to/contract.json") procedural macro that reads an ARC-4 ABI JSON file at compile time and generates a type-safe Rust struct with methods for each ABI method. The generated struct holds app_id, sender, and signer; each method returns a builder pre-configured with the method invocation. Integrates with the existing MethodCall::builder pattern and reuses the algonaut_abi_sig grammar for signature validation. Initial scope covers scalar value types (uint*, bool, address, string, byte[]); transaction and reference arguments route through the dynamic Invocation::new path.
status: proposed
date: 2026-05-24
deciders: []
tags: [api, abi, macros, codegen, ergonomics]
---

# Generate typed contract clients from ARC-4 ABI JSON files

## Status

Proposed.

## Context

[`abi-method-signature-macro`](abi-method-signature-macro.md) introduced
`abi_call!` and `abi_method!` for compile-time checked method invocations,
treating the ARC-4 signature literal as a format string. This eliminates
signature typos and argument mismatches for *inline* method calls.

However, when working with a deployed contract whose ABI is defined in a
JSON file (the standard ARC-4 contract specification format), developers
still face repetitive ceremony:

```rust
// Current pattern: manually transcribe each method signature
let call = MethodCall::builder(AppId(123), alice.address(), signer)
    .invoke(abi_call!("add(uint64,uint64)uint64", 2u64, 3u64))
    .build(&params);

let call2 = MethodCall::builder(AppId(123), alice.address(), signer)
    .invoke(abi_call!("subtract(uint64,uint64)uint64", 5u64, 2u64))
    .build(&params);
```

This pattern has several friction points:

1. **Signature drift**: The ABI JSON is the source of truth, but signatures
   are copied into Rust code by hand. A renamed method or changed argument
   type in the JSON does not trigger a compile error in Rust.

2. **Repetitive boilerplate**: Every call repeats `AppId(123)`,
   `alice.address()`, `signer` — the contract-scoped context that should
   be factored out.

3. **No IDE discoverability**: Developers cannot autocomplete method names
   or see parameter types without consulting the JSON file.

4. **Network app IDs are manual**: The ARC-4 `networks` field maps genesis
   hashes to app IDs, but developers must manually extract these.

The Go SDK's `abi.Contract` and Python SDK's typed bindings (via
`algokit-utils`) demonstrate the value of generating typed clients from
ABI JSON. This ADR brings the same ergonomics to Rust via a proc-macro.

## Decision

### D1 — `contract!` macro: generate a typed client from ABI JSON

```rust
algonaut::contract!("contracts/calculator.json");

// Expands to:
pub struct Calculator { /* app_id, sender, signer */ }
impl Calculator {
    pub fn new(app_id, sender, signer) -> Self { ... }
    pub fn add(&self, a: u64, b: u64) -> CalculatorAddBuilder { ... }
    pub fn subtract(&self, a: u64, b: u64) -> CalculatorSubtractBuilder { ... }
}
```

The macro:

1. **Resolves the path** relative to `CARGO_MANIFEST_DIR` (same as
   `include_str!`).
2. **Parses the ARC-4 JSON** into `name`, `description`, `networks`, and
   `methods`.
3. **Validates each method signature** through `algonaut_abi_sig`, ensuring
   the macro and runtime parser cannot disagree.
4. **Generates a struct** named after the contract (PascalCase), holding
   `AppId`, `Address`, and `Arc<dyn Signer>`.
5. **Generates a method** for each ABI method, converting names to
   snake_case for idiomatic Rust.

### D2 — Method builders, not direct MethodCall

Each generated method returns a per-method builder struct:

```rust
pub fn add(&self, a: u64, b: u64) -> CalculatorAddBuilder<'_> { ... }

impl<'a> CalculatorAddBuilder<'a> {
    pub fn build(self, params: &SuggestedParams) -> MethodCall { ... }
}
```

This preserves the ability to set optional parameters (fee, note, boxes)
before finalizing, matching the existing `MethodCall::builder` pattern:

```rust
let call = calculator.add(2u64, 3u64)
    .note(b"calculation".to_vec())
    .build(&params);
```

The builder internally constructs a `MethodInvocation` (the same type
`abi_call!` produces) and feeds it to `MethodCall::builder().invoke()`.

### D3 — Network-specific constructors

If the ABI JSON contains a `networks` field:

```json
{
  "networks": {
    "wGHE2Pwdvd7S12BL5FaOP20EGYesN73ktiC1qzkkit8=": { "appID": 123 }
  }
}
```

The macro generates named constructors for known networks:

```rust
impl Calculator {
    pub fn testnet(sender: Address, signer: Arc<dyn Signer>) -> Self {
        Self::new(AppId(123), sender, signer)
    }
}
```

Genesis hash → network name mapping:
- `wGHE2Pwdvd7S12BL5FaOP20EGYesN73ktiC1qzkkit8=` → `testnet`
- `SGO1GKSzyE7IEPItTxCByw9x8FmnrCDexi9/cOUJOiI=` → `mainnet`
- `mFgazF+2uRS1tMiL9dsj01hJGySEmPN28B/TjjvpVW0=` → `betanet`

Unknown genesis hashes generate a constructor with a sanitized name.

### D4 — Type mapping for initial implementation

The initial implementation maps scalar ABI types to Rust types, matching
the coverage of `abi_call!`:

| ABI Type | Rust Parameter Type |
|----------|---------------------|
| `uint8` | `u8` |
| `uint16` | `u16` |
| `uint32` | `u32` |
| `uint64` | `u64` |
| `uint128` | `u128` |
| `uint256`+ | `num_bigint::BigUint` |
| `bool` | `bool` |
| `address` | `algonaut_core::Address` |
| `string` | `String` |
| `byte[]` | `Vec<u8>` |

Methods containing unsupported argument types (transaction args, reference
args, `ufixed`, compound arrays, tuples) generate a compile error with
guidance to use the dynamic path:

```text
error: method `transfer` has unsupported argument type `pay`;
       use MethodCall::builder().invoke(Invocation::new(...)) for this method
```

This is an honest scope boundary, not a silent fallback.

### D5 — Placement in algonaut_abi_macros

The macro lives in `algonaut_abi_macros`, alongside `abi_call!` and
`abi_method!`. This:

- Keeps ABI-related macros together
- Reuses the existing `algonaut_abi_sig` dependency
- Avoids adding another workspace crate

New dependencies: `serde` and `serde_json` for JSON parsing. A minimal
internal `ContractJson` struct is defined in the macro crate to avoid
depending on `algonaut_abi` (which would create a circular dependency
through re-exports).

Re-export path: `algonaut::contract!` (via `algonaut_abi::contract!`).

### D6 — Method name conversion

ABI method names are converted to snake_case:

- `addLiquidity` → `add_liquidity`
- `getBalance` → `get_balance`
- `add` → `add`

Rust keywords are escaped with a raw identifier prefix (`r#`):

- `type` → `r#type`

### Alternatives considered

- **Runtime loading**: Parse ABI JSON at runtime and return a dynamic
  client. Rejected: loses compile-time type safety, the primary goal.

- **Derive macro on a trait**: Require users to define a trait and derive
  the implementation. Rejected: adds boilerplate; the file-based approach
  is more ergonomic and matches how ABI files are typically used.

- **Separate crate**: Create a new `algonaut_contract_macros` crate.
  Rejected: fragments the macro ecosystem unnecessarily.

- **Generate all types from day one**: Include transaction args, reference
  args, arrays, tuples immediately. Rejected: incremental delivery
  preferred; the scalar types cover the common case and match `abi_call!`
  parity.

## Consequences

- **ABI JSON becomes the single source of truth.** Changing a method
  signature in the JSON triggers recompilation and surfaces mismatches as
  compile errors in Rust code.

- **IDE discoverability.** Developers can autocomplete method names and
  see parameter types directly in their editor.

- **Reduced boilerplate.** Contract context (app ID, sender, signer) is
  factored into the struct; each method call is a single line.

- **New dependencies in algonaut_abi_macros.** `serde` and `serde_json`
  add to the proc-macro build, but these are widely-used, well-optimized
  crates.

- **Partial type coverage by design.** Methods with unsupported argument
  types produce a compile error rather than silently degrading. This is
  consistent with `abi_call!`'s approach.

- **File path resolution at compile time.** The ABI JSON must exist when
  `cargo build` runs; missing files are compile errors. This matches
  `include_str!` behavior.

- **Recompilation on ABI changes.** The macro emits `include_str!` or
  equivalent to establish a dependency on the JSON file, ensuring changes
  trigger rebuilds.
