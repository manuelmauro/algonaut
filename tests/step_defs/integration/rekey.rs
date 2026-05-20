use crate::step_defs::{integration::world::World, util::wait_for_pending_transaction};
use algonaut_core::{Address, MicroAlgos, TxId};
use algonaut_transaction::{Pay, TxnBuilder, account::Account};
use cucumber::{given, when};
use std::error::Error;

#[when(expr = "I generate a key using kmd for rekeying and fund it")]
async fn i_generate_a_key_using_kmd_for_rekeying_and_fund_it(
    w: &mut World,
) -> Result<(), Box<dyn Error>> {
    let kmd = w.kmd.as_ref().expect("kmd not set");
    let algod = w.algod.as_ref().expect("algod not set");
    let handle = w.handle.as_ref().expect("wallet handle not set");
    let password = w.password.as_ref().expect("wallet password not set");
    let accounts = w.accounts.as_ref().expect("accounts not set");

    let key_resp = kmd.generate_key(handle).await?;
    let new_addr: Address = key_resp.address.parse()?;

    // Fund the freshly-generated account from the first wallet account so
    // it can pay fees and serve as the rekey sender.
    let funder = accounts[0];
    let funder_key = kmd.export_key(handle, password, &funder).await?;
    let funder_account = Account::from_seed(funder_key.private_key[0..32].try_into()?);
    let params = algod.txn_params().await?;
    let tx = TxnBuilder::with(
        &params,
        Pay::new(funder, new_addr, MicroAlgos(10_000_000)).build(),
    )
    .build()?;
    let signed = funder_account.sign_transaction(tx)?;
    let resp = algod.send_txn(&signed).await?;
    let tx_id: TxId = resp.tx_id.into();
    wait_for_pending_transaction(algod, &tx_id).await?;

    w.rekey_target = Some(new_addr);
    Ok(())
}

#[given(regex = r#"^default transaction with parameters (\d+) "([^"]*)" and rekeying key$"#)]
async fn default_transaction_with_parameters_and_rekeying_key(
    w: &mut World,
    amt: u64,
    note_b64: String,
) -> Result<(), Box<dyn Error>> {
    let algod = w.algod.as_ref().expect("algod not set");
    let accounts = w.accounts.as_ref().expect("accounts not set");
    let rekey_target = w.rekey_target.expect("rekey target not generated");
    let params = algod.txn_params().await?;
    let note = if note_b64.is_empty() {
        Vec::new()
    } else {
        data_encoding::BASE64.decode(note_b64.as_bytes())?
    };
    let mut builder = TxnBuilder::with(
        &params,
        Pay::new(rekey_target, accounts[1], MicroAlgos(amt)).build(),
    );
    if !note.is_empty() {
        builder = builder.note(note.clone());
    }
    w.tx = Some(builder.build()?);
    w.note = Some(note);
    Ok(())
}

#[when(regex = r#"^I add a rekeyTo field with address "([^"]*)"$"#)]
async fn i_add_a_rekey_to_field_with_address(
    w: &mut World,
    rekey_to: String,
) -> Result<(), Box<dyn Error>> {
    let address: Address = rekey_to.parse()?;
    let tx = w.tx.as_mut().expect("tx not set");
    tx.rekey_to = Some(address);
    Ok(())
}

#[when(expr = "I add a rekeyTo field with the private key algorand address")]
async fn i_add_a_rekey_to_field_with_the_private_key_algorand_address(
    w: &mut World,
) -> Result<(), Box<dyn Error>> {
    let account = w
        .sender_account
        .as_ref()
        .expect("sender account not set (call `I get the private key` first)");
    let tx = w.tx.as_mut().expect("tx not set");
    tx.rekey_to = Some(account.address());
    Ok(())
}

#[given(regex = r#"^mnemonic for private key "([^"]*)"$"#)]
async fn mnemonic_for_private_key(w: &mut World, mnemonic: String) -> Result<(), Box<dyn Error>> {
    w.sender_account = Some(Account::from_mnemonic(&mnemonic)?);
    Ok(())
}
