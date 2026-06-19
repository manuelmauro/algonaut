//! The crate's structured error type.
//!
//! Per the workspace's structured-leaf-errors ADR, there is no `Msg(String)`
//! catch-all: each failure mode is a typed variant. Network-only variants are
//! gated behind the `fetch` feature so the offline build's error enum carries no
//! HTTP-client types.

use thiserror::Error;

/// Anything that can go wrong in the NFT conventions layer.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum NftError {
    /// An asset URL did not parse as an ARC-19 `template-ipfs://…` template.
    #[error("invalid ARC-19 template URL: {0}")]
    BadTemplateUrl(String),

    /// The template named a CID version / multicodec / hash type this crate does
    /// not support (clients MUST support v0/v1, `raw`/`dag-pb`, `sha2-256`).
    #[error("unsupported CID parameter: {0}")]
    UnsupportedCid(String),

    /// A CID string (base32 / base58btc) failed to decode, or its multihash was
    /// not a 32-byte `sha2-256` digest as ARC-19 requires.
    #[error("malformed CID: {0}")]
    MalformedCid(String),

    /// A computed metadata hash did not match the asset's on-chain `am` field.
    #[error("ARC-3 metadata hash mismatch")]
    MetadataHashMismatch,

    /// A subresource-integrity (`*_integrity`) check failed.
    #[error("integrity mismatch for {field}")]
    IntegrityMismatch {
        /// The metadata field whose integrity failed (e.g. `image`).
        field: String,
    },

    /// An ASA's parameters do not describe a pure NFT (total 1, decimals 0).
    #[error("asset is not a pure NFT (total={total}, decimals={decimals})")]
    NotPureNft {
        /// The asset's total supply.
        total: u64,
        /// The asset's decimals.
        decimals: u32,
    },

    /// Fractional-NFT decimals were outside the valid range `1..=19` (so that
    /// `total = 10^decimals` is non-trivial and fits in a `u64`).
    #[error("fractional NFT decimals must be in 1..=19, got {decimals}")]
    InvalidFractionalDecimals {
        /// The out-of-range decimals value.
        decimals: u32,
    },

    /// An `acfg` note did not contain a valid ARC-69 metadata object.
    #[error("invalid ARC-69 note: {0}")]
    InvalidArc69Note(String),

    /// A base64-encoded metadata field (e.g. ARC-3 `extra_metadata`) failed to decode.
    #[error("invalid base64 in {field}")]
    BadBase64 {
        /// The metadata field that failed to decode.
        field: String,
    },

    /// An ARC-89 Asset Metadata Box value was malformed (too short, oversized, …).
    #[error("invalid ARC-89 metadata box: {0}")]
    InvalidMetadataBox(String),

    /// An ARC-4 method signature could not be parsed / selected.
    #[error("ABI error: {0}")]
    Abi(#[from] algonaut_abi::abi_error::AbiError),

    /// JSON (de)serialisation failed.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// An HTTP request to an indexer or gateway failed.
    #[cfg(feature = "fetch")]
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}
