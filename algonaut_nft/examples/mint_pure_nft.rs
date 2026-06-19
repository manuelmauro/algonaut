//! Mint a pure (1-of-1) NFT with ARC-3 off-chain JSON metadata.
//!
//! The point of this example is the *minting ergonomics*: you describe the token
//! and its metadata once, and `NftMint` computes the ARC-3 metadata hash and
//! applies the `#arc3` URL convention for you. You never touch a hash function.
//!
//! Run: `cargo run -p algonaut_nft --example mint_pure_nft`

use algonaut_core::Address;
use algonaut_crypto::HashDigest;
use algonaut_nft::metadata::integrity;
use algonaut_nft::prelude::*;
use algonaut_transaction::builder::TransactionParams;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let creator = Address([0x11; 32]);

    // 1. Describe the NFT's off-chain metadata.
    let meta = Arc3Metadata {
        name: Some("Cosmic Cube #1".into()),
        description: Some("A one-of-a-kind cube.".into()),
        image: Some("ipfs://bafybeigdyrhz.../1.png".into()),
        image_mimetype: Some("image/png".into()),
        ..Default::default()
    };
    let metadata_url = "ipfs://bafybeigdyrhz.../1.json";

    // 2. Mint. `.arc3(..)` computes `am` and appends `#arc3` internally — the
    //    caller does not hash anything or know the URL-fragment rule.
    let txn = NftMint::pure(creator)
        .unit_name("CUBE")
        .asset_name("Cosmic Cube #1")
        .arc3(&meta, metadata_url)?
        .build(&demo_params())?;

    // For display only: the hash the builder pinned on-chain.
    let am = integrity::metadata_hash(&meta.to_json()?, None);

    println!("== Pure NFT mint ==");
    println!("creator : {creator}");
    println!("unit/name: CUBE / Cosmic Cube #1");
    println!("asset url: {metadata_url}#arc3");
    println!("am (hex) : {}", hex(&am));
    println!("txn id   : {}", txn.id()?);
    println!("sender   : {}", txn.sender());
    println!("\nPublish this exact JSON at the asset url:");
    println!("{}", serde_json::to_string_pretty(&meta)?);
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Stand-in suggested params so the example builds a real transaction offline.
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
