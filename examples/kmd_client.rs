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

    info!("creating wallet");
    let create_wallet_response = kmd
        .create_wallet(
            "testwallet",
            "testpassword",
            "sqlite",
            MasterDerivationKey([0; 32]),
        )
        .await?;
    let wallet_id = create_wallet_response.wallet.id;

    info!("initializing handle to the wallet");
    let init_response = kmd.init_wallet_handle(&wallet_id, "testpassword").await?;
    let wallet_handle_token = init_response.wallet_handle_token;

    info!("generating key");
    let gen_response = kmd.generate_key(&wallet_handle_token).await?;
    info!("generated address: {}", gen_response.address);

    Ok(())
}
