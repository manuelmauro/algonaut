//! The *reading* side: resolve an ARC-19 pointer and verify integrity offline.
//!
//! A marketplace reads an asset's params, resolves the metadata URL, fetches the
//! JSON (with its own transport — the crate stays transport-agnostic by default),
//! then verifies the bytes against the on-chain `am` hash and each resource
//! against its `*_integrity` string.
//!
//! Run: `cargo run -p algonaut_nft --example resolve_metadata`

use algonaut_core::Address;
use algonaut_nft::metadata::{arc3, integrity};
use algonaut_nft::prelude::*;
use algonaut_nft::url::{Cid, CidVersion, Codec};
use sha2::{Digest, Sha256};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- minter side: author metadata, pin its hash, publish via ARC-19 ---
    let image_bytes = b"...the PNG bytes...";
    let meta = arc3::Metadata {
        name: Some("Verified Piece".into()),
        image: Some("ipfs://bafy.../piece.png".into()),
        image_integrity: Some(integrity::sri_sha256(image_bytes)), // SRI computed for us
        ..Default::default()
    };
    let json = meta.to_json()?;
    let am = integrity::metadata_hash(&json, None); // goes on-chain as `am`
    let cid = Cid {
        version: CidVersion::V1,
        codec: Codec::Raw,
        digest: Sha256::digest(&json).into(),
    };
    let reserve = Address::from(cid); // goes on-chain as the Reserve

    // --- reader side: only the on-chain facts are known ---
    let asset_url = TemplateIpfsUrl::from_cid(&cid, "").to_string();
    let resolved = algonaut_nft::url::resolve_url(&asset_url, reserve)?;
    println!("== Resolve & verify ==");
    println!("asset url   : {asset_url}");
    println!("resolved    : {resolved}");

    // The reader fetches `json` from `resolved` (transport is theirs) and verifies.
    match integrity::verify_metadata_hash(&json, None, &am) {
        Ok(()) => println!("metadata am : OK"),
        Err(e) => println!("metadata am : FAILED ({e})"),
    }

    // Then verifies the referenced image against its SRI.
    let sri = meta.image_integrity.as_deref().unwrap();
    match integrity::verify_sri(image_bytes, sri, "image") {
        Ok(()) => println!("image SRI   : OK"),
        Err(e) => println!("image SRI   : FAILED ({e})"),
    }

    // Tampering is caught.
    assert!(integrity::verify_sri(b"tampered", sri, "image").is_err());
    println!("tamper test : rejected as expected");
    Ok(())
}
