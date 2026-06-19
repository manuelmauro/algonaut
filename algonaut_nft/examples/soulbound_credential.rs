//! Issue, hold, and revoke an ARC-71 soulbound (non-transferable) credential.
//!
//! The lifecycle is a typestate: `Soulbound<Issued>` → `Soulbound<Held>` →
//! `Soulbound<Revoked>`. Each transition hands back the transaction that effects
//! it, and the later states carry the `AssetId`, so `claim`/`revoke` are total —
//! no `Option`, no `unwrap`. A wrong-order call (e.g. revoking before claiming)
//! simply does not compile.
//!
//! Run: `cargo run -p algonaut_nft --example soulbound_credential`

use algonaut_core::{Address, AssetId};
use algonaut_crypto::HashDigest;
use algonaut_nft::metadata::arc3;
use algonaut_nft::prelude::*;
use algonaut_transaction::builder::TransactionParams;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let issuer = Address([0x44; 32]); // a smart-contract account, in practice
    let holder = Address([0x55; 32]);

    // 1. Issue: a preconfigured mint (clawback zeroed, freeze + manager = issuer).
    let sbt = Soulbound::new(issuer);
    let cred = arc3::Metadata {
        name: Some("Conference 2026 Attendance".into()),
        ..Default::default()
    };
    let create = sbt
        .mint()
        .unit_name("BADGE")
        .asset_name("Conf2026")
        .arc3(&cred, "ipfs://bafy.../badge.json")?
        .build(&demo_params())?;
    println!("== ARC-71 soulbound credential ==");
    println!("[issue]  txn {}  by {}", create.id()?, create.sender());

    // 2. Claim: after the holder opts in, freeze it in their account → Held.
    let asset_id = AssetId(1001); // known after the create confirms
    let (held, freeze) = Soulbound::new(issuer).claim(asset_id, holder);
    let freeze_txn = freeze.build(&demo_params())?;
    println!(
        "[claim]  asset {} frozen for holder; txn {}",
        held.asset_id(),
        freeze_txn.id()?
    );

    // 3. Revoke: zero the manager → Revoked. Token stays in the wallet.
    let (revoked, update) = held.revoke();
    let revoke_txn = update.build(&demo_params())?;
    println!(
        "[revoke] asset {} manager zeroed; txn {}",
        revoked.asset_id(),
        revoke_txn.id()?
    );

    // The typestate makes illegal lifecycles unrepresentable, e.g. this would
    // not compile:
    //     revoked.revoke();        // no `revoke` on Soulbound<Revoked>
    //     Soulbound::new(issuer).revoke();  // no `revoke` on Soulbound<Issued>
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
