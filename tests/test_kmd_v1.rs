use algorand_rs::{Kmd, MasterDerivationKey};
use dotenv::dotenv;
use std::env;
use std::error::Error;

#[test]
fn test_versions_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let versions = kmd.versions();
    println!("{:#?}", versions);
    assert!(versions.is_ok());

    Ok(())
}

#[test]
fn test_list_wallets_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let wallets = kmd.list_wallets();
    println!("{:#?}", wallets);
    assert!(wallets.is_ok());

    Ok(())
}

#[test]
fn test_create_wallet_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;
    
    let wallet = kmd.create_wallet(
        "testwallet",
        "testpassword",
        "sqlite",
        MasterDerivationKey([0; 32]),
    );
    println!("{:#?}", wallet);
    assert!(wallet.is_ok());

    Ok(())
}

#[test]
fn test_init_wallet_handle_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;
    
    let wallet = kmd.init_wallet_handle(
        "testwallet",
        "testpassword",
    );
    
    println!("{:#?}", wallet);
    assert!(wallet.is_ok());

    Ok(())
}
