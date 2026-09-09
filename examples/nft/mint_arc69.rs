//! Mint an NFT whose metadata lives on-chain via ARC-69 — against a live algod.
//!
//! ARC-69 puts the metadata JSON in the asset-config note; `NftMint::arc69`
//! places it there for you. The asset URL points at the media.
//!
//! Needs a sandbox: `ALGOD_URL`, `ALGOD_TOKEN`, `ALICE_MNEMONIC`.
//! Run: `cargo run --features nft --example nft_mint_arc69`

use algonaut::Algod;
use algonaut::nft::metadata::arc69::MediaType;
use algonaut::nft::prelude::*;
use algonaut::transaction::account::Account;
use dotenv::dotenv;
use std::env;
use std::error::Error;
#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    env_logger::init();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;
    let alice = Account::from_mnemonic(&env::var("ALICE_MNEMONIC")?)?;

    let media_url = "ipfs://bafy.../clip.mp4#v"; // `#v` signals video
    let meta = Arc69Metadata {
        description: Some("On-chain music video.".into()),
        media_url: Some(media_url.into()),
        mime_type: Some("video/mp4".into()),
        properties: Some(serde_json::json!({ "artist": "Nova", "edition": 1 })),
        ..Default::default()
    };

    let params = algod.suggested_params().await?;
    let txn = NftMint::pure(alice.address())
        .unit_name("CLIP")
        .asset_name("Nova — Clip")
        .url(media_url)
        .arc69(&meta)?
        .build(&params)?;

    let pending = algod.submit(&alice.sign(txn)?).await?;
    info!("submitted {}", pending.transaction_id());
    let asset_id = pending.confirm().await?.asset_index.expect("asset index");

    println!("== ARC-69 on-chain NFT minted ==");
    println!("asset id  : {asset_id}");
    println!("media url : {media_url}");
    println!("media type: {:?}", MediaType::from_url(media_url));
    println!("\nacfg note (the on-chain metadata):");
    println!("{}", serde_json::to_string_pretty(&meta)?);
    Ok(())
}
