use algonaut::crypto::mnemonic;
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

    info!("searching for testwallet");
    let list_response = kmd.list_wallets().await?;
    let wallet_id = match list_response
        .wallets
        .into_iter()
        .find(|wallet| wallet.name == "testwallet")
    {
        Some(wallet) => wallet.id,
        None => return Err("Wallet not found".into()),
    };

    info!("getting wallet handle");
    let init_response = kmd.init_wallet_handle(&wallet_id, "testpassword").await?;
    let wallet_handle_token = init_response.wallet_handle_token;

    info!("exporting wallet");
    let export_response = kmd
        .export_master_derivation_key(&wallet_handle_token, "testpassword")
        .await?;
    let mdk = export_response.master_derivation_key;

    // String representation of the mdk, keep in safe place and don't share it
    let string_to_save = mnemonic::from_key(&mdk.0)?;

    info!("backup phrase: {}", string_to_save);

    Ok(())
}
