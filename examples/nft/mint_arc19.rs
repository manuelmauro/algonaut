//! Mint a *mutable* NFT with ARC-19 and then update it — against a live algod.
//!
//! ARC-19 stores the IPFS content hash in the Reserve address; `NftMint::arc19`
//! sets the templated URL and the Reserve from a [`Cid`]. Updating is a single
//! `UpdateAsset` that changes only the Reserve (keeping the manager).
//!
//! Needs a sandbox: `ALGOD_URL`, `ALGOD_TOKEN`, `ALICE_MNEMONIC`.
//! Run: `cargo run --features nft --example nft_mint_arc19`

use algonaut::Algod;
use algonaut::core::{Address, AssetId};
use algonaut::nft::metadata::arc3;
use algonaut::nft::prelude::*;
use algonaut::nft::url::{Cid, Codec};
use algonaut::transaction::UpdateAsset;
use algonaut::transaction::account::Account;
use dotenv::dotenv;
use sha2::{Digest, Sha256};
use std::env;
use std::error::Error;
#[macro_use]
extern crate log;

/// The CIDv1 (raw, sha2-256) of a small single-block file — what
/// `ipfs add --cid-version=1 --raw-leaves` produces.
fn cid_of(bytes: &[u8]) -> Cid {
    Cid::v1(Codec::Raw, Sha256::digest(bytes).into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    env_logger::init();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;
    let alice = Account::from_mnemonic(&env::var("ALICE_MNEMONIC")?)?;

    // --- mint, pointing at version 1 of the metadata ---
    let v1 = arc3::Metadata {
        name: Some("Evolving Avatar".into()),
        image: Some("ipfs://bafy.../v1.png".into()),
        ..Default::default()
    };
    let cid_v1 = cid_of(&v1.to_json()?);

    let params = algod.suggested_params().await?;
    let txn = NftMint::pure(alice.address())
        .unit_name("AVATAR")
        .asset_name("Evolving Avatar")
        .manager(alice.address()) // keep control so we can update later
        .arc19(&cid_v1, "")
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

    let template = TemplateIpfsUrl::from_cid(&cid_v1, "");
    println!("== ARC-19 mutable NFT ==");
    println!("asset id : {asset_id}");
    println!("asset url: {template}");
    println!("resolves : {}", template.resolve(Address::from(cid_v1)));

    // --- update: new artwork → new CID → new Reserve (manager kept) ---
    let v2 = arc3::Metadata {
        name: Some("Evolving Avatar".into()),
        image: Some("ipfs://bafy.../v2.png".into()),
        ..Default::default()
    };
    let cid_v2 = cid_of(&v2.to_json()?);

    let params = algod.suggested_params().await?;
    let update = UpdateAsset::new(alice.address(), asset_id)
        .manager(alice.address())
        .reserve(Address::from(cid_v2))
        .build(&params)?;
    algod.submit(&alice.sign(update)?).await?.confirm().await?;
    info!("reserve updated");

    println!("\n-- after update --");
    println!("new resolves: {}", template.resolve(Address::from(cid_v2)));
    Ok(())
}
