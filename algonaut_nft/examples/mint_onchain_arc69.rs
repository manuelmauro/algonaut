//! Mint an NFT whose metadata lives *on-chain* via ARC-69.
//!
//! ARC-69 puts the metadata JSON in the asset-config transaction note (so it
//! works on assets that already exist and can be updated by the manager). The
//! asset URL points at the media; `NftMint::arc69` places the metadata in the
//! note for you.
//!
//! Run: `cargo run -p algonaut_nft --example mint_onchain_arc69`

use algonaut_core::Address;
use algonaut_crypto::HashDigest;
use algonaut_nft::metadata::arc69::{self, MediaType};
use algonaut_nft::prelude::*;
use algonaut_transaction::builder::TransactionParams;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let creator = Address([0x33; 32]);

    let media_url = "ipfs://bafy.../clip.mp4#v"; // `#v` signals video
    let meta = Arc69Metadata {
        description: Some("On-chain music video.".into()),
        media_url: Some(media_url.into()),
        mime_type: Some("video/mp4".into()),
        properties: Some(serde_json::json!({ "artist": "Nova", "edition": 1 })),
        ..Default::default()
    };

    let txn = NftMint::pure(creator)
        .unit_name("CLIP")
        .asset_name("Nova — Clip")
        .url(media_url)
        .arc69(&meta)?
        .build(&demo_params())?;

    println!("== ARC-69 on-chain NFT ==");
    println!("creator   : {creator}");
    println!("media url : {media_url}");
    println!("media type: {:?}", MediaType::from_url(media_url));
    println!("txn id    : {}", txn.id()?);
    println!("\nacfg note (the on-chain metadata):");
    println!("{}", serde_json::to_string_pretty(&meta)?);

    // Round-trips back from raw note bytes.
    let note = meta.to_note()?;
    let parsed = arc69::Metadata::from_note(&note)?;
    assert_eq!(parsed.mime_type.as_deref(), Some("video/mp4"));
    Ok(())
}

fn demo_params() -> impl TransactionParams {
    struct Demo {
        genesis_id: String,
    }
    impl TransactionParams for Demo {
        fn last_round(&self) -> u64 {
            1
        }
        fn min_fee(&self) -> u64 {
            1_000
        }
        fn genesis_hash(&self) -> HashDigest {
            HashDigest([0; 32])
        }
        fn genesis_id(&self) -> &String {
            &self.genesis_id
        }
    }
    Demo {
        genesis_id: "testnet-v1.0".into(),
    }
}
