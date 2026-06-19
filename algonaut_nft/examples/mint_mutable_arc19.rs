//! Mint a *mutable* NFT with ARC-19, and show how an update works.
//!
//! ARC-19 stores the IPFS content hash in the ASA's Reserve address. The crate
//! does the CID ↔ reserve-address math: you hand `NftMint::arc19` a [`Cid`] and
//! it sets both the `template-ipfs://…` URL and the Reserve. To later repoint the
//! NFT you change only the Reserve to `Address::from(new_cid)`.
//!
//! Run: `cargo run -p algonaut_nft --example mint_mutable_arc19`

use algonaut_core::Address;
use algonaut_nft::metadata::arc3;
use algonaut_nft::prelude::*;
use algonaut_nft::url::{Cid, CidVersion, Codec};
use sha2::{Digest, Sha256};

/// The CIDv1 (raw, sha2-256) of a small single-block file — exactly what
/// `ipfs add --cid-version=1 --raw-leaves` produces.
fn cid_of(bytes: &[u8]) -> Cid {
    Cid {
        version: CidVersion::V1,
        codec: Codec::Raw,
        digest: Sha256::digest(bytes).into(),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let creator = Address([0x22; 32]);

    // Version 1 of the metadata.
    let v1 = arc3::Metadata {
        name: Some("Evolving Avatar".into()),
        image: Some("ipfs://bafy.../v1.png".into()),
        ..Default::default()
    };
    let cid_v1 = cid_of(&v1.to_json()?);

    // Mint: `.arc19` sets the templated URL and the Reserve from the CID.
    let _create = NftMint::pure(creator)
        .unit_name("AVATAR")
        .asset_name("Evolving Avatar")
        .arc19(&cid_v1, "")
        .into_create();

    let reserve_v1 = Address::from(cid_v1);
    let template = TemplateIpfsUrl::from_cid(&cid_v1, "");

    println!("== ARC-19 mutable NFT ==");
    println!("content CID : {cid_v1}");
    println!("asset url   : {template}");
    println!("reserve     : {reserve_v1}");
    println!("resolves to : {}", template.resolve(reserve_v1));

    // Later: new artwork → new CID → new Reserve. Only the Reserve changes; the
    // asset URL template stays the same.
    let v2 = arc3::Metadata {
        name: Some("Evolving Avatar".into()),
        image: Some("ipfs://bafy.../v2.png".into()),
        ..Default::default()
    };
    let cid_v2 = cid_of(&v2.to_json()?);
    let reserve_v2 = Address::from(cid_v2);

    println!("\n-- update (send UpdateAsset with this reserve) --");
    println!("new CID     : {cid_v2}");
    println!("new reserve : {reserve_v2}");
    println!("resolves to : {}", template.resolve(reserve_v2));
    assert_ne!(reserve_v1, reserve_v2);
    Ok(())
}
