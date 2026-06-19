//! ASA-based NFTs: NFT-shaped presets and the ARC-71 soulbound lifecycle.
//!
//! This module never introduces a new transaction type or signs anything. It
//! configures the existing `algonaut_transaction` asset builders into the shapes
//! the ARCs require, and returns them for the caller to `build`, sign, and submit
//! through the normal pipeline.
//!
//! - [`create_pure_nft`] / [`create_fractional_nft`] apply the ARC-3 pure
//!   (`total = 1`, `decimals = 0`) and fractional (`total = 10ⁿ`, `decimals = n`,
//!   so the total supply is exactly 1) shapes.
//! - [`Soulbound`] models the ARC-71 non-transferable ("soulbound") lifecycle as
//!   a typestate, so an illegal transition is a compile error.

use algonaut_core::{Address, AssetId};
use algonaut_transaction::transaction::AssetParams;
use algonaut_transaction::{CreateAsset, FreezeAsset, UpdateAsset};
use std::marker::PhantomData;

use crate::zero_address;

/// Build a `CreateAsset` for a pure NFT (total supply 1, indivisible).
///
/// Chain the returned builder to attach metadata (`.url(..)`, `.meta_data_hash(..)`,
/// `.reserve(..)` for ARC-19, etc.) before `.build(..)`.
pub fn create_pure_nft(sender: Address) -> CreateAsset {
    CreateAsset::new(sender, 1, 0, false)
}

/// Build a `CreateAsset` for a fractional NFT: `decimals = n` and
/// `total = 10ⁿ`, so the whole supply represents exactly one token.
pub fn create_fractional_nft(sender: Address, decimals: u32) -> CreateAsset {
    let total = 10u64.pow(decimals);
    CreateAsset::new(sender, total, decimals, false)
}

/// True if these asset parameters describe a pure NFT (total 1, decimals 0).
pub fn is_pure_nft(params: &AssetParams) -> bool {
    params.total == Some(1) && params.decimals == Some(0)
}

/// True if these parameters describe a fractional NFT: `decimals = n > 0` and
/// `total = 10ⁿ`.
pub fn is_fractional_nft(params: &AssetParams) -> bool {
    match (params.total, params.decimals) {
        (Some(total), Some(decimals)) if decimals > 0 => 10u64.checked_pow(decimals) == Some(total),
        _ => false,
    }
}

/// Return an error unless the parameters describe a pure NFT.
pub fn ensure_pure_nft(params: &AssetParams) -> Result<(), crate::NftError> {
    if is_pure_nft(params) {
        Ok(())
    } else {
        Err(crate::NftError::NotPureNft {
            total: params.total.unwrap_or(0),
            decimals: params.decimals.unwrap_or(0),
        })
    }
}

// --- ARC-71 soulbound (non-transferable ASA) lifecycle ---

/// The asset has been issued but not yet held by its recipient.
pub struct Issued;
/// The asset has been claimed by, and frozen in, the holder's account.
pub struct Held;
/// The asset's manager has been zeroed; it is no longer a valid credential.
pub struct Revoked;

/// An ARC-71 non-transferable ("soulbound") ASA, tracked through its lifecycle.
///
/// The type parameter is the current state ([`Issued`], [`Held`], [`Revoked`]);
/// transitions consume `self` and return the next state together with the
/// transaction that effects it. Non-transferability is enforced through the
/// freeze mechanism with a zeroed clawback, exactly as ARC-71 specifies.
pub struct Soulbound<S> {
    /// The issuer — a smart-contract account, the freeze and (recommended)
    /// manager address.
    pub issuer: Address,
    /// The asset id, once known (after creation).
    pub asset_id: Option<AssetId>,
    _state: PhantomData<S>,
}

impl Soulbound<Issued> {
    /// Begin the lifecycle for a given issuer.
    pub fn new(issuer: Address) -> Self {
        Soulbound {
            issuer,
            asset_id: None,
            _state: PhantomData,
        }
    }

    /// Build the issuing `CreateAsset`: a pure NFT with clawback zeroed and the
    /// freeze and manager set to the issuer (the ARC-71 *Issued* state).
    ///
    /// Set the metadata pointer on the returned builder (`.url`/`.reserve`).
    pub fn create(&self) -> CreateAsset {
        create_pure_nft(self.issuer)
            .clawback(zero_address())
            .freeze(self.issuer)
            .manager(self.issuer)
            .reserve(self.issuer)
    }

    /// After the recipient has opted in and received the token, freeze it in
    /// their account, moving to the [`Held`] state. The issuer (the freeze
    /// address) is the sender.
    pub fn freeze_holder(
        self,
        asset_id: AssetId,
        holder: Address,
    ) -> (Soulbound<Held>, FreezeAsset) {
        let txn = FreezeAsset::new(self.issuer, holder, asset_id, true);
        (
            Soulbound {
                issuer: self.issuer,
                asset_id: Some(asset_id),
                _state: PhantomData,
            },
            txn,
        )
    }
}

impl Soulbound<Held> {
    /// Revoke the credential by zeroing the manager address (the ARC-71
    /// *Revoked* state). The token stays in the holder's wallet for provenance
    /// but is no longer a valid credential. Requires the asset id to be known.
    pub fn revoke(self) -> Option<(Soulbound<Revoked>, UpdateAsset)> {
        let asset_id = self.asset_id?;
        let txn = UpdateAsset::new(self.issuer, asset_id).manager(zero_address());
        Some((
            Soulbound {
                issuer: self.issuer,
                asset_id: Some(asset_id),
                _state: PhantomData,
            },
            txn,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(b: u8) -> Address {
        Address([b; 32])
    }

    fn params(total: u64, decimals: u32) -> AssetParams {
        AssetParams {
            asset_name: None,
            decimals: Some(decimals),
            default_frozen: None,
            total: Some(total),
            unit_name: None,
            meta_data_hash: None,
            url: None,
            clawback: None,
            freeze: None,
            manager: None,
            reserve: None,
        }
    }

    #[test]
    fn nft_shape_classifiers() {
        let pure = params(1, 0);
        assert!(is_pure_nft(&pure));
        assert!(!is_fractional_nft(&pure));
        ensure_pure_nft(&pure).unwrap();

        let fractional = params(1000, 3);
        assert!(is_fractional_nft(&fractional));
        assert!(!is_pure_nft(&fractional));
        assert!(matches!(
            ensure_pure_nft(&fractional),
            Err(crate::NftError::NotPureNft {
                total: 1000,
                decimals: 3
            })
        ));
    }

    #[test]
    fn soulbound_lifecycle_typestate() {
        let issuer = addr(1);
        let holder = addr(2);
        let sb = Soulbound::<Issued>::new(issuer);
        // create() yields a builder; the lifecycle advances on freeze_holder.
        let _create = sb.create();

        let sb = Soulbound::<Issued>::new(issuer);
        let (held, _freeze) = sb.freeze_holder(AssetId(42), holder);
        assert_eq!(held.asset_id, Some(AssetId(42)));
        let (revoked, _update) = held.revoke().unwrap();
        assert_eq!(revoked.asset_id, Some(AssetId(42)));
    }
}
