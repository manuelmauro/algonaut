use algorand_rs::account::Account;
use algorand_rs::models::Ed25519PublicKey;
use algorand_rs::transaction::{BaseTransaction, Payment, Transaction, TransactionType};
use algorand_rs::{Address, HashDigest, Kmd, MasterDerivationKey, MicroAlgos, Round};
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
        "testwalletasda",
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

    let res = kmd.init_wallet_handle("a4814294b8ae9829943572146053565e", "testpassword");

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
fn test_release_wallet_handle_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let handle = kmd.init_wallet_handle("a4814294b8ae9829943572146053565e", "testpassword");
    let res = kmd.release_wallet_handle(handle.unwrap().wallet_handle_token.as_ref());

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
fn test_renew_wallet_handle_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let handle = kmd.init_wallet_handle("a4814294b8ae9829943572146053565e", "testpassword");
    let res = kmd.renew_wallet_handle(handle.unwrap().wallet_handle_token.as_ref());

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
fn test_rename_wallet_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let res = kmd.rename_wallet(
        "a4814294b8ae9829943572146053565e",
        "testpassword",
        "newtestwallet",
    );

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
fn test_get_wallet_info_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let handle = kmd.init_wallet_handle("a4814294b8ae9829943572146053565e", "testpassword");
    let res = kmd.get_wallet_info(handle.unwrap().wallet_handle_token.as_ref());

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
fn test_export_wallet_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let handle = kmd.init_wallet_handle("a4814294b8ae9829943572146053565e", "testpassword");
    let res = kmd
        .export_master_derivation_key(handle.unwrap().wallet_handle_token.as_ref(), "testpassword");

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
fn test_import_key_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let handle = kmd.init_wallet_handle("a4814294b8ae9829943572146053565e", "testpassword");
    let res = kmd.import_key(handle.unwrap().wallet_handle_token.as_ref(), [0; 32]);

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
fn test_export_key_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let handle = kmd.init_wallet_handle("a4814294b8ae9829943572146053565e", "testpassword");
    let res = kmd.export_key(
        handle.unwrap().wallet_handle_token.as_ref(),
        "testpassword",
        "XEELY6SJ5VTIK5S7W4C66XQC4P43FPMUX34HUQJJHL3WVJHRE6LCYTUWI4",
    );

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
fn test_generate_key_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let handle = kmd.init_wallet_handle("a4814294b8ae9829943572146053565e", "testpassword");
    let res = kmd.generate_key(handle.unwrap().wallet_handle_token.as_ref());

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
fn test_delete_key_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let handle = kmd.init_wallet_handle("a4814294b8ae9829943572146053565e", "testpassword");
    let res = kmd.delete_key(
        handle.unwrap().wallet_handle_token.as_ref(),
        "testpassword",
        "XEELY6SJ5VTIK5S7W4C66XQC4P43FPMUX34HUQJJHL3WVJHRE6LCYTUWI4",
    );

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
fn test_list_keys_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let handle = kmd.init_wallet_handle("a4814294b8ae9829943572146053565e", "testpassword");
    let res = kmd.list_keys(handle.unwrap().wallet_handle_token.as_ref());

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
fn test_sign_transaction_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let account = Account::generate();

    let fee = MicroAlgos(1000);
    let amount = MicroAlgos(20000);
    let first_round = Round(642_715);
    let last_round = first_round + 1000;

    let base = BaseTransaction {
        sender: account.address(),
        first_valid: first_round,
        last_valid: last_round,
        note: Vec::new(),
        genesis_id: "".to_string(),
        genesis_hash: HashDigest([0; 32]),
    };

    let payment = Payment {
        amount,
        receiver: Address::from_string(
            "4MYUHDWHWXAKA5KA7U5PEN646VYUANBFXVJNONBK3TIMHEMWMD4UBOJBI4",
        )?,
        close_remainder_to: None,
    };

    let transaction = Transaction::new(base, fee, TransactionType::Payment(payment))?;

    let handle = kmd.init_wallet_handle("a4814294b8ae9829943572146053565e", "testpassword");

    let res = kmd.sign_transaction(
        handle.unwrap().wallet_handle_token.as_ref(),
        "testpassword",
        &transaction,
    );

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
fn test_list_multisig_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let handle = kmd.init_wallet_handle("a4814294b8ae9829943572146053565e", "testpassword");
    let res = kmd.list_multisig(handle.unwrap().wallet_handle_token.as_ref());

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
fn test_import_multisig_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let version = 1;
    let threshold = 1;
    let pks = [Ed25519PublicKey([0; 32]), Ed25519PublicKey([1; 32])];

    let handle = kmd.init_wallet_handle("a4814294b8ae9829943572146053565e", "testpassword");
    let res = kmd.import_multisig(
        handle.unwrap().wallet_handle_token.as_ref(),
        version,
        threshold,
        &pks,
    );

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
fn test_export_multisig_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let handle = kmd.init_wallet_handle("a4814294b8ae9829943572146053565e", "testpassword");
    let res = kmd.export_multisig(
        handle.unwrap().wallet_handle_token.as_ref(),
        "HIY63SJOQ44P7YMGZLBZJHJ2TVDG5MD4W5PREAQVTTDQMBLPJ2V2EO3PT4",
    );

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
fn test_delete_multisig_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let handle = kmd.init_wallet_handle("a4814294b8ae9829943572146053565e", "testpassword");
    let res = kmd.delete_multisig(
        handle.unwrap().wallet_handle_token.as_ref(),
        "testpassword",
        "HIY63SJOQ44P7YMGZLBZJHJ2TVDG5MD4W5PREAQVTTDQMBLPJ2V2EO3PT4",
    );

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
fn test_sign_multisig_transaction_endpoint() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let account = Account::generate();

    let fee = MicroAlgos(1000);
    let amount = MicroAlgos(20000);
    let first_round = Round(642_715);
    let last_round = first_round + 1000;

    let base = BaseTransaction {
        sender: account.address(),
        first_valid: first_round,
        last_valid: last_round,
        note: Vec::new(),
        genesis_id: "".to_string(),
        genesis_hash: HashDigest([0; 32]),
    };

    let payment = Payment {
        amount,
        receiver: Address::from_string(
            "4MYUHDWHWXAKA5KA7U5PEN646VYUANBFXVJNONBK3TIMHEMWMD4UBOJBI4",
        )?,
        close_remainder_to: None,
    };

    let transaction = Transaction::new(base, fee, TransactionType::Payment(payment))?;
    let pk = Ed25519PublicKey([0; 32]);
    let partial_multisig = None;

    let handle = kmd.init_wallet_handle("a4814294b8ae9829943572146053565e", "testpassword");
    let res = kmd.sign_multisig_transaction(
        handle.unwrap().wallet_handle_token.as_ref(),
        "testpassword",
        &transaction,
        pk,
        partial_multisig,
    );

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}
