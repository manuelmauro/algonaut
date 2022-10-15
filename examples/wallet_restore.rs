use algonaut::crypto::mnemonic;
use algonaut::crypto::MasterDerivationKey;
use algonaut::kmd::v1::Kmd;
use dotenv::dotenv;
use std::env;
use std::error::Error;
#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    env_logger::init();

    info!("creating kmd client");
    let kmd = Kmd::new(&env::var("KMD_URL")?, &env::var("KMD_TOKEN")?)?;

    info!("creating backup phrase");
    let backup_phrase = "fire enlist diesel stamp nuclear chunk student stumble call snow flock brush example slab guide choice option recall south kangaroo hundred matrix school above zero";
    let key_bytes = mnemonic::to_key(backup_phrase)?;
    let mdk = MasterDerivationKey(key_bytes);

    info!("creating wallet");
    let create_wallet_response = kmd
        .create_wallet("testwallet", "testpassword", "sqlite", mdk)
        .await?;
    let wallet = create_wallet_response.wallet;

    info!("created wallet {} with ID: {}", wallet.name, wallet.id);

    Ok(())
}
