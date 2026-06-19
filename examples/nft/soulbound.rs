//! Full ARC-71 soulbound lifecycle against a live algod: issue → hold → revoke,
//! across an issuer (Alice) and a freshly funded holder.
//!
//! The lifecycle is a typestate (`Soulbound<Issued>` → `Held` → `Revoked`); the
//! later states carry the real `AssetId`, so `claim`/`revoke` are total.
//!
//! Needs a sandbox: `ALGOD_URL`, `ALGOD_TOKEN`, `ALICE_MNEMONIC`.
//! Run: `cargo run --features nft --example nft_soulbound`

use algonaut::Algod;
use algonaut::core::{AssetId, MicroAlgos};
use algonaut::nft::metadata::arc3;
use algonaut::nft::prelude::*;
use algonaut::transaction::account::Account;
use algonaut::transaction::{AcceptAsset, Pay, TransferAsset};
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
    let alice = Account::from_mnemonic(&env::var("ALICE_MNEMONIC")?)?; // the issuer
    let holder = Account::generate();
    let issuer = alice.address();

    // 1. Issue: a preconfigured soulbound mint (clawback zeroed, freeze + manager
    //    = issuer). The asset id comes back from the network.
    let sbt = Soulbound::new(issuer);
    let cred = arc3::Metadata {
        name: Some("Conference 2026 Attendance".into()),
        ..Default::default()
    };
    let params = algod.suggested_params().await?;
    let mint = sbt
        .mint()
        .unit_name("BADGE")
        .asset_name("Conf2026")
        .arc3(&cred, "ipfs://bafy.../badge.json")?
        .build(&params)?;
    let asset_id = AssetId(
        algod
            .submit(&alice.sign(mint)?)
            .await?
            .confirm()
            .await?
            .asset_index
            .expect("asset index"),
    );
    println!("== ARC-71 soulbound credential ==");
    println!("[issue]  asset {asset_id} issued by {issuer}");

    // 2. Fund the holder so it can opt in and cover its min-balance.
    let params = algod.suggested_params().await?;
    let fund = Pay::new(issuer, holder.address(), MicroAlgos(500_000)).build(&params)?;
    algod.submit(&alice.sign(fund)?).await?.confirm().await?;

    // 3. Holder opts in, then the issuer transfers the single unit to it.
    let params = algod.suggested_params().await?;
    let optin = AcceptAsset::new(holder.address(), asset_id).build(&params)?;
    algod.submit(&holder.sign(optin)?).await?.confirm().await?;

    let params = algod.suggested_params().await?;
    let xfer = TransferAsset::new(issuer, asset_id, 1, holder.address()).build(&params)?;
    algod.submit(&alice.sign(xfer)?).await?.confirm().await?;
    println!(
        "[hold]   asset {asset_id} delivered to {}",
        holder.address()
    );

    // 4. Claim: freeze it in the holder's account → Held (typestate carries the id).
    let (held, freeze) = sbt.claim(asset_id, holder.address());
    let params = algod.suggested_params().await?;
    algod
        .submit(&alice.sign(freeze.build(&params)?)?)
        .await?
        .confirm()
        .await?;
    info!("frozen for holder; now Held");

    // 5. Revoke: zero the manager → Revoked. The token stays in the wallet.
    let (revoked, update) = held.revoke();
    let params = algod.suggested_params().await?;
    algod
        .submit(&alice.sign(update.build(&params)?)?)
        .await?
        .confirm()
        .await?;
    println!(
        "[revoke] asset {} manager zeroed (credential revoked)",
        revoked.asset_id()
    );
    Ok(())
}
