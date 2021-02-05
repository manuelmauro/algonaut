use std::error::Error;

use algorand_rs::{Kmd, mnemonic, MasterDerivationKey};

fn main() -> Result<(), Box<dyn Error>> {
    let kmd_address = "http://localhost:4002";
    let kmd_token = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    let kmd = Kmd::new().bind(kmd_address).auth(kmd_token).client_v1()?;

    let backup_phrase = "fire enlist diesel stamp nuclear chunk student stumble call snow flock brush example slab guide choice option recall south kangaroo hundred matrix school above zero";
    let key_bytes = mnemonic::to_key(backup_phrase)?;
    let mdk = MasterDerivationKey(key_bytes);

    let create_wallet_response =
        kmd.create_wallet("testwallet", "testpassword", "sqlite", mdk)?;
    let wallet = create_wallet_response.wallet;

    println!("Created wallet {} with ID: {}", wallet.name, wallet.id);

    Ok(())
}
