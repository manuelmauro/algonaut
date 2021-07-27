use algonaut::algod::AlgodBuilder;
use algonaut_model::algod::v1::QueryAccountTransactions;
use dotenv::dotenv;
use std::env;
use std::error::Error;
use tokio::test;

#[test]
async fn test_transactions_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = AlgodBuilder::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .build_v1()?;

    let address = env::var("ACCOUNT")?.parse()?;

    let last_round = algod.status().await?.last_round;

    let query = QueryAccountTransactions {
        first_round: Some(last_round),
        from_date: None,
        last_round: Some(last_round),
        max: None,
        to_date: None,
    };

    let res = algod.transactions(&address, &query).await;

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
async fn test_pending_transactions_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = AlgodBuilder::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .build_v1()?;

    println!("{:?}", algod.pending_transactions(0).await);
    assert!(algod.pending_transactions(0).await.is_ok());

    Ok(())
}

#[test]
async fn test_transaction_information_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = AlgodBuilder::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .build_v1()?;

    println!("{:?}", algod.transaction_params().await);
    assert!(algod.transaction_params().await.is_ok());

    Ok(())
}
