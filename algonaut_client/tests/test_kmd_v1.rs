use algonaut_client::Kmd;
use algonaut_crypto::{Ed25519PublicKey, MasterDerivationKey};
use dotenv::dotenv;
use rand::{distributions::Alphanumeric, Rng};
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
fn test_create_wallet_and_obtain_handle() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let wallet_name: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

    let wallet = kmd.create_wallet(
        wallet_name.as_ref(),
        "testpassword",
        "sqlite",
        MasterDerivationKey([0; 32]),
    );

    println!("{:#?}", wallet);
    assert!(wallet.is_ok());

    let wallet = wallet.unwrap();

    let id = wallet.wallet.id.as_ref();
    let handle = kmd.init_wallet_handle(id, "testpassword");

    println!("{:#?}", handle);
    assert!(handle.is_ok());

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

    let wallet_name: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

    let wallet = kmd.create_wallet(
        wallet_name.as_ref(),
        "testpassword",
        "sqlite",
        MasterDerivationKey([0; 32]),
    );
    println!("{:#?}", wallet);
    assert!(wallet.is_ok());

    let wallet = wallet.unwrap();

    let id = wallet.wallet.id.as_ref();
    let handle = kmd.init_wallet_handle(id, "testpassword");

    println!("{:#?}", handle);
    assert!(handle.is_ok());

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

    let wallet_name: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

    let wallet = kmd.create_wallet(
        wallet_name.as_ref(),
        "testpassword",
        "sqlite",
        MasterDerivationKey([0; 32]),
    );
    println!("{:#?}", wallet);
    assert!(wallet.is_ok());

    let wallet = wallet.unwrap();

    let id = wallet.wallet.id.as_ref();
    let handle = kmd.init_wallet_handle(id, "testpassword");

    println!("{:#?}", handle);
    assert!(handle.is_ok());

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

    let wallet_name: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

    let wallet = kmd.create_wallet(
        wallet_name.as_ref(),
        "testpassword",
        "sqlite",
        MasterDerivationKey([0; 32]),
    );
    println!("{:#?}", wallet);
    assert!(wallet.is_ok());

    let new_name: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

    let res = kmd.rename_wallet(
        wallet.unwrap().wallet.id.as_ref(),
        "testpassword",
        new_name.as_ref(),
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

    let wallet_name: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

    let wallet = kmd.create_wallet(
        wallet_name.as_ref(),
        "testpassword",
        "sqlite",
        MasterDerivationKey([0; 32]),
    );
    println!("{:#?}", wallet);
    assert!(wallet.is_ok());

    let wallet = wallet.unwrap();

    let id = wallet.wallet.id.as_ref();
    let handle = kmd.init_wallet_handle(id, "testpassword");

    println!("{:#?}", handle);
    assert!(handle.is_ok());

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

    let wallet_name: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

    let wallet = kmd.create_wallet(
        wallet_name.as_ref(),
        "testpassword",
        "sqlite",
        MasterDerivationKey([0; 32]),
    );
    println!("{:#?}", wallet);
    assert!(wallet.is_ok());

    let wallet = wallet.unwrap();

    let id = wallet.wallet.id.as_ref();
    let handle = kmd.init_wallet_handle(id, "testpassword");

    println!("{:#?}", handle);
    assert!(handle.is_ok());

    let res = kmd
        .export_master_derivation_key(handle.unwrap().wallet_handle_token.as_ref(), "testpassword");

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
fn test_import_export_key() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let wallet_name: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

    let wallet = kmd.create_wallet(
        wallet_name.as_ref(),
        "testpassword",
        "sqlite",
        MasterDerivationKey([0; 32]),
    );
    println!("{:#?}", wallet);
    assert!(wallet.is_ok());

    let wallet = wallet.unwrap();

    let id = wallet.wallet.id.as_ref();
    let handle = kmd.init_wallet_handle(id, "testpassword");

    println!("{:#?}", handle);
    assert!(handle.is_ok());

    let handle = handle.unwrap();

    let key = kmd.import_key(handle.wallet_handle_token.as_ref(), [0; 32]);

    println!("{:#?}", key);
    assert!(key.is_ok());

    let key = key.unwrap();

    let res = kmd.export_key(
        handle.wallet_handle_token.as_ref(),
        "testpassword",
        key.address.as_ref(),
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

    let wallet_name: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

    let wallet = kmd.create_wallet(
        wallet_name.as_ref(),
        "testpassword",
        "sqlite",
        MasterDerivationKey([0; 32]),
    );
    println!("{:#?}", wallet);
    assert!(wallet.is_ok());

    let wallet = wallet.unwrap();

    let id = wallet.wallet.id.as_ref();
    let handle = kmd.init_wallet_handle(id, "testpassword");

    println!("{:#?}", handle);
    assert!(handle.is_ok());

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

    let wallet_name: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

    let wallet = kmd.create_wallet(
        wallet_name.as_ref(),
        "testpassword",
        "sqlite",
        MasterDerivationKey([0; 32]),
    );
    println!("{:#?}", wallet);
    assert!(wallet.is_ok());

    let wallet = wallet.unwrap();

    let id = wallet.wallet.id.as_ref();
    let handle = kmd.init_wallet_handle(id, "testpassword");

    println!("{:#?}", handle);
    assert!(handle.is_ok());

    let handle = handle.unwrap();

    let key = kmd.generate_key(handle.wallet_handle_token.as_ref());

    println!("{:#?}", key);
    assert!(key.is_ok());

    let key = key.unwrap();

    let res = kmd.delete_key(
        handle.wallet_handle_token.as_ref(),
        "testpassword",
        key.address.as_ref(),
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

    let wallet_name: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

    let wallet = kmd.create_wallet(
        wallet_name.as_ref(),
        "testpassword",
        "sqlite",
        MasterDerivationKey([0; 32]),
    );
    println!("{:#?}", wallet);
    assert!(wallet.is_ok());

    let wallet = wallet.unwrap();

    let id = wallet.wallet.id.as_ref();
    let handle = kmd.init_wallet_handle(id, "testpassword");

    println!("{:#?}", handle);
    assert!(handle.is_ok());

    let handle = handle.unwrap();

    let res = kmd.generate_key(handle.wallet_handle_token.as_ref());

    println!("{:#?}", res);
    assert!(res.is_ok());

    let res = kmd.list_keys(handle.wallet_handle_token.as_ref());

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
fn test_list_keys_of_empty_wallet() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let wallet_name: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

    let wallet = kmd.create_wallet(
        wallet_name.as_ref(),
        "testpassword",
        "sqlite",
        MasterDerivationKey([0; 32]),
    );
    println!("{:#?}", wallet);
    assert!(wallet.is_ok());

    let wallet = wallet.unwrap();

    let id = wallet.wallet.id.as_ref();
    let handle = kmd.init_wallet_handle(id, "testpassword");

    println!("{:#?}", handle);
    assert!(handle.is_ok());

    let res = kmd.list_keys(handle.unwrap().wallet_handle_token.as_ref());

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

    let wallet_name: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

    let wallet = kmd.create_wallet(
        wallet_name.as_ref(),
        "testpassword",
        "sqlite",
        MasterDerivationKey([0; 32]),
    );
    println!("{:#?}", wallet);
    assert!(wallet.is_ok());

    let wallet = wallet.unwrap();

    let id = wallet.wallet.id.as_ref();
    let handle = kmd.init_wallet_handle(id, "testpassword");

    println!("{:#?}", handle);
    assert!(handle.is_ok());

    let res = kmd.list_multisig(handle.unwrap().wallet_handle_token.as_ref());

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}

#[test]
fn test_import_export_multisig() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    let version = 1;
    let threshold = 1;
    let pks = [Ed25519PublicKey([0; 32]), Ed25519PublicKey([1; 32])];

    let wallet_name: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

    let wallet = kmd.create_wallet(
        wallet_name.as_ref(),
        "testpassword",
        "sqlite",
        MasterDerivationKey([0; 32]),
    );
    println!("{:#?}", wallet);
    assert!(wallet.is_ok());

    let wallet = wallet.unwrap();

    let id = wallet.wallet.id.as_ref();
    let handle = kmd.init_wallet_handle(id, "testpassword");

    println!("{:#?}", handle);
    assert!(handle.is_ok());

    let handle = handle.unwrap();

    let multisig = kmd.import_multisig(
        handle.wallet_handle_token.as_ref(),
        version,
        threshold,
        &pks,
    );

    println!("{:#?}", multisig);
    assert!(multisig.is_ok());

    let res = kmd.export_multisig(
        handle.wallet_handle_token.as_ref(),
        multisig.unwrap().address.as_ref(),
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

    let version = 1;
    let threshold = 1;
    let pks = [Ed25519PublicKey([0; 32]), Ed25519PublicKey([1; 32])];

    let wallet_name: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

    let wallet = kmd.create_wallet(
        wallet_name.as_ref(),
        "testpassword",
        "sqlite",
        MasterDerivationKey([0; 32]),
    );
    println!("{:#?}", wallet);
    assert!(wallet.is_ok());

    let wallet = wallet.unwrap();

    let id = wallet.wallet.id.as_ref();
    let handle = kmd.init_wallet_handle(id, "testpassword");

    println!("{:#?}", handle);
    assert!(handle.is_ok());

    let handle = handle.unwrap();

    let multisig = kmd.import_multisig(
        handle.wallet_handle_token.as_ref(),
        version,
        threshold,
        &pks,
    );

    println!("{:#?}", multisig);
    assert!(multisig.is_ok());

    let res = kmd.delete_multisig(
        handle.wallet_handle_token.as_ref(),
        "testpassword",
        multisig.unwrap().address.as_ref(),
    );

    println!("{:#?}", res);
    assert!(res.is_ok());

    Ok(())
}
