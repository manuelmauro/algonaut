//! ARC-72 — the smart-contract NFT interface, plus ARC-73 interface detection.
//!
//! ARC-72 is Algorand's ERC-721 analogue: an NFT implemented as a smart contract
//! and addressed by `(app id, tokenId)`. Because it is an *application* standard,
//! this module does not introduce execution machinery — it provides the canonical
//! ARC-4 method signatures, a helper to resolve each to an
//! [`AbiMethod`](algonaut_abi::abi_interactions::AbiMethod) / 4-byte selector, and
//! the ARC-73 interface identifiers for capability detection. Compose and run the
//! calls through `algonaut::atomic`.

use algonaut_abi::abi_interactions::AbiMethod;

/// Core interface method signatures (ARC-73 id [`INTERFACE_CORE`]).
pub mod core {
    /// `arc72_ownerOf(uint256)address` — current owner (zero address if invalid).
    pub const OWNER_OF: &str = "arc72_ownerOf(uint256)address";
    /// `arc72_transferFrom(address,address,uint256)void` — transfer a token.
    pub const TRANSFER_FROM: &str = "arc72_transferFrom(address,address,uint256)void";
    /// All core method signatures.
    pub const ALL: &[&str] = &[OWNER_OF, TRANSFER_FROM];
}

/// Metadata extension (ARC-73 id [`INTERFACE_METADATA`]).
pub mod metadata {
    /// `arc72_tokenURI(uint256)byte[256]` — zero-padded URI for a token.
    pub const TOKEN_URI: &str = "arc72_tokenURI(uint256)byte[256]";
    /// All metadata method signatures.
    pub const ALL: &[&str] = &[TOKEN_URI];
}

/// Transfer-management extension (ARC-73 id [`INTERFACE_TRANSFER_MGMT`]).
pub mod transfer_management {
    /// `arc72_approve(address,uint256)void` — approve a controller for one token.
    pub const APPROVE: &str = "arc72_approve(address,uint256)void";
    /// `arc72_setApprovalForAll(address,bool)void` — approve an operator for all.
    pub const SET_APPROVAL_FOR_ALL: &str = "arc72_setApprovalForAll(address,bool)void";
    /// `arc72_getApproved(uint256)address` — the approved controller of a token.
    pub const GET_APPROVED: &str = "arc72_getApproved(uint256)address";
    /// `arc72_isApprovedForAll(address,address)bool` — operator approval query.
    pub const IS_APPROVED_FOR_ALL: &str = "arc72_isApprovedForAll(address,address)bool";
    /// All transfer-management method signatures.
    pub const ALL: &[&str] = &[
        APPROVE,
        SET_APPROVAL_FOR_ALL,
        GET_APPROVED,
        IS_APPROVED_FOR_ALL,
    ];
}

/// Enumeration extension (ARC-73 id [`INTERFACE_ENUMERATION`]).
pub mod enumeration {
    /// `arc72_balanceOf(address)uint256` — tokens owned by an address.
    pub const BALANCE_OF: &str = "arc72_balanceOf(address)uint256";
    /// `arc72_totalSupply()uint256` — total tokens.
    pub const TOTAL_SUPPLY: &str = "arc72_totalSupply()uint256";
    /// `arc72_tokenByIndex(uint256)uint256` — token id at an index.
    pub const TOKEN_BY_INDEX: &str = "arc72_tokenByIndex(uint256)uint256";
    /// All enumeration method signatures.
    pub const ALL: &[&str] = &[BALANCE_OF, TOTAL_SUPPLY, TOKEN_BY_INDEX];
}

/// ARC-73 interface id of the ARC-72 core interface.
pub const INTERFACE_CORE: [u8; 4] = [0x53, 0xf0, 0x2a, 0x40];
/// ARC-73 interface id of the metadata extension.
pub const INTERFACE_METADATA: [u8; 4] = [0xc3, 0xc1, 0xfc, 0x00];
/// ARC-73 interface id of the transfer-management extension.
pub const INTERFACE_TRANSFER_MGMT: [u8; 4] = [0xb9, 0xc6, 0xf6, 0x96];
/// ARC-73 interface id of the enumeration extension.
pub const INTERFACE_ENUMERATION: [u8; 4] = [0xa5, 0x7d, 0x46, 0x79];

/// Parse an ARC-72 method signature into an [`AbiMethod`].
pub fn method(signature: &str) -> Result<AbiMethod, crate::NftError> {
    Ok(AbiMethod::from_signature(signature)?)
}

/// Compute the 4-byte ARC-4 selector for an ARC-72 method signature.
pub fn selector(signature: &str) -> Result<[u8; 4], crate::NftError> {
    Ok(method(signature)?.get_selector()?)
}

/// Every ARC-72 method signature across the core interface and all extensions.
pub fn all_signatures() -> Vec<&'static str> {
    let mut v = Vec::new();
    v.extend_from_slice(core::ALL);
    v.extend_from_slice(metadata::ALL);
    v.extend_from_slice(transfer_management::ALL);
    v.extend_from_slice(enumeration::ALL);
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_signature_parses_and_selects() {
        for sig in all_signatures() {
            let sel = selector(sig).unwrap_or_else(|e| panic!("{sig}: {e}"));
            assert_eq!(sel.len(), 4);
        }
    }

    #[test]
    fn owner_of_round_trips_through_abimethod() {
        let m = method(core::OWNER_OF).unwrap();
        assert_eq!(m.get_signature(), core::OWNER_OF);
    }
}
