use algonaut::crypto::MasterDerivationKey;
use algonaut::Kmd;
use dotenv::dotenv;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let create_wallet_response = kmd
        .create_wallet(
            "testwallet",
            "testpassword",
            "sqlite",
            MasterDerivationKey([0; 32]),
        )
        .await?;
    let wallet_id = create_wallet_response.wallet.id;

    let init_response = kmd.init_wallet_handle(&wallet_id, "testpassword").await?;
    let wallet_handle_token = init_response.wallet_handle_token;

    let gen_response = kmd.generate_key(&wallet_handle_token).await?;
    println!("Generated address: {}", gen_response.address);

    Ok(())
}
