use crate::step_defs::integration::world::World;
use algonaut::error::Error;
use algonaut_core::{Address, MultisigAddress, ToMsgPack};
use algonaut_crypto::{Ed25519PublicKey, MasterDerivationKey};
use algonaut_transaction::{account::Account, transaction::TransactionSignature};
use cucumber::{given, then, when};
use rand::Rng;

const NEW_WALLET_NAME: &str = "algonaut-cucumber-test";
const NEW_WALLET_PASSWORD: &str = "algonaut-cucumber-password";

#[when(expr = "I get versions with kmd")]
async fn i_get_versions_with_kmd(w: &mut World) -> Result<(), Error> {
    let kmd = w.kmd.as_ref().expect("kmd not set");
    let resp = kmd.versions().await?;
    w.versions = Some(resp.versions);
    Ok(())
}

#[when(expr = "I create a wallet")]
async fn i_create_a_wallet(w: &mut World) -> Result<(), Error> {
    let kmd = w.kmd.as_ref().expect("kmd not set");
    // Append randomness so reruns against the same kmd don't collide.
    let mut rng = rand::thread_rng();
    let suffix: u32 = rng.r#gen();
    let name = format!("{NEW_WALLET_NAME}-{suffix:08x}");
    let mdk = MasterDerivationKey([0u8; 32]);
    let resp = kmd
        .create_wallet(&name, NEW_WALLET_PASSWORD, "sqlite", mdk)
        .await?;
    w.created_wallet_id = Some(resp.wallet.id);
    w.created_wallet_name = Some(name);
    Ok(())
}

#[then(expr = "the wallet should exist")]
async fn the_wallet_should_exist(w: &mut World) -> Result<(), Error> {
    let kmd = w.kmd.as_ref().expect("kmd not set");
    let wallet_id = w
        .created_wallet_id
        .as_ref()
        .expect("created wallet id not set");
    let wallets = kmd.list_wallets().await?;
    assert!(
        wallets.wallets.iter().any(|wal| &wal.id == wallet_id),
        "freshly-created wallet not found in list"
    );
    Ok(())
}

#[when(expr = "I get the wallet handle")]
async fn i_get_the_wallet_handle(w: &mut World) -> Result<(), Error> {
    // The "Wallet handle" scenario exercises the handle lifecycle without an
    // explicit "I create a wallet" step, and cucumber resets the World per
    // scenario, so there is no created wallet to inherit. Lazily create one.
    if w.created_wallet_id.is_none() {
        i_create_a_wallet(w).await?;
    }

    let kmd = w.kmd.as_ref().expect("kmd not set");
    let wallet_id = w
        .created_wallet_id
        .as_ref()
        .expect("created wallet id not set");
    let resp = kmd
        .init_wallet_handle(wallet_id, NEW_WALLET_PASSWORD)
        .await?;
    w.created_wallet_handle = Some(resp.wallet_handle_token);
    Ok(())
}

#[then(expr = "I can get the master derivation key")]
async fn i_can_get_the_master_derivation_key(w: &mut World) -> Result<(), Error> {
    let kmd = w.kmd.as_ref().expect("kmd not set");
    let handle = w
        .created_wallet_handle
        .as_ref()
        .expect("created wallet handle not set");
    kmd.export_master_derivation_key(handle, NEW_WALLET_PASSWORD)
        .await?;
    Ok(())
}

#[when(expr = "I rename the wallet")]
async fn i_rename_the_wallet(w: &mut World) -> Result<(), Error> {
    let kmd = w.kmd.as_ref().expect("kmd not set");
    let wallet_id = w
        .created_wallet_id
        .as_ref()
        .expect("created wallet id not set");
    let new_name = format!(
        "{}-renamed",
        w.created_wallet_name
            .as_ref()
            .expect("created wallet name not set")
    );
    kmd.rename_wallet(wallet_id, NEW_WALLET_PASSWORD, &new_name)
        .await?;
    w.created_wallet_name = Some(new_name);
    Ok(())
}

#[then(expr = "I can still get the wallet information with the same handle")]
async fn i_can_still_get_the_wallet_information_with_the_same_handle(
    w: &mut World,
) -> Result<(), Error> {
    let kmd = w.kmd.as_ref().expect("kmd not set");
    let handle = w
        .created_wallet_handle
        .as_ref()
        .expect("created wallet handle not set");
    kmd.get_wallet_info(handle).await?;
    Ok(())
}

#[when(expr = "I renew the wallet handle")]
async fn i_renew_the_wallet_handle(w: &mut World) -> Result<(), Error> {
    let kmd = w.kmd.as_ref().expect("kmd not set");
    let handle = w
        .created_wallet_handle
        .as_ref()
        .or(w.handle.as_ref())
        .expect("no wallet handle set");
    kmd.renew_wallet_handle(handle).await?;
    Ok(())
}

#[when(expr = "I release the wallet handle")]
async fn i_release_the_wallet_handle(w: &mut World) -> Result<(), Error> {
    let kmd = w.kmd.as_ref().expect("kmd not set");
    let handle = w
        .created_wallet_handle
        .as_ref()
        .or(w.handle.as_ref())
        .expect("no wallet handle set");
    kmd.release_wallet_handle(handle).await?;
    Ok(())
}

#[then(expr = "the wallet handle should not work")]
async fn the_wallet_handle_should_not_work(w: &mut World) {
    let kmd = w.kmd.as_ref().expect("kmd not set");
    let handle = w
        .created_wallet_handle
        .as_ref()
        .or(w.handle.as_ref())
        .expect("no wallet handle set");
    assert!(
        kmd.get_wallet_info(handle).await.is_err(),
        "released wallet handle still works"
    );
}

#[when(expr = "I generate a key using kmd")]
async fn i_generate_a_key_using_kmd(w: &mut World) -> Result<(), Error> {
    let kmd = w.kmd.as_ref().expect("kmd not set");
    let handle = w.handle.as_ref().expect("wallet handle not set");
    let resp = kmd.generate_key(handle).await?;
    let address: Address = resp
        .address
        .parse()
        .map_err(|e: String| algonaut::Error::Msg(e))?;
    w.generated_kmd_address = Some(address);
    Ok(())
}

#[then(expr = "the key should be in the wallet")]
async fn the_key_should_be_in_the_wallet(w: &mut World) -> Result<(), Error> {
    let kmd = w.kmd.as_ref().expect("kmd not set");
    let handle = w.handle.as_ref().expect("wallet handle not set");
    let addr = w
        .generated_kmd_address
        .as_ref()
        .expect("generated kmd address not set")
        .to_string();
    let keys = kmd.list_keys(handle).await?;
    assert!(
        keys.addresses.contains(&addr),
        "generated key not in wallet"
    );
    Ok(())
}

#[when(expr = "I delete the key")]
async fn i_delete_the_key(w: &mut World) -> Result<(), Error> {
    let kmd = w.kmd.as_ref().expect("kmd not set");
    let handle = w.handle.as_ref().expect("wallet handle not set");
    let password = w.password.as_ref().expect("wallet password not set");
    let addr = w
        .generated_kmd_address
        .as_ref()
        .expect("generated kmd address not set");
    kmd.delete_key(handle, password, addr).await?;
    Ok(())
}

#[then(expr = "the key should not be in the wallet")]
async fn the_key_should_not_be_in_the_wallet(w: &mut World) -> Result<(), Error> {
    let kmd = w.kmd.as_ref().expect("kmd not set");
    let handle = w.handle.as_ref().expect("wallet handle not set");
    let addr = w
        .generated_kmd_address
        .as_ref()
        .expect("generated kmd address not set")
        .to_string();
    let keys = kmd.list_keys(handle).await?;
    assert!(
        !keys.addresses.contains(&addr),
        "deleted key still in wallet"
    );
    Ok(())
}

#[then(expr = "I can get account information")]
async fn i_can_get_account_information(w: &mut World) -> Result<(), Error> {
    let algod = w.algod.as_ref().expect("algod not set").clone();
    let addr = w
        .generated_kmd_address
        .as_ref()
        .expect("generated kmd address not set");
    algod.account(addr).await?;
    Ok(())
}

#[when(expr = "I generate a key")]
async fn i_generate_a_key(w: &mut World) {
    w.generated_account = Some(Account::generate());
}

#[when(expr = "I import the key")]
async fn i_import_the_key(w: &mut World) -> Result<(), Error> {
    let kmd = w.kmd.as_ref().expect("kmd not set");
    let handle = w.handle.as_ref().expect("wallet handle not set");
    let account = w
        .generated_account
        .as_ref()
        .expect("generated account not set");
    kmd.import_key(handle, account.seed()).await?;
    Ok(())
}

#[then(expr = "the private key should be equal to the exported private key")]
async fn the_private_key_should_be_equal_to_the_exported_private_key(
    w: &mut World,
) -> Result<(), Error> {
    let kmd = w.kmd.as_ref().expect("kmd not set");
    let handle = w.handle.as_ref().expect("wallet handle not set");
    let password = w.password.as_ref().expect("wallet password not set");
    let account = w
        .generated_account
        .as_ref()
        .expect("generated account not set");
    let exported = kmd.export_key(handle, password, &account.address()).await?;
    // The first 32 bytes are the seed; compare to what we generated.
    assert_eq!(
        &exported.private_key[0..32],
        &account.seed()[..],
        "exported seed differs from generated seed"
    );
    // Clean up so the wallet doesn't accumulate keys across runs.
    kmd.delete_key(handle, password, &account.address())
        .await
        .ok();
    Ok(())
}

#[when(expr = "I sign the transaction with kmd")]
async fn i_sign_the_transaction_with_kmd(w: &mut World) -> Result<(), Error> {
    let kmd = w.kmd.as_ref().expect("kmd not set");
    let handle = w.handle.as_ref().expect("wallet handle not set");
    let password = w.password.as_ref().expect("wallet password not set");
    let tx = w.tx.as_ref().expect("tx not set");
    let resp = kmd.sign(handle, password, tx).await?;
    w.kmd_signed_tx_bytes = Some(resp.signed_transaction);
    Ok(())
}

#[then(expr = "the signed transaction should equal the kmd signed transaction")]
async fn the_signed_transaction_should_equal_the_kmd_signed_transaction(
    w: &mut World,
) -> Result<(), Error> {
    let signed = w.signed_tx.as_ref().expect("signed tx not set");
    let kmd_bytes = w
        .kmd_signed_tx_bytes
        .as_ref()
        .expect("kmd-signed transaction not set");
    let sdk_bytes = signed.to_msg_pack()?;
    assert_eq!(&sdk_bytes, kmd_bytes, "sdk signed tx != kmd signed tx");
    Ok(())
}

#[given(regex = r#"^multisig addresses "([^"]*)"$"#)]
async fn multisig_addresses(w: &mut World, addresses: String) -> Result<(), Error> {
    let parsed: Vec<Address> = addresses
        .split_whitespace()
        .map(|s| s.parse::<Address>())
        .collect::<Result<_, _>>()
        .map_err(algonaut::Error::Msg)?;
    let msig = MultisigAddress::new(1, 1, &parsed).map_err(algonaut::Error::Msg)?;
    w.multisig = Some(msig);
    Ok(())
}

#[when(expr = "I import the multisig")]
async fn i_import_the_multisig(w: &mut World) -> Result<(), Error> {
    let kmd = w.kmd.as_ref().expect("kmd not set");
    let handle = w.handle.as_ref().expect("wallet handle not set");
    let msig = w.multisig.as_ref().expect("multisig not set");
    kmd.import_multisig(handle, msig.version, msig.threshold, &msig.public_keys)
        .await?;
    Ok(())
}

#[then(expr = "the multisig should be in the wallet")]
async fn the_multisig_should_be_in_the_wallet(w: &mut World) -> Result<(), Error> {
    let kmd = w.kmd.as_ref().expect("kmd not set");
    let handle = w.handle.as_ref().expect("wallet handle not set");
    let msig = w.multisig.as_ref().expect("multisig not set");
    let resp = kmd.list_multisig(handle).await?;
    assert!(
        resp.addresses.contains(&msig.address().to_string()),
        "imported multisig not in wallet"
    );
    Ok(())
}

#[when(expr = "I export the multisig")]
async fn i_export_the_multisig(w: &mut World) -> Result<(), Error> {
    let kmd = w.kmd.as_ref().expect("kmd not set");
    let handle = w.handle.as_ref().expect("wallet handle not set");
    let msig = w.multisig.as_ref().expect("multisig not set");
    let resp = kmd.export_multisig(handle, &msig.address()).await?;
    w.exported_multisig_pks = Some(resp.pks);
    Ok(())
}

#[then(expr = "the multisig should equal the exported multisig")]
async fn the_multisig_should_equal_the_exported_multisig(w: &mut World) {
    let msig = w.multisig.as_ref().expect("multisig not set");
    let exported = w
        .exported_multisig_pks
        .as_ref()
        .expect("exported multisig pks not set");
    assert_eq!(
        &msig.public_keys, exported,
        "exported multisig pks differ from imported ones"
    );
}

#[when(expr = "I delete the multisig")]
async fn i_delete_the_multisig(w: &mut World) -> Result<(), Error> {
    let kmd = w.kmd.as_ref().expect("kmd not set");
    let handle = w.handle.as_ref().expect("wallet handle not set");
    let password = w.password.as_ref().expect("wallet password not set");
    let msig = w.multisig.as_ref().expect("multisig not set");
    kmd.delete_multisig(handle, password, &msig.address())
        .await?;
    Ok(())
}

#[then(expr = "the multisig should not be in the wallet")]
async fn the_multisig_should_not_be_in_the_wallet(w: &mut World) -> Result<(), Error> {
    let kmd = w.kmd.as_ref().expect("kmd not set");
    let handle = w.handle.as_ref().expect("wallet handle not set");
    let msig = w.multisig.as_ref().expect("multisig not set");
    let resp = kmd.list_multisig(handle).await?;
    assert!(
        !resp.addresses.contains(&msig.address().to_string()),
        "deleted multisig still in wallet"
    );
    Ok(())
}

#[when(expr = "I sign the multisig transaction with kmd")]
async fn i_sign_the_multisig_transaction_with_kmd(w: &mut World) -> Result<(), Error> {
    let kmd = w.kmd.as_ref().expect("kmd not set");
    let handle = w.handle.as_ref().expect("wallet handle not set");
    let password = w.password.as_ref().expect("wallet password not set");
    let tx = w.tx.as_ref().expect("tx not set");
    let msig = w.multisig.as_ref().expect("multisig not set");

    // The wallet must hold the multisig preimage before kmd will sign on
    // its behalf. Import once; ignore "already imported" errors.
    let _ = kmd
        .import_multisig(handle, msig.version, msig.threshold, &msig.public_keys)
        .await;

    let signing_pk: Ed25519PublicKey = msig.public_keys[0];
    let resp = kmd
        .sign_multisig_transaction(handle, password, tx, signing_pk, None)
        .await?;
    w.kmd_signed_multisig_bytes = Some(resp.multisig);
    Ok(())
}

#[then(expr = "the multisig transaction should equal the kmd signed multisig transaction")]
async fn the_multisig_transaction_should_equal_the_kmd_signed_multisig_transaction(
    w: &mut World,
) -> Result<(), Error> {
    let signed = w.signed_tx.as_ref().expect("signed tx not set");
    let kmd_bytes = w
        .kmd_signed_multisig_bytes
        .as_ref()
        .expect("kmd-signed multisig not set");
    let sdk_msig = match signed.sig() {
        TransactionSignature::Multi(msig) => msig,
        _ => panic!("expected a multisig signature on the signed tx"),
    };
    let sdk_bytes = rmp_serde::to_vec_named(sdk_msig)?;
    assert_eq!(
        &sdk_bytes, kmd_bytes,
        "sdk multisig sig != kmd multisig sig"
    );
    Ok(())
}
