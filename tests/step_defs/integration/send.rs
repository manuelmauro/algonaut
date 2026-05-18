use crate::step_defs::{integration::world::World, util::account_from_kmd_response};
use algonaut_core::{MicroAlgos, MultisigAddress};
use algonaut_transaction::{Pay, SignedTransaction, TxnBuilder, transaction::TransactionSignature};
use cucumber::{given, then, when};
use data_encoding::BASE64;
use std::error::Error;

async fn build_default_payment(
    w: &mut World,
    sender_index: usize,
    receiver_index: usize,
    amt: u64,
    note_b64: &str,
) -> Result<(), Box<dyn Error>> {
    let algod = w.algod.as_ref().expect("algod not set");
    let accounts = w.accounts.as_ref().expect("accounts not set");
    let params = algod.txn_params().await?;
    let note = if note_b64.is_empty() {
        Vec::new()
    } else {
        BASE64.decode(note_b64.as_bytes())?
    };

    let mut tx_builder = TxnBuilder::with(
        &params,
        Pay::new(
            accounts[sender_index],
            accounts[receiver_index],
            MicroAlgos(amt),
        )
        .build(),
    );
    if !note.is_empty() {
        tx_builder = tx_builder.note(note.clone());
    }
    w.tx = Some(tx_builder.build()?);
    w.note = Some(note);
    Ok(())
}

#[given(regex = r#"^default transaction with parameters (\d+) "([^"]*)"$"#)]
async fn default_transaction_with_parameters(
    w: &mut World,
    amt: u64,
    note_b64: String,
) -> Result<(), Box<dyn Error>> {
    build_default_payment(w, 0, 1, amt, &note_b64).await
}

#[given(regex = r#"^default multisig transaction with parameters (\d+) "([^"]*)"$"#)]
async fn default_multisig_transaction_with_parameters(
    w: &mut World,
    amt: u64,
    note_b64: String,
) -> Result<(), Box<dyn Error>> {
    let algod = w.algod.as_ref().expect("algod not set").clone();
    let accounts = w.accounts.as_ref().expect("accounts not set").clone();

    let msig =
        MultisigAddress::new(1, 1, &accounts).map_err(|e| format!("invalid multisig: {e}"))?;
    let params = algod.txn_params().await?;
    let note = if note_b64.is_empty() {
        Vec::new()
    } else {
        BASE64.decode(note_b64.as_bytes())?
    };

    let mut tx_builder = TxnBuilder::with(
        &params,
        Pay::new(msig.address(), accounts[1], MicroAlgos(amt)).build(),
    );
    if !note.is_empty() {
        tx_builder = tx_builder.note(note.clone());
    }
    w.tx = Some(tx_builder.build()?);
    w.note = Some(note);
    w.multisig = Some(msig);
    Ok(())
}

#[when(expr = "I get the private key")]
async fn i_get_the_private_key(w: &mut World) -> Result<(), Box<dyn Error>> {
    let kmd = w.kmd.as_ref().expect("kmd not set");
    let handle = w.handle.as_ref().expect("wallet handle not set");
    let password = w.password.as_ref().expect("wallet password not set");
    let tx = w.tx.as_ref().expect("tx not set");

    // For a multisig sender we need to export the key of *one* of the
    // multisig participants (we always sign with accounts[0]). Otherwise
    // export the key of the tx sender itself.
    let address = match &w.multisig {
        Some(_) => w.accounts.as_ref().expect("accounts not set")[0],
        None => tx.sender(),
    };
    let key_resp = kmd.export_key(handle, password, &address).await?;
    w.sender_account = Some(account_from_kmd_response(&key_resp)?);
    Ok(())
}

#[when(expr = "I sign the transaction with the private key")]
async fn i_sign_the_transaction_with_the_private_key(w: &mut World) -> Result<(), Box<dyn Error>> {
    let account = w.sender_account.as_ref().expect("sender account not set");
    let tx = w.tx.as_ref().expect("tx not set");
    w.signed_tx = Some(account.sign_transaction(tx.clone())?);
    Ok(())
}

#[when(expr = "I sign the multisig transaction with the private key")]
async fn i_sign_the_multisig_transaction_with_the_private_key(
    w: &mut World,
) -> Result<(), Box<dyn Error>> {
    let account = w.sender_account.as_ref().expect("sender account not set");
    let msig = w.multisig.as_ref().expect("multisig not set");
    let tx = w.tx.as_ref().expect("tx not set");

    let multisig_sig = account.init_transaction_msig(tx, msig)?;
    let transaction_id = tx.id()?;
    w.signed_tx = Some(SignedTransaction {
        transaction: tx.clone(),
        transaction_id,
        sig: TransactionSignature::Multi(multisig_sig),
        auth_address: None,
    });
    Ok(())
}

#[when(expr = "I send the transaction")]
#[when(expr = "I send the multisig transaction")]
async fn i_send_the_transaction(w: &mut World) {
    let algod = w.algod.as_ref().expect("algod not set");
    let signed = w.signed_tx.as_ref().expect("signed tx not set");

    match algod.send_txn(signed).await {
        Ok(resp) => {
            w.tx_id = Some(resp.tx_id);
            w.last_send_succeeded = Some(true);
        }
        Err(_) => {
            w.last_send_succeeded = Some(false);
        }
    }
}

#[then(expr = "the transaction should not go through")]
async fn the_transaction_should_not_go_through(w: &mut World) {
    assert_eq!(
        w.last_send_succeeded,
        Some(false),
        "expected send to fail, but it succeeded"
    );
}
