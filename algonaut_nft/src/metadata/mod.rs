//! The off-chain and on-chain NFT metadata conventions.
//!
//! Several ARCs combine into the artifact a wallet or marketplace actually reads:
//!
//! - [`arc3`] — the off-chain JSON metadata file (ARC-3),
//! - [`arc69`] — the on-chain metadata stored in the asset-config `note` (ARC-69),
//! - [`traits`] — the `traits` object for rarity (ARC-16),
//! - [`filters`] — the `filters` object for non-rarity filtering (ARC-36),
//! - [`collection`] — the collection-level declaration (ARC-53),
//! - [`integrity`] — the ARC-3 `am` hash and the SRI `*_integrity` strings.
//!
//! The ARC-3 and ARC-69 models are kept **distinct** rather than merged: they are
//! stored differently (off-chain file vs `acfg` note), hash differently, and
//! evolve independently. Merging them would force lossy round-trips.

pub mod arc3;
pub mod arc69;
pub mod collection;
pub mod filters;
pub mod integrity;
pub mod traits;
