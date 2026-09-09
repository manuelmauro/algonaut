//! Mint a pure (1-of-1) NFT with ARC-3 off-chain JSON metadata, against a live
//! algod. `NftMint` computes the ARC-3 metadata hash and applies the `#arc3` URL
//! convention for you, then we submit and read the real asset id back.
//!
//! Needs a sandbox: set `ALGOD_URL`, `ALGOD_TOKEN`, `ALICE_MNEMONIC` (a funded
//! account). Run: `cargo run --features nft --example nft_mint_arc3`

use algonaut::Algod;
use algonaut::nft::metadata::integrity;
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

    // Describe the NFT's off-chain metadata.
    let meta = Arc3Metadata {
        name: Some("Cosmic Cube #1".into()),
        description: Some("A one-of-a-kind cube.".into()),
        image: Some("ipfs://bafybeigdyrhz.../1.png".into()),
        image_mimetype: Some("image/png".into()),
        ..Default::default()
    };
    let metadata_url = "ipfs://bafybeigdyrhz.../1.json";

    // Mint: `.arc3` computes `am` and appends `#arc3` internally.
    let params = algod.suggested_params().await?;
    let txn = NftMint::pure(alice.address())
        .unit_name("CUBE")
        .asset_name("Cosmic Cube #1")
        .arc3(&meta, metadata_url)?
        .build(&params)?;

    let pending = algod.submit(&alice.sign(txn)?).await?;
    info!("submitted {}", pending.transaction_id());
    let asset_id = pending.confirm().await?.asset_index.expect("asset index");

    println!("== Pure NFT minted ==");
    println!("asset id : {asset_id}");
    println!("asset url: {metadata_url}#arc3");
    println!(
        "am (hex) : {}",
        hex(&integrity::metadata_hash(&meta.to_json()?, None))
    );
    println!("\nPublish this exact JSON at the asset url:");
    println!("{}", serde_json::to_string_pretty(&meta)?);
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
