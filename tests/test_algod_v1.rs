use algonaut::algod::AlgodBuilder;
use algonaut_core::Round;
use dotenv::dotenv;
use std::env;
use std::error::Error;
use tokio::test;

#[test]
async fn test_health_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = AlgodBuilder::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .build_v1()?;

    assert!(algod.health().await.is_ok());

    Ok(())
}

#[test]
async fn test_versions_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = AlgodBuilder::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .build_v1()?;

    assert!(algod.versions().await.is_ok());

    Ok(())
}

#[test]
async fn test_status_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = AlgodBuilder::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .build_v1()?;

    assert!(algod.status().await.is_ok());

    Ok(())
}

#[test]
async fn test_status_after_block_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = AlgodBuilder::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .build_v1()?;

    assert!(algod.status_after_block(Round(0)).await.is_ok());

    Ok(())
}

#[test]
async fn test_block_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = AlgodBuilder::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .build_v1()?;

    let node_status = algod.status().await;
    println!("{:#?}", node_status);
    assert!(node_status.is_ok());

    let res = algod.block(node_status.unwrap().last_round).await;
    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
async fn test_ledger_supply_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = AlgodBuilder::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .build_v1()?;

    assert!(algod.ledger_supply().await.is_ok());

    Ok(())
}

#[test]
async fn test_account_information_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = AlgodBuilder::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .build_v1()?;

    assert!(algod
        .account_information("4MYUHDWHWXAKA5KA7U5PEN646VYUANBFXVJNONBK3TIMHEMWMD4UBOJBI4")
        .await
        .ok()
        .is_some());

    Ok(())
}
