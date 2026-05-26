//! Pure ARC-56 / ARC-4 app-spec JSON data model.
//!
//! These structs mirror the on-the-wire app-spec JSON and nothing more: no
//! resolved [`AbiType`], no typed `AppId`, no method selectors. They are the
//! single source of truth for the JSON shape, shared by two crates that must
//! agree on it:
//!
//! - the runtime ([`algonaut_abi`]), whose richer `AbiContract`/`AbiMethod`/…
//!   add a lazily-parsed type cache and typed identifiers and (de)serialize by
//!   delegating here via `#[serde(from / into)]`;
//! - the compile-time `contract!` macro ([`algonaut_abi_macros`]), which reads
//!   these to generate a typed client.
//!
//! # ARC-4 and ARC-56
//!
//! ARC-56 ("Extended App Description") is a strict superset of the ARC-4
//! contract description: same `name`, `methods`, and `networks`, plus named
//! `structs`, declared `state`, ARC-28 `events`, per-method `actions` /
//! `readonly`, per-argument `struct` / `defaultValue`, and deploy metadata
//! (`source` / `byteCode` / `compilerInfo`). Every ARC-56-only field here is
//! optional with a serde default, so an ARC-4 document parses unchanged and
//! re-serializes byte-identically (the new fields are skipped when empty). The
//! method-argument `type` stays the canonical ARC-4 ABI string even in ARC-56;
//! `struct` is only a naming overlay, so [`AbiMethod::get_signature`] is
//! unaffected.
//!
//! Keeping the model in a dependency-light leaf crate (serde-derive only, like
//! the sibling [`algonaut_abi_sig`] grammar crate) is what breaks the cycle:
//! `algonaut_abi` re-exports the macros, so `algonaut_abi_macros` cannot depend
//! on `algonaut_abi` — but both can depend on this.
//!
//! # Module layout
//!
//! The model is grouped by ARC-56 concern across private submodules; every type
//! is re-exported here, so the crate's public API stays flat
//! (`algonaut_abi_model::AbiContract`, etc.):
//!
//! - [`mod@contract`] — the contract/interface/network types and
//!   [`genesis_to_network`];
//! - [`mod@method`] — methods, arguments, returns, default values, recommendations;
//! - [`mod@structs`] — named-struct field definitions;
//! - [`mod@state`] — declared global/local/box state;
//! - [`mod@actions`] — OnComplete actions;
//! - [`mod@events`] — ARC-28 events;
//! - [`mod@program`] — deploy/compile metadata (source, bytecode, template vars).
//!
//! [`AbiType`]: https://docs.rs/algonaut_abi
//! [`algonaut_abi`]: https://docs.rs/algonaut_abi
//! [`algonaut_abi_macros`]: https://docs.rs/algonaut_abi_macros
//! [`algonaut_abi_sig`]: https://docs.rs/algonaut_abi_sig

mod actions;
mod contract;
mod events;
mod method;
mod program;
mod state;
mod structs;

pub use actions::Actions;
pub use contract::{AbiContract, AbiContractNetworkInfo, AbiInterface, genesis_to_network};
pub use events::{Event, EventArg};
pub use method::{
    AbiMethod, AbiMethodArg, AbiReturn, BoxRecommendation, DefaultValue, Recommendations,
};
pub use program::{
    CompilerInfo, CompilerVersion, ProgramPair, ProgramSourceInfo, ProgramSourceInfoPair,
    ScratchVariable, SourceInfo, TemplateVariable,
};
pub use state::{
    ContractState, SchemaCounts, StateKeys, StateMaps, StateSchema, StorageKey, StorageMap,
};
pub use structs::{StructField, StructFieldType};
