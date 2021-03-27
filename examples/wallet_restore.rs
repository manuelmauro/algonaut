use algonaut::crypto::address::MasterDerivationKey;
use algonaut::crypto::mnemonic;
use algonaut::Kmd;
use dotenv::dotenv;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let backup_phrase = "fire enlist diesel stamp nuclear chunk student stumble call snow flock brush example slab guide choice option recall south kangaroo hundred matrix school above zero";
    let key_bytes = mnemonic::to_key(backup_phrase)?;
    let mdk = MasterDerivationKey(key_bytes);

    let create_wallet_response = kmd.create_wallet("testwallet", "testpassword", "sqlite", mdk)?;
    let wallet = create_wallet_response.wallet;

    println!("Created wallet {} with ID: {}", wallet.name, wallet.id);

    Ok(())
}
