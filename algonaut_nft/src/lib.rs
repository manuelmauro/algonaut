//! # algonaut_nft
//!
//! The NFT conventions layer for [algonaut](https://crates.io/crates/algonaut).
//!
//! Algorand has no single "NFT" — it has roughly a dozen ARCs that fall into two
//! families with little in common at the protocol level: NFTs that are **Algorand
//! Standard Assets** (ASAs) and NFTs that are **smart contracts**. This crate is
//! the convention layer that sits on top of the raw protocol types in
//! `algonaut_core` / `algonaut_transaction` / `algonaut_abi`, so callers do not
//! re-derive the same metadata models, hashing, CID math, and ABI signatures
//! everyone else does.
//!
//! It is **offline-first**: the default build links no HTTP client. Everything
//! that is pure computation — metadata (de)serialisation, integrity hashing, the
//! ARC-19 URL ↔ reserve-address transform, NFT-shaped ASA presets, the ARC-71
//! lifecycle, ARC-72/ARC-18 ABI signatures, and the ARC-89 box codec — is always
//! available. The network helpers (the ARC-74 client and off-chain metadata
//! fetching) live behind the opt-in `fetch` feature.
//!
//! ## Coverage
//!
//! | Module | ARC(s) | Concern |
//! |--------|--------|---------|
//! | [`metadata::arc3`] | ARC-3 | Off-chain JSON metadata; pure/fractional NFT shape |
//! | [`metadata::arc69`] | ARC-69 | On-chain metadata in the `acfg` note |
//! | [`metadata::traits`] | ARC-16 | `traits` for rarity |
//! | [`metadata::filters`] | ARC-36 | `filters` for non-rarity filtering |
//! | [`metadata::collection`] | ARC-53 | Collection-level declaration |
//! | [`metadata::integrity`] | ARC-3 | `am` hashing + SRI `*_integrity` |
//! | [`url`] | ARC-19 | `template-ipfs://` ↔ reserve-address CID transform |
//! | [`asa`] | ARC-3/69/71 | NFT-shaped ASA presets + soulbound lifecycle |
//! | [`arc72`] | ARC-72/73 | Smart-contract NFT ABI bindings + interface detection |
//! | [`royalty`] | ARC-18 | Royalty-enforcer ABI bindings |
//! | [`indexer`] | ARC-74 | NFT indexer API types (+ `fetch` client) |
//! | [`arc89`] | ARC-89 | **Preview**: Asset Metadata Registry box codec |
//!
//! ARC-49 (a deprecated marketplace rewards program) is intentionally out of
//! scope: it defines no on-chain format to model.

#![forbid(unsafe_code)]

pub mod arc72;
pub mod arc89;
pub mod asa;
pub mod error;
pub mod indexer;
pub mod metadata;
pub mod royalty;
pub mod url;

pub use error::NftError;

/// The zero (all-bytes-zero) Algorand address.
///
/// Several ARCs use it as a sentinel — ARC-71 clawback, ARC-72 "invalid token"
/// ownership, ARC-18 forbidding it as the clawback — so it is re-exported here
/// for convenience.
pub fn zero_address() -> algonaut_core::Address {
    algonaut_core::Address([0u8; 32])
}
