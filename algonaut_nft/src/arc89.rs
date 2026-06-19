//! ARC-89 — ASA Metadata Registry (**preview**).
//!
//! > **Unstable.** ARC-89 is in Last Call and supersedes the ARC-19/69 line. This
//! > module is a *preview*: an offline, byte-exact codec for the on-chain Asset
//! > Metadata Box plus the registry constants, with no compatibility guarantee
//! > until ARC-89 reaches Final. The registry's on-chain ABI client is not yet
//! > implemented; its method signatures are documented in [`method`].
//!
//! ARC-89 stores ASA metadata in a box of a singleton registry application, keyed
//! by the 8-byte big-endian asset id. Each box value is a fixed **51-byte header**
//! (metadata-identifier, reversible-flag and irreversible-flag bytes, a 32-byte
//! metadata hash, the last-modified round, and a "deprecated-by" registry id)
//! followed by the metadata body (a UTF-8 JSON object). This module encodes and
//! decodes that value and recomputes the domain-separated metadata hash.

use algonaut_core::{AppId, AssetId};
use sha2::{Digest, Sha512_256};

/// Box name size: the 8-byte big-endian asset id.
pub const BOX_KEY_SIZE: usize = 8;
/// Fixed header size in bytes.
pub const HEADER_SIZE: usize = 51;
/// Metadata at or below this size is flagged "short".
pub const SHORT_METADATA_SIZE: usize = 4096;
/// Hash/encoding page size in bytes.
pub const PAGE_SIZE: usize = 1007;
/// Maximum metadata body size in bytes.
pub const MAX_METADATA_SIZE: usize = 30506;

// Header field byte offsets.
const OFF_IDENTIFIERS: usize = 0;
const OFF_REVERSIBLE: usize = 1;
const OFF_IRREVERSIBLE: usize = 2;
const OFF_HASH: usize = 3;
const OFF_LAST_MODIFIED: usize = 35;
const OFF_DEPRECATED_BY: usize = 43;

const HEADER_PREFIX: &[u8] = b"arc0089/header";
const PAGE_PREFIX: &[u8] = b"arc0089/page";
const AM_PREFIX: &[u8] = b"arc0089/am";

/// The trusted ARC-89 registry app id on TestNet.
pub const TESTNET_REGISTRY_APP_ID: AppId = AppId(753324084);
/// MainNet genesis hash (base64); the MainNet registry app id is not yet assigned.
pub const MAINNET_GENESIS_HASH: &str = "wGHE2Pwdvd7S12BL5FaOP20EGYesN73ktiC1qzkkit8=";
/// TestNet genesis hash (base64).
pub const TESTNET_GENESIS_HASH: &str = "SGO1GKSzyE7IEPItTxCByw9x8FmnrCDexi9/cOUJOiI=";

/// Metadata Identifiers byte (set by the registry).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MetadataIdentifiers(pub u8);

impl MetadataIdentifiers {
    const SHORT: u8 = 0x80;
    /// Whether the "short metadata" bit (MSB) is set.
    pub fn is_short(self) -> bool {
        self.0 & Self::SHORT != 0
    }
    /// Set or clear the short-metadata bit.
    pub fn with_short(self, short: bool) -> Self {
        MetadataIdentifiers(set_bit(self.0, Self::SHORT, short))
    }
}

/// Reversible Flags byte (two-way switches set by the ASA manager).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReversibleFlags(pub u8);

impl ReversibleFlags {
    const ARC20: u8 = 0x01;
    const ARC62: u8 = 0x02;
    const NTT: u8 = 0x04;
    /// ARC-20 Smart ASA.
    pub fn arc20_smart_asa(self) -> bool {
        self.0 & Self::ARC20 != 0
    }
    /// ARC-62 circulating supply.
    pub fn arc62_circulating_supply(self) -> bool {
        self.0 & Self::ARC62 != 0
    }
    /// Native Token Transfers (NTT) supported.
    pub fn ntt(self) -> bool {
        self.0 & Self::NTT != 0
    }
    /// Set or clear the ARC-20 Smart ASA bit.
    pub fn with_arc20_smart_asa(self, on: bool) -> Self {
        ReversibleFlags(set_bit(self.0, Self::ARC20, on))
    }
    /// Set or clear the ARC-62 circulating-supply bit.
    pub fn with_arc62_circulating_supply(self, on: bool) -> Self {
        ReversibleFlags(set_bit(self.0, Self::ARC62, on))
    }
    /// Set or clear the Native Token Transfers bit.
    pub fn with_ntt(self, on: bool) -> Self {
        ReversibleFlags(set_bit(self.0, Self::NTT, on))
    }
}

/// Irreversible Flags byte (one-way switches set by the ASA manager).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IrreversibleFlags(pub u8);

impl IrreversibleFlags {
    const ARC3: u8 = 0x01;
    const ARC89_NATIVE: u8 = 0x02;
    const ARC54_BURNABLE: u8 = 0x04;
    const IMMUTABLE: u8 = 0x80;
    /// ARC-3 compliant (settable only at creation).
    pub fn arc3_compliant(self) -> bool {
        self.0 & Self::ARC3 != 0
    }
    /// ARC-89 native ASA (settable only at creation).
    pub fn arc89_native(self) -> bool {
        self.0 & Self::ARC89_NATIVE != 0
    }
    /// ARC-54 burnable ASA.
    pub fn arc54_burnable(self) -> bool {
        self.0 & Self::ARC54_BURNABLE != 0
    }
    /// Metadata immutability (MSB) — once set, never cleared.
    pub fn is_immutable(self) -> bool {
        self.0 & Self::IMMUTABLE != 0
    }
    /// Set or clear the ARC-3 compliant bit (settable only at creation).
    pub fn with_arc3_compliant(self, on: bool) -> Self {
        IrreversibleFlags(set_bit(self.0, Self::ARC3, on))
    }
    /// Set or clear the ARC-89 native ASA bit (settable only at creation).
    pub fn with_arc89_native(self, on: bool) -> Self {
        IrreversibleFlags(set_bit(self.0, Self::ARC89_NATIVE, on))
    }
    /// Set or clear the ARC-54 burnable bit.
    pub fn with_arc54_burnable(self, on: bool) -> Self {
        IrreversibleFlags(set_bit(self.0, Self::ARC54_BURNABLE, on))
    }
    /// Set the metadata-immutability bit (one-way; cannot be cleared once set).
    pub fn with_immutable(self) -> Self {
        IrreversibleFlags(self.0 | Self::IMMUTABLE)
    }
}

fn set_bit(byte: u8, mask: u8, on: bool) -> u8 {
    if on { byte | mask } else { byte & !mask }
}

/// A decoded ARC-89 Asset Metadata Box value (header + body).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetadataBox {
    /// Registry-set identifier bits.
    pub identifiers: MetadataIdentifiers,
    /// Manager-set reversible flag bits.
    pub reversible: ReversibleFlags,
    /// Manager-set irreversible flag bits.
    pub irreversible: IrreversibleFlags,
    /// The 32-byte metadata hash stored in the header.
    pub hash: [u8; 32],
    /// Round of last modification.
    pub last_modified_round: u64,
    /// App id of a newer registry this metadata migrated to (0 if none).
    pub deprecated_by: u64,
    /// The metadata body (a UTF-8 JSON object).
    pub metadata: Vec<u8>,
}

impl MetadataBox {
    /// The box name for an asset: the 8-byte big-endian asset id.
    pub fn box_name(asset_id: AssetId) -> [u8; BOX_KEY_SIZE] {
        asset_id.0.to_be_bytes()
    }

    /// Whether the body qualifies as "short" (≤ [`SHORT_METADATA_SIZE`]).
    pub fn is_short(&self) -> bool {
        self.metadata.len() <= SHORT_METADATA_SIZE
    }

    /// Encode to the on-chain box value (`HEADER_SIZE` bytes + body).
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(HEADER_SIZE + self.metadata.len());
        out.push(self.identifiers.0);
        out.push(self.reversible.0);
        out.push(self.irreversible.0);
        out.extend_from_slice(&self.hash);
        out.extend_from_slice(&self.last_modified_round.to_be_bytes());
        out.extend_from_slice(&self.deprecated_by.to_be_bytes());
        out.extend_from_slice(&self.metadata);
        out
    }

    /// Decode an on-chain box value.
    pub fn decode(bytes: &[u8]) -> Result<Self, crate::NftError> {
        if bytes.len() < HEADER_SIZE {
            return Err(crate::NftError::InvalidMetadataBox(format!(
                "box is {} bytes, need at least {HEADER_SIZE}",
                bytes.len()
            )));
        }
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&bytes[OFF_HASH..OFF_LAST_MODIFIED]);
        Ok(MetadataBox {
            identifiers: MetadataIdentifiers(bytes[OFF_IDENTIFIERS]),
            reversible: ReversibleFlags(bytes[OFF_REVERSIBLE]),
            irreversible: IrreversibleFlags(bytes[OFF_IRREVERSIBLE]),
            hash,
            last_modified_round: u64::from_be_bytes(
                bytes[OFF_LAST_MODIFIED..OFF_DEPRECATED_BY]
                    .try_into()
                    .unwrap(),
            ),
            deprecated_by: u64::from_be_bytes(
                bytes[OFF_DEPRECATED_BY..HEADER_SIZE].try_into().unwrap(),
            ),
            metadata: bytes[HEADER_SIZE..].to_vec(),
        })
    }

    /// Recompute the domain-separated ARC-89 metadata hash for this box.
    ///
    /// The hash covers the asset id, identifier/flag bytes, metadata size, and
    /// the page-wise body — but never the hash field itself, the last-modified
    /// round, or the deprecated-by field.
    pub fn compute_hash(&self, asset_id: AssetId) -> Result<[u8; 32], crate::NftError> {
        let size: u16 = self.metadata.len().try_into().map_err(|_| {
            crate::NftError::InvalidMetadataBox(format!(
                "metadata is {} bytes, exceeds {MAX_METADATA_SIZE}",
                self.metadata.len()
            ))
        })?;
        if self.metadata.len() > MAX_METADATA_SIZE {
            return Err(crate::NftError::InvalidMetadataBox(format!(
                "metadata is {} bytes, exceeds {MAX_METADATA_SIZE}",
                self.metadata.len()
            )));
        }
        let asset = asset_id.0.to_be_bytes();

        // Header hash.
        let hh = sha512_256(&[
            HEADER_PREFIX,
            &asset,
            &[self.identifiers.0],
            &[self.reversible.0],
            &[self.irreversible.0],
            &size.to_be_bytes(),
        ]);

        // Per-page hashes.
        let mut am_parts: Vec<Vec<u8>> = vec![AM_PREFIX.to_vec(), hh.to_vec()];
        for (i, page) in self.metadata.chunks(PAGE_SIZE).enumerate() {
            let page_size = page.len() as u16;
            let ph = sha512_256(&[
                PAGE_PREFIX,
                &asset,
                &[i as u8],
                &page_size.to_be_bytes(),
                page,
            ]);
            am_parts.push(ph.to_vec());
        }

        let refs: Vec<&[u8]> = am_parts.iter().map(|v| v.as_slice()).collect();
        Ok(sha512_256(&refs))
    }

    /// Recompute and store the metadata hash in the header.
    pub fn set_hash(&mut self, asset_id: AssetId) -> Result<(), crate::NftError> {
        self.hash = self.compute_hash(asset_id)?;
        Ok(())
    }
}

/// The ARC-89 registry ABI method signatures (documented for the future client).
pub mod method {
    /// `arc89_create_metadata(uint64,byte,byte,uint16,byte[],pay)(uint8,uint64)`.
    pub const CREATE_METADATA: &str =
        "arc89_create_metadata(uint64,byte,byte,uint16,byte[],pay)(uint8,uint64)";
    /// `arc89_replace_metadata(uint64,uint16,byte[])(uint8,uint64)`.
    pub const REPLACE_METADATA: &str = "arc89_replace_metadata(uint64,uint16,byte[])(uint8,uint64)";
    /// `arc89_delete_metadata(uint64)(uint8,uint64)`.
    pub const DELETE_METADATA: &str = "arc89_delete_metadata(uint64)(uint8,uint64)";
    /// `arc89_set_immutable(uint64)void`.
    pub const SET_IMMUTABLE: &str = "arc89_set_immutable(uint64)void";
    /// `arc89_get_metadata_header(uint64)(byte,byte,byte,byte[32],uint64,uint64)`.
    pub const GET_METADATA_HEADER: &str =
        "arc89_get_metadata_header(uint64)(byte,byte,byte,byte[32],uint64,uint64)";
    /// `arc89_get_metadata_hash(uint64)byte[32]`.
    pub const GET_METADATA_HASH: &str = "arc89_get_metadata_hash(uint64)byte[32]";
}

/// Build the partial ARC-90 asset URL for an ARC-89 registry box.
///
/// `netauth` is `None` for MainNet (the authority is empty) and `Some("net:testnet")`
/// (or another network authority) otherwise. Append a fragment such as `#arc89`
/// or `#arc3` to complete it.
pub fn partial_uri(registry: AppId, netauth: Option<&str>) -> String {
    match netauth {
        None => format!("algorand://app/{}?box=", registry.0),
        Some(auth) => format!("algorand://{}/app/{}?box=", auth, registry.0),
    }
}

fn sha512_256(parts: &[&[u8]]) -> [u8; 32] {
    let mut h = Sha512_256::new();
    for p in parts {
        h.update(p);
    }
    h.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(metadata: Vec<u8>) -> MetadataBox {
        MetadataBox {
            identifiers: MetadataIdentifiers::default().with_short(true),
            reversible: ReversibleFlags(0x01),
            irreversible: IrreversibleFlags(0x03), // arc3 + arc89-native
            hash: [0u8; 32],
            last_modified_round: 12345,
            deprecated_by: 0,
            metadata,
        }
    }

    #[test]
    fn box_name_is_big_endian_asset_id() {
        assert_eq!(MetadataBox::box_name(AssetId(1)), [0, 0, 0, 0, 0, 0, 0, 1]);
        assert_eq!(
            MetadataBox::box_name(AssetId(0x0102030405060708)),
            [1, 2, 3, 4, 5, 6, 7, 8]
        );
    }

    #[test]
    fn header_round_trips_through_encode_decode() {
        let b = sample(br#"{"name":"x"}"#.to_vec());
        let bytes = b.encode();
        assert_eq!(bytes.len(), HEADER_SIZE + b.metadata.len());
        assert_eq!(MetadataBox::decode(&bytes).unwrap(), b);
    }

    #[test]
    fn decode_rejects_short_buffer() {
        assert!(matches!(
            MetadataBox::decode(&[0u8; 10]),
            Err(crate::NftError::InvalidMetadataBox(_))
        ));
    }

    #[test]
    fn flag_accessors() {
        let irr = IrreversibleFlags(0x83); // arc3 + arc89-native + immutable
        assert!(irr.arc3_compliant());
        assert!(irr.arc89_native());
        assert!(!irr.arc54_burnable());
        assert!(irr.is_immutable());
        assert!(MetadataIdentifiers(0x80).is_short());
        assert!(ReversibleFlags(0x06).arc62_circulating_supply());
        assert!(ReversibleFlags(0x06).ntt());
    }

    #[test]
    fn hash_is_deterministic_and_flag_sensitive() {
        let b = sample(br#"{"name":"x"}"#.to_vec());
        let h1 = b.compute_hash(AssetId(7)).unwrap();
        assert_eq!(h1, b.compute_hash(AssetId(7)).unwrap());
        // A different asset id changes the hash (it is mixed into the header).
        assert_ne!(h1, b.compute_hash(AssetId(8)).unwrap());

        // A changed flag byte changes the hash.
        let mut b2 = b.clone();
        b2.reversible = ReversibleFlags(0x02);
        assert_ne!(h1, b2.compute_hash(AssetId(7)).unwrap());
    }

    #[test]
    fn empty_metadata_hashes_without_pages() {
        let b = sample(Vec::new());
        // Should not panic and should differ from a one-byte body.
        let empty = b.compute_hash(AssetId(1)).unwrap();
        let one = sample(vec![b'{']).compute_hash(AssetId(1)).unwrap();
        assert_ne!(empty, one);
    }

    #[test]
    fn multi_page_metadata_hashes() {
        // Larger than one page exercises the page loop.
        let b = sample(vec![b'a'; PAGE_SIZE * 2 + 5]);
        let h = b.compute_hash(AssetId(1)).unwrap();
        assert_eq!(h, b.compute_hash(AssetId(1)).unwrap());
    }

    #[test]
    fn set_hash_fills_header() {
        let mut b = sample(br#"{"k":1}"#.to_vec());
        assert_eq!(b.hash, [0u8; 32]);
        b.set_hash(AssetId(9)).unwrap();
        assert_ne!(b.hash, [0u8; 32]);
    }

    #[test]
    fn partial_uri_forms() {
        assert_eq!(
            partial_uri(AppId(753324084), Some("net:testnet")),
            "algorand://net:testnet/app/753324084?box="
        );
        assert_eq!(partial_uri(AppId(123), None), "algorand://app/123?box=");
    }
}
