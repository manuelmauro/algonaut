//! Offline integration tests for algonaut's public API and proc-macros.
//!
//! These need no node: they exercise the `contract!`-generated clients (ARC-4
//! and ARC-56) and the logic-signature APIs. Each area is a module below, all
//! compiled into a single `integration` test binary.

mod arc56_test;
mod array_args;
mod box_references;
mod contract_macro;
mod contract_macro_arc56;
mod logic_signature;
mod reference_args;
mod transaction_args;
mod unsupported;
