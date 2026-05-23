pub mod v2;

/// Raw OpenAPI operation functions and transport configuration.
///
/// Re-exported from [`algonaut_indexer::apis`] so callers depending only on
/// `algonaut` can reach the low-level operations without adding an explicit
/// `algonaut_indexer` dependency.
pub use algonaut_indexer::apis as api;
