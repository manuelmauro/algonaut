//! ARC-89 preview: build, encode, decode, and hash an Asset Metadata Box.
//!
//! ARC-89 (Last Call) stores ASA metadata in a box of a singleton registry app.
//! This crate's `arc89` module is an offline, byte-exact codec for that box —
//! marked unstable until ARC-89 reaches Final. This example builds a box, sets
//! its flags via `with_*` setters (no hex literals), encodes it, decodes it back,
//! and computes the domain-separated metadata hash.
//!
//! Run: `cargo run -p algonaut_nft --example arc89_preview`

use algonaut_core::AssetId;
use algonaut_nft::arc89::{
    IrreversibleFlags, MetadataBox, MetadataIdentifiers, ReversibleFlags, TESTNET_REGISTRY_APP_ID,
    partial_uri,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let asset_id = AssetId(753_324_085);
    let body = br#"{"name":"Registry NFT","standard":"arc89"}"#.to_vec();

    let mut metadata_box = MetadataBox {
        identifiers: MetadataIdentifiers::default().with_short(true),
        reversible: ReversibleFlags::default().with_arc62_circulating_supply(true),
        irreversible: IrreversibleFlags::default()
            .with_arc3_compliant(true)
            .with_arc89_native(true),
        hash: [0; 32],
        last_modified_round: 41_000_000,
        deprecated_by: 0,
        metadata: body,
    };

    // The hash is computed for us (domain-separated over header + pages).
    metadata_box.set_hash(asset_id)?;

    let encoded = metadata_box.encode();
    let decoded = MetadataBox::decode(&encoded)?;
    assert_eq!(decoded, metadata_box);

    println!("== ARC-89 Asset Metadata Box (preview) ==");
    println!("asset id     : {}", asset_id.0);
    println!("box name (be): {}", hex(&MetadataBox::box_name(asset_id)));
    println!("box size     : {} bytes (51 header + body)", encoded.len());
    println!("short?       : {}", decoded.is_short());
    println!(
        "arc3/arc89   : {} / {}",
        decoded.irreversible.arc3_compliant(),
        decoded.irreversible.arc89_native()
    );
    println!(
        "arc62 supply : {}",
        decoded.reversible.arc62_circulating_supply()
    );
    println!("metadata hash: {}", hex(&decoded.hash));
    println!(
        "testnet uri  : {}",
        partial_uri(TESTNET_REGISTRY_APP_ID, Some("net:testnet"))
    );
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
