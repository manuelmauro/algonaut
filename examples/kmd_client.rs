use std::error::Error;

use algorust::{kmd, MasterDerivationKey};

fn main() -> Result<(), Box<dyn Error>> {
    let kmd_address = "http://localhost:4002";
    let kmd_token = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    let kmd_client = kmd::Client::new(kmd_address, kmd_token);

    let create_wallet_response = kmd_client.create_wallet(
        "testwallet",
        "testpassword",
        "sqlite",
        MasterDerivationKey([0; 32]),
    )?;
    let wallet_id = create_wallet_response.wallet.id;

    let init_response = kmd_client.init_wallet_handle(&wallet_id, "testpassword")?;
    let wallet_handle_token = init_response.wallet_handle_token;

    let gen_response = kmd_client.generate_key(&wallet_handle_token)?;
    println!("Generated address: {}", gen_response.address);

    Ok(())
}
