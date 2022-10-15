use algonaut::algod::v2::Algod;
use algonaut_core::Round;
use algonaut_model::algod::v2::KeyRegistration;
use dotenv::dotenv;
use std::env;
use std::error::Error;
use tokio::test;

#[test]
async fn test_genesis_endpoint() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let res = algod.genesis().await;

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
async fn test_health_endpoint() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let res = algod.health().await;

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
async fn test_metrics_endpoint() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let res = algod.metrics().await;

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
async fn test_account_information_endpoint() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let res = algod
        .account_information(&"4MYUHDWHWXAKA5KA7U5PEN646VYUANBFXVJNONBK3TIMHEMWMD4UBOJBI4".parse()?)
        .await;

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
async fn test_pending_transactions_for_endpoint() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let res = algod
        .pending_transactions_for(
            &"4MYUHDWHWXAKA5KA7U5PEN646VYUANBFXVJNONBK3TIMHEMWMD4UBOJBI4".parse()?,
            0,
        )
        .await;

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
#[ignore]
async fn test_application_information_endpoint() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let res = algod.application_information(0).await;

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
#[ignore]
async fn test_asset_information_endpoint() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let res = algod.asset_information(0).await;

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
#[ignore]
async fn test_block_endpoint() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let last_round = algod.status().await?.last_round;
    let res = algod.block(Round(last_round)).await;

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
#[ignore]
async fn test_start_catchup_endpoint() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let res = algod
        .start_catchup("4420000#Q7T2RRTDIRTYESIXKAAFJYFQWG4A3WRA3JIUZVCJ3F4AQ2G2HZRA")
        .await;

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
#[ignore]
async fn test_abort_catchup_endpoint() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let res = algod
        .abort_catchup("4420000#Q7T2RRTDIRTYESIXKAAFJYFQWG4A3WRA3JIUZVCJ3F4AQ2G2HZRA")
        .await;

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
async fn test_ledger_supply_endpoint() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let res = algod.ledger_supply().await;

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
#[ignore]
async fn test_register_participation_keys_endpoint() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let params = KeyRegistration {
        fee: None,
        key_dilution: None,
        no_wait: None,
        round_last_valid: None,
    };

    let address = env::var("ALICE_ADDRESS")?.parse()?;

    let res = algod.register_participation_keys(&address, &params).await;

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
#[ignore]
async fn test_shutdown_endpoint() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let res = algod.shutdown(0).await;

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
async fn test_status_endpoint() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let res = algod.status().await;

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
async fn test_status_after_round_endpoint() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let node_status = algod.status().await?;

    let res = algod
        .status_after_round(Round(node_status.last_round + 2))
        .await;

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
async fn test_compile_teal_endpoint() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let res = algod
        .compile_teal(
            r#"
int 1
bnz safe
err
safe:
pop
"#
            .as_bytes(),
        )
        .await;

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
async fn test_failure_compiling_teal() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let res = algod.compile_teal("not-a-teal-program".as_bytes()).await;

    println!("{:#?}", res);
    assert!(res.is_err());

    Ok(())
}

#[test]
#[ignore = "TODO"]
async fn test_dryrun_teal_endpoint() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    Ok(())
}

#[test]
#[ignore = "TODO"]
async fn test_broadcast_raw_transaction_endpoint() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let res = algod.broadcast_raw_transaction(&[0; 32]).await;

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
async fn test_transaction_params_endpoint() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let res = algod.transaction_params().await;

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
async fn test_pending_transactions_endpoint() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let res = algod.pending_transactions(0).await;

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
#[ignore]
async fn test_pending_transaction_with_id_endpoint() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let res = algod.pending_transaction_with_id("").await;

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
async fn test_versions_endpoint() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let res = algod.versions().await;

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}
