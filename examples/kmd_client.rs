use std::error::Error;

use algorand_rs::{Kmd, MasterDerivationKey};

fn main() -> Result<(), Box<dyn Error>> {
    let kmd_address = "http://localhost:4002";
    let kmd_token = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    let kmd = Kmd::new().bind(kmd_address).auth(kmd_token).client_v1()?;

    let create_wallet_response = kmd.create_wallet(
        "testwallet",
        "testpassword",
        "sqlite",
        MasterDerivationKey([0; 32]),
    )?;
    let wallet_id = create_wallet_response.wallet.id;

    let init_response = kmd.init_wallet_handle(&wallet_id, "testpassword")?;
    let wallet_handle_token = init_response.wallet_handle_token;

    let gen_response = kmd.generate_key(&wallet_handle_token)?;
    println!("Generated address: {}", gen_response.address);

    Ok(())
}
