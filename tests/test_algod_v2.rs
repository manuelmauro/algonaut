use algorand_rs::Algod;
use dotenv::dotenv;
use std::env;
use std::error::Error;

#[test]
fn test_health_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v2()?;

    assert!(algod.health().is_ok());

    Ok(())
}

#[test]
fn test_versions_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v2()?;

    assert!(algod.versions().ok().is_some());

    Ok(())
}
