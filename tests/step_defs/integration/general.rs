use std::error::Error;

use crate::step_defs::{
    integration::world::World,
    util::{account_from_kmd_response, wait_for_pending_transaction},
};
use algonaut::{algod::v2::Algod, kmd::v1::Kmd};
use algonaut_core::{MicroAlgos, Round};
use algonaut_transaction::{Pay, TxnBuilder};
use cucumber::{given, then, when};

#[given(regex = "an algod v2 client")]
async fn an_algod_v2_client(w: &mut World) -> Result<(), Box<dyn Error>> {
    let algod = Algod::new(
        "http://localhost:60000",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    )
    .unwrap();

    algod.status_after_round(Round(1)).await?;
    w.algod = Some(algod);

    Ok(())
}

#[given(regex = r#"^an algod v2 client connected to "([^"]*)" port (\d+) with token "([^"]*)"$"#)]
async fn an_algod_v2_client_connected_to(w: &mut World, host: String, port: String, token: String) {
    let algod = Algod::new(&format!("http://{}:{}", host, port), &token).unwrap();
    w.algod = Some(algod)
}

#[given(expr = "a kmd client")]
async fn a_kmd_client(w: &mut World) {
    let kmd = Kmd::new(
        "http://localhost:60001",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    )
    .unwrap();
    w.kmd = Some(kmd)
}

#[given(expr = "wallet information")]
async fn wallet_information(w: &mut World) -> Result<(), Box<dyn Error>> {
    let kmd = w.kmd.as_ref().unwrap();

    let list_response = kmd.list_wallets().await?;
    let wallet_id = match list_response
        .wallets
        .into_iter()
        .find(|wallet| wallet.name == "unencrypted-default-wallet")
    {
        Some(wallet) => wallet.id,
        None => return Err("Wallet not found".into()),
    };
    let password = "";
    let init_response = kmd.init_wallet_handle(&wallet_id, "").await?;

    let keys = kmd
        .list_keys(init_response.wallet_handle_token.as_ref())
        .await?;

    w.password = Some(password.to_owned());
    w.handle = Some(init_response.wallet_handle_token);
    w.accounts = Some(
        keys.addresses
            .into_iter()
            .map(|s| s.parse().unwrap())
            .collect(),
    );

    Ok(())
}

#[given(regex = "suggested transaction parameters from the algod v2 client")]
async fn suggested_params(w: &mut World) -> Result<(), Box<dyn Error>> {
    let algod = w.algod.as_ref().unwrap();

    w.tx_params = Some(algod.suggested_transaction_params().await?);

    Ok(())
}

#[given(regex = r#"^I create a new transient account and fund it with (\d+) microalgos\.$"#)]
async fn i_create_a_new_transient_account_and_fund_it_with_microalgos(
    w: &mut World,
    micro_algos: u64,
) -> Result<(), Box<dyn Error>> {
    let kmd = w.kmd.as_ref().unwrap();
    let algod = w.algod.as_ref().unwrap();
    let accounts = w.accounts.as_ref().unwrap();
    let password = w.password.as_ref().unwrap();
    let handle = w.handle.as_ref().unwrap();

    let sender_address = accounts[1];

    let sender_key = kmd.export_key(handle, password, &sender_address).await?;

    let sender_account = account_from_kmd_response(&sender_key)?;

    let params = algod.suggested_transaction_params().await?;
    let tx = TxnBuilder::with(
        &params,
        Pay::new(
            accounts[1],
            sender_account.address(),
            MicroAlgos(micro_algos),
        )
        .build(),
    )
    .build()?;

    let s_tx = sender_account.sign_transaction(tx)?;

    let send_response = algod.broadcast_signed_transaction(&s_tx).await?;
    let _ = wait_for_pending_transaction(&algod, &send_response.tx_id);

    w.transient_account = Some(sender_account);

    Ok(())
}

#[given(
    regex = r#"I sign and submit the transaction, saving the txid\. If there is an error it is "([^"]*)"\.$"#
)]
#[then(
    regex = r#"I sign and submit the transaction, saving the txid\. If there is an error it is "([^"]*)"\.$"#
)]
#[when(
    regex = r#"I sign and submit the transaction, saving the txid\. If there is an error it is "([^"]*)"\.$"#
)]
async fn i_sign_and_submit_the_transaction_saving_the_tx_id_if_there_is_an_error_it_is(
    w: &mut World,
    err: String,
) {
    let algod = w.algod.as_ref().unwrap();
    let transient_account = w.transient_account.as_ref().unwrap();
    let tx = w.tx.as_ref().unwrap();

    let s_tx = transient_account.sign_transaction(tx.clone()).unwrap();

    match algod.broadcast_signed_transaction(&s_tx).await {
        Ok(response) => {
            w.tx_id = Some(response.tx_id);
        }
        Err(e) => {
            assert!(e.to_string().contains(&err));
        }
    }
}

#[given(expr = "I wait for the transaction to be confirmed.")]
#[then(expr = "I wait for the transaction to be confirmed.")]
#[when(expr = "I wait for the transaction to be confirmed.")]
async fn i_wait_for_the_transaction_to_be_confirmed(w: &mut World) {
    let algod = w.algod.as_ref().expect("algod not set");
    let tx_id = w.tx_id.as_ref().expect("tx id not set");

    wait_for_pending_transaction(&algod, &tx_id)
        .await
        .expect("couldn't get pending tx");
}
