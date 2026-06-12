use crate::step_defs::{integration::world::World, util::account_from_kmd_response};
use algonaut::{algod::v2::Algod, kmd::v1::Kmd};
use algonaut_core::{Address, MicroAlgos, Round, TransactionId};
use algonaut_transaction::Pay;
use algonaut_transaction::account::Account;
use cucumber::{given, then, when};
use rand::Rng;
use std::error::Error;

/// Find the wallet account with the highest microalgos balance.
/// kmd's list_keys ordering is unstable across sandbox runs, so the
/// pre-funded "creator" account is not guaranteed to land at any
/// particular index.
pub async fn pick_funded_account(
    algod: &Algod,
    accounts: &[Address],
) -> Result<Address, Box<dyn Error>> {
    let mut best: Option<(Address, u64)> = None;
    for addr in accounts {
        let info = algod.account(addr).await?;
        if best.is_none_or(|(_, bal)| info.amount > bal) {
            best = Some((*addr, info.amount));
        }
    }
    let (addr, bal) = best.ok_or("wallet has no accounts")?;
    if bal == 0 {
        return Err("no funded account found in the wallet".into());
    }
    Ok(addr)
}

#[given(regex = r"^an algod v2 client$")]
async fn an_algod_v2_client(w: &mut World) -> Result<(), Box<dyn Error>> {
    let algod = Algod::new(
        "http://localhost:60000",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    )
    .unwrap();

    algod.status_after_block(Round(1)).await?;
    w.algod = Some(algod);

    Ok(())
}

#[given(regex = r#"^an algod v2 client connected to "([^"]*)" port (\d+) with token "([^"]*)"$"#)]
async fn an_algod_v2_client_connected_to(w: &mut World, host: String, port: String, token: String) {
    let algod = Algod::new(&format!("http://{}:{}", host, port), &token).unwrap();
    w.algod = Some(algod)
}

#[given(regex = r"^an indexer v2 client$")]
async fn an_indexer_v2_client(w: &mut World) {
    let indexer = algonaut::indexer::v2::Indexer::new("http://localhost:60002", "").unwrap();
    w.indexer = Some(indexer);
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

#[given(regex = r"^suggested transaction parameters from the algod v2 client$")]
async fn suggested_params(w: &mut World) -> Result<(), Box<dyn Error>> {
    let algod = w.algod.as_ref().unwrap();

    w.suggested_params = Some(algod.suggested_params().await?);

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
    // add dust to make the funding transaction unique (with high probability)
    // within a block
    let mut rng = rand::thread_rng();
    let dust: u64 = rng.gen_range(1..1_000_000);

    // Pick the funded account (kmd's list_keys ordering is unstable, so
    // accounts[0] isn't reliably the creator) and use it to fund a brand-new
    // account.
    let funder_address = pick_funded_account(algod, accounts).await?;
    let funder_key = kmd.export_key(handle, password, &funder_address).await?;
    let funder = account_from_kmd_response(&funder_key)?;

    // A genuinely *new* account, funded with exactly the requested amount.
    // Reusing the shared funder as the "transient" account broke isolation:
    // the account held the funder's full balance, so scenarios asserting an
    // overspend (e.g. a self-pay larger than the funded amount) never tripped
    // it, and a rekey/spend leaked into later scenarios.
    let transient = Account::generate();

    let params = algod.suggested_params().await?;
    let tx = Pay::new(
        funder_address,
        transient.address(),
        MicroAlgos(micro_algos + dust),
    )
    .build(&params)?;

    let s_tx = funder.sign(tx)?;
    algod.submit(&s_tx).await?.confirm().await?;

    w.transient_account = Some(transient);

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

    let s_tx = transient_account.sign(tx.clone()).unwrap();

    match algod.send(&s_tx).await {
        Ok(response) => {
            w.transaction_id = Some(TransactionId(response.tx_id));
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
    let transaction_id = w.transaction_id.as_ref().expect("tx id not set");

    algod
        .pending_submission(transaction_id)
        .confirm()
        .await
        .expect("couldn't get pending tx");
}
