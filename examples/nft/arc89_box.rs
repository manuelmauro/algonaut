//! ARC-89 preview: mint a real NFT, then build its Asset Metadata Box (offline
//! codec) keyed by the real asset id.
//!
//! ARC-89 (Last Call) stores ASA metadata in a box of a singleton registry app,
//! keyed by the 8-byte big-endian asset id. This crate's `arc89` module is an
//! offline, byte-exact codec for that box (marked unstable until ARC-89 is
//! Final); the on-chain registry client is not implemented yet, so we mint a
//! plain NFT for a real id and build the box for it locally.
//!
//! Needs a sandbox: `ALGOD_URL`, `ALGOD_TOKEN`, `ALICE_MNEMONIC`.
//! Run: `cargo run --features nft --example nft_arc89_box`

use algonaut::Algod;
use algonaut::core::AssetId;
use algonaut::nft::arc89::{
    IrreversibleFlags, MetadataBox, MetadataIdentifiers, ReversibleFlags, TESTNET_REGISTRY_APP_ID,
    partial_uri,
};
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

    // Mint a plain NFT to obtain a real asset id.
    let params = algod.suggested_params().await?;
    let txn = NftMint::pure(alice.address())
        .unit_name("REG")
        .asset_name("Registry NFT")
        .build(&params)?;
    let asset_id = AssetId(
        algod
            .submit(&alice.sign(txn)?)
            .await?
            .confirm()
            .await?
            .asset_index
            .expect("asset index"),
    );
    info!("minted asset {asset_id}");

    // Build the ARC-89 Asset Metadata Box for that real id (offline codec).
    let body = br#"{"name":"Registry NFT","standard":"arc89"}"#.to_vec();
    let mut metadata_box = MetadataBox {
        identifiers: MetadataIdentifiers::default().with_short(true),
        reversible: ReversibleFlags::default().with_arc62_circulating_supply(true),
        irreversible: IrreversibleFlags::default()
            .with_arc3_compliant(true)
            .with_arc89_native(true),
        hash: [0; 32],
        last_modified_round: 0,
        deprecated_by: 0,
        metadata: body,
    };
    metadata_box.set_hash(asset_id)?; // domain-separated hash computed for us

    let encoded = metadata_box.encode();
    assert_eq!(MetadataBox::decode(&encoded)?, metadata_box);

    println!("== ARC-89 Asset Metadata Box (preview) ==");
    println!("asset id     : {asset_id}");
    println!("box name (be): {}", hex(&MetadataBox::box_name(asset_id)));
    println!("box size     : {} bytes (51 header + body)", encoded.len());
    println!("metadata hash: {}", hex(&metadata_box.hash));
    println!(
        "testnet uri  : {}",
        partial_uri(TESTNET_REGISTRY_APP_ID, Some("net:testnet"))
    );
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
