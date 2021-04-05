use algonaut_client::indexer::v2::message::{
    QueryAccount, QueryAccountTransaction, QueryApplications, QueryAssetTransaction, QueryAssets,
    QueryBalances, QueryTransaction, Role,
};
use algonaut_client::{Algod, Indexer};
use algonaut_core::Round;
use dotenv::dotenv;
use std::env;
use std::error::Error;

#[test]
fn test_health_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let indexer = Indexer::new()
        .bind(env::var("INDEXER_URL")?.as_ref())
        .client_v2()?;

    let res = indexer.health();

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
fn test_accounts_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let indexer = Indexer::new()
        .bind(env::var("INDEXER_URL")?.as_ref())
        .client_v2()?;

    let query = QueryAccount {
        application_id: None,
        asset_id: None,
        auth_addr: None,
        currency_greater_than: None,
        currency_less_than: None,
        limit: None,
        next: None,
        round: None,
    };

    let res = indexer.accounts(&query);

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
fn test_account_info_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let indexer = Indexer::new()
        .bind(env::var("INDEXER_URL")?.as_ref())
        .client_v2()?;

    let res = indexer.account_info(
        "WADYBW6UZZOWJLKPUWJ5EYXTUOFA5KVYKGUPSBUWXXSXOMMCLI7OG6J7WE",
        None,
    );

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
fn test_account_transactions_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let indexer = Indexer::new()
        .bind(env::var("INDEXER_URL")?.as_ref())
        .client_v2()?;

    let query = QueryAccountTransaction {
        after_time: None,
        asset_id: None,
        before_time: None,
        currency_greater_than: None,
        currency_less_than: None,
        limit: None,
        max_round: None,
        min_round: None,
        next: None,
        note_prefix: None,
        rekey_to: None,
        round: None,
        sig_type: None,
        tx_type: None,
        txid: None,
    };

    let res = indexer.account_transactions(
        "AJY36ONODGCVHZSJR4B7LZ4C6BFLBQIDIGHLNKD4WVTWTXT6XPV7RSIHNY",
        &query,
    );

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
fn test_applications_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let indexer = Indexer::new()
        .bind(env::var("INDEXER_URL")?.as_ref())
        .client_v2()?;

    let query = QueryApplications {
        application_id: None,
        limit: None,
        next: None,
    };

    let res = indexer.applications(&query);

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
#[ignore]
fn test_applications_info_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let indexer = Indexer::new()
        .bind(env::var("INDEXER_URL")?.as_ref())
        .client_v2()?;

    let res = indexer.application_info("");

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
fn test_assets_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let indexer = Indexer::new()
        .bind(env::var("INDEXER_URL")?.as_ref())
        .client_v2()?;

    let query = QueryAssets {
        asset_id: None,
        creator: None,
        limit: None,
        name: None,
        next: None,
        unit: None,
    };

    let res = indexer.assets(&query);

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
#[ignore]
fn test_assets_info_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let indexer = Indexer::new()
        .bind(env::var("INDEXER_URL")?.as_ref())
        .client_v2()?;

    let res = indexer.assets_info("");

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
#[ignore]
fn test_asset_balances_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let indexer = Indexer::new()
        .bind(env::var("INDEXER_URL")?.as_ref())
        .client_v2()?;

    let query = QueryBalances {
        currency_greater_than: None,
        currency_less_than: None,
        limit: None,
        next: None,
        round: None,
    };

    let res = indexer.asset_balances(
        "AJY36ONODGCVHZSJR4B7LZ4C6BFLBQIDIGHLNKD4WVTWTXT6XPV7RSIHNY",
        &query,
    );

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
#[ignore]
fn test_asset_transactions_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let indexer = Indexer::new()
        .bind(env::var("INDEXER_URL")?.as_ref())
        .client_v2()?;

    let query = QueryAssetTransaction {
        address: None,
        address_role: Some(Role::Sender),
        after_time: None,
        before_time: None,
        currency_greater_than: None,
        currency_less_than: None,
        exclude_close_to: None,
        limit: None,
        max_round: None,
        min_round: None,
        next: None,
        note_prefix: None,
        rekey_to: None,
        round: None,
        sig_type: None,
        tx_type: None,
        txid: None,
    };

    let res = indexer.asset_transactions(
        "AJY36ONODGCVHZSJR4B7LZ4C6BFLBQIDIGHLNKD4WVTWTXT6XPV7RSIHNY",
        &query,
    );

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
fn test_block_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v2()?;

    let indexer = Indexer::new()
        .bind(env::var("INDEXER_URL")?.as_ref())
        .client_v2()?;

    let last_round = algod.status()?.last_round;
    let res = indexer.block(Round(last_round - 1));

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
#[ignore]
fn test_transactions_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let indexer = Indexer::new()
        .bind(env::var("INDEXER_URL")?.as_ref())
        .client_v2()?;

    let query = QueryTransaction {
        address: None,
        address_role: None,
        after_time: None,
        application_id: None,
        asset_id: None,
        before_time: None,
        currency_greater_than: None,
        currency_less_than: None,
        exclude_close_to: None,
        limit: None,
        max_round: None,
        min_round: None,
        next: None,
        note_prefix: None,
        rekey_to: None,
        round: None,
        sig_type: None,
        tx_type: None,
        txid: None,
    };

    let res = indexer.transactions(&query);

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
#[ignore]
fn test_transaction_info_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let indexer = Indexer::new()
        .bind(env::var("INDEXER_URL")?.as_ref())
        .client_v2()?;

    let res = indexer.transaction_info("");

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}
