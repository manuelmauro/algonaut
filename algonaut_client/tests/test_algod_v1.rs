use algonaut_client::Algod;
use algonaut_core::Round;
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
        .client_v1()?;

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
        .client_v1()?;

    assert!(algod.versions().is_ok());

    Ok(())
}

#[test]
fn test_status_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v1()?;

    assert!(algod.status().is_ok());

    Ok(())
}

#[test]
fn test_status_after_block_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v1()?;

    assert!(algod.status_after_block(Round(0)).is_ok());

    Ok(())
}

#[test]
fn test_block_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v1()?;

    let node_status = algod.status();
    println!("{:#?}", node_status);
    assert!(node_status.is_ok());

    let res = algod.block(node_status.unwrap().last_round);
    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
fn test_ledger_supply_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v1()?;

    assert!(algod.ledger_supply().is_ok());

    Ok(())
}

#[test]
fn test_account_information_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v1()?;

    assert!(algod
        .account_information("4MYUHDWHWXAKA5KA7U5PEN646VYUANBFXVJNONBK3TIMHEMWMD4UBOJBI4")
        .ok()
        .is_some());

    Ok(())
}
