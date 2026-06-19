//! ARC-18 — royalty enforcement.
//!
//! ARC-18 guarantees a royalty on every primary *and* secondary sale by routing
//! all transfers through a Royalty Enforcer application whose address is the
//! asset's clawback. Like [`arc72`](crate::arc72), this is an *application*
//! standard, so the module provides the policy/offer data types and the ARC-4
//! method signatures (with selector resolution); calls are composed and run
//! through `algonaut::atomic`.

use algonaut_abi::abi_interactions::AbiMethod;
use algonaut_core::Address;

/// A royalty policy: who receives the royalty and how much.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoyaltyPolicy {
    /// Royalty share in basis points (1% = 100; range 0–10000).
    pub royalty_basis: u64,
    /// The account that receives royalties.
    pub royalty_recipient: Address,
}

/// An owner's transfer offer, stored in local state keyed by the asset id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssetOffer {
    /// The address authorised to initiate the transfer (e.g. a marketplace).
    pub auth_address: Address,
    /// The amount the owner has offered for transfer.
    pub offered_amount: u64,
}

/// ARC-18 Royalty Enforcer method signatures.
pub mod method {
    /// `set_administrator(address)void`.
    pub const SET_ADMINISTRATOR: &str = "set_administrator(address)void";
    /// `set_policy(uint64,account)void` — `royalty_basis`, `royalty_recipient`.
    pub const SET_POLICY: &str = "set_policy(uint64,account)void";
    /// `set_payment_asset(asset,bool)void` — opt an asset in/out as payment.
    pub const SET_PAYMENT_ASSET: &str = "set_payment_asset(asset,bool)void";
    /// `offer(asset,uint64,account,uint64,account)void` — flag transferable and
    /// set the authorised initiator (named `set_offer` in some app specs).
    pub const OFFER: &str = "offer(asset,uint64,account,uint64,account)void";
    /// `royalty_free_move(asset,uint64,account,account,uint64)void` — admin move
    /// with no royalty (e.g. escrow).
    pub const ROYALTY_FREE_MOVE: &str =
        "royalty_free_move(asset,uint64,account,account,uint64)void";
    /// `transfer_algo_payment(asset,uint64,account,account,account,pay,uint64)void`.
    pub const TRANSFER_ALGO_PAYMENT: &str =
        "transfer_algo_payment(asset,uint64,account,account,account,pay,uint64)void";
    /// `transfer_asset_payment(asset,uint64,account,account,account,axfer,asset,uint64)void`.
    pub const TRANSFER_ASSET_PAYMENT: &str =
        "transfer_asset_payment(asset,uint64,account,account,account,axfer,asset,uint64)void";
    /// `get_policy()(address,uint64)` — read-only: recipient + basis.
    pub const GET_POLICY: &str = "get_policy()(address,uint64)";
    /// `get_offer(asset,account)(address,uint64)` — read-only.
    pub const GET_OFFER: &str = "get_offer(asset,account)(address,uint64)";
    /// `get_administrator()address` — read-only.
    pub const GET_ADMINISTRATOR: &str = "get_administrator()address";

    /// Every ARC-18 method signature.
    pub const ALL: &[&str] = &[
        SET_ADMINISTRATOR,
        SET_POLICY,
        SET_PAYMENT_ASSET,
        OFFER,
        ROYALTY_FREE_MOVE,
        TRANSFER_ALGO_PAYMENT,
        TRANSFER_ASSET_PAYMENT,
        GET_POLICY,
        GET_OFFER,
        GET_ADMINISTRATOR,
    ];
}

/// The maximum royalty basis (100%), in basis points.
pub const MAX_BASIS_POINTS: u64 = 10_000;

impl RoyaltyPolicy {
    /// The royalty owed on a sale of `sale_amount`, truncated to whole units.
    ///
    /// `royalty_basis` is clamped to [`MAX_BASIS_POINTS`] (100%) so an
    /// out-of-range policy cannot overflow the intermediate product or return a
    /// nonsensical amount; with the clamp the `u128 → u64` result always fits.
    pub fn royalty_for(&self, sale_amount: u64) -> u64 {
        let basis = self.royalty_basis.min(MAX_BASIS_POINTS);
        (sale_amount as u128 * basis as u128 / MAX_BASIS_POINTS as u128) as u64
    }
}

/// Parse an ARC-18 method signature into an [`AbiMethod`].
pub fn method(signature: &str) -> Result<AbiMethod, crate::NftError> {
    Ok(AbiMethod::from_signature(signature)?)
}

/// Compute the 4-byte ARC-4 selector for an ARC-18 method signature.
pub fn selector(signature: &str) -> Result<[u8; 4], crate::NftError> {
    Ok(method(signature)?.get_selector()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_method_parses_including_txn_and_ref_types() {
        for sig in method::ALL {
            let sel = selector(sig).unwrap_or_else(|e| panic!("{sig}: {e}"));
            assert_eq!(sel.len(), 4);
        }
    }

    #[test]
    fn basis_point_royalty_math() {
        let p = RoyaltyPolicy {
            royalty_basis: 500, // 5%
            royalty_recipient: Address([1; 32]),
        };
        assert_eq!(p.royalty_for(1_000_000), 50_000);
        assert_eq!(p.royalty_for(0), 0);
    }

    #[test]
    fn out_of_range_basis_is_clamped_not_truncated() {
        // 500% would truncate on the u128 -> u64 cast; clamp to 100% instead.
        let p = RoyaltyPolicy {
            royalty_basis: 50_000,
            royalty_recipient: Address([1; 32]),
        };
        assert_eq!(p.royalty_for(1_000_000), 1_000_000);
    }
}
