use crate::step_defs::util::{
    account_from_kmd_response, parse_app_args, split_addresses, split_uint64,
    wait_for_pending_transaction,
};
use algonaut::{algod::v2::Algod, kmd::v1::Kmd};
use algonaut_core::{Address, CompiledTeal, MicroAlgos};
use algonaut_model::algod::v2::{Application, ApplicationLocalState};
use algonaut_transaction::account::Account;
use algonaut_transaction::builder::{
    CallApplication, ClearApplication, CloseApplication, DeleteApplication, OptInApplication,
    UpdateApplication,
};
use algonaut_transaction::transaction::StateSchema;
use algonaut_transaction::{CreateApplication, Pay, Transaction, TxnBuilder};
use async_trait::async_trait;
use cucumber::{given, then, WorldInit};
use data_encoding::BASE64;
use std::convert::Infallible;
use std::error::Error;
use std::fs;

#[derive(Default, Debug, WorldInit)]
pub struct World {
    algod: Option<Algod>,

    kmd: Option<Kmd>,
    handle: Option<String>,
    password: Option<String>,
    accounts: Option<Vec<Address>>,

    transient_account: Option<Account>,

    tx: Option<Transaction>,
    tx_id: Option<String>,

    app_id: Option<u64>,
}

#[async_trait(?Send)]
impl cucumber::World for World {
    type Error = Infallible;

    async fn new() -> Result<Self, Self::Error> {
        Ok(Self::default())
    }
}

#[given(expr = "an algod client")]
async fn an_algod_client(_: &mut World) {
    // Do nothing - we don't support v1
    // The reference (Go) SDK doesn't use it in the definitions
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

#[given(regex = r#"^an algod v2 client connected to "([^"]*)" port (\d+) with token "([^"]*)"$"#)]
async fn an_algod_v2_client_connected_to(w: &mut World, host: String, port: String, token: String) {
    let algod = Algod::new(&format!("http://{}:{}", host, port), &token).unwrap();
    w.algod = Some(algod)
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
        params,
        Pay::new(
            accounts[1],
            sender_account.address(),
            MicroAlgos(micro_algos),
        )
        .build(),
    )
    .build()?;

    let s_tx = sender_account.sign_transaction(&tx)?;

    let send_response = algod.broadcast_signed_transaction(&s_tx).await?;
    let _ = wait_for_pending_transaction(&algod, &send_response.tx_id);

    w.transient_account = Some(sender_account);

    Ok(())
}

#[given(
    regex = r#"^I build an application transaction with the transient account, the current application, suggested params, operation "([^"]*)", approval-program "([^"]*)", clear-program "([^"]*)", global-bytes (\d+), global-ints (\d+), local-bytes (\d+), local-ints (\d+), app-args "([^"]*)", foreign-apps "([^"]*)", foreign-assets "([^"]*)", app-accounts "([^"]*)", extra-pages (\d+)$"#
)]
#[then(
    regex = r#"^I build an application transaction with the transient account, the current application, suggested params, operation "([^"]*)", approval-program "([^"]*)", clear-program "([^"]*)", global-bytes (\d+), global-ints (\d+), local-bytes (\d+), local-ints (\d+), app-args "([^"]*)", foreign-apps "([^"]*)", foreign-assets "([^"]*)", app-accounts "([^"]*)", extra-pages (\d+)$"#
)]
async fn i_build_an_application_transaction(
    w: &mut World,
    operation: String,
    approval_program_file: String,
    clear_program_file: String,
    global_bytes: u64,
    global_ints: u64,
    local_bytes: u64,
    local_ints: u64,
    app_args: String,
    foreign_apps: String,
    foreign_assets: String,
    app_accounts: String,
    extra_pages: u64,
) -> Result<(), Box<dyn Error>> {
    let algod = w.algod.as_ref().unwrap();
    let transient_account = w.transient_account.as_ref().unwrap();

    let args = parse_app_args(app_args)?;

    let accounts = split_addresses(app_accounts)?;

    let foreign_apps = split_uint64(&foreign_apps)?;
    let foreign_assets = split_uint64(&foreign_assets)?;

    let params = algod.suggested_transaction_params().await?;

    let tx_type = match operation.as_str() {
        "create" => {
            let approval_program = load_teal(&approval_program_file)?;
            let clear_program = load_teal(&clear_program_file)?;

            let global_schema = StateSchema {
                number_ints: global_ints,
                number_byteslices: global_bytes,
            };

            let local_schema = StateSchema {
                number_ints: local_ints,
                number_byteslices: local_bytes,
            };

            CreateApplication::new(
                transient_account.address(),
                CompiledTeal(approval_program),
                CompiledTeal(clear_program),
                global_schema,
                local_schema,
            )
            .foreign_assets(foreign_assets)
            .foreign_apps(foreign_apps)
            .accounts(accounts)
            .app_arguments(args)
            .extra_pages(extra_pages)
            .build()
        }
        "update" => {
            let app_id = w.app_id.unwrap();

            let approval_program = load_teal(&approval_program_file)?;
            let clear_program = load_teal(&clear_program_file)?;

            UpdateApplication::new(
                transient_account.address(),
                app_id,
                CompiledTeal(approval_program),
                CompiledTeal(clear_program),
            )
            .foreign_assets(foreign_assets)
            .foreign_apps(foreign_apps)
            .accounts(accounts)
            .app_arguments(args)
            .build()
        }
        "call" => {
            let app_id = w.app_id.unwrap();
            CallApplication::new(transient_account.address(), app_id)
                .foreign_assets(foreign_assets)
                .foreign_apps(foreign_apps)
                .accounts(accounts)
                .app_arguments(args)
                .build()
        }
        "optin" => {
            let app_id = w.app_id.unwrap();

            OptInApplication::new(transient_account.address(), app_id)
                .foreign_assets(foreign_assets)
                .foreign_apps(foreign_apps)
                .accounts(accounts)
                .app_arguments(args)
                .build()
        }
        "clear" => {
            let app_id = w.app_id.unwrap();
            ClearApplication::new(transient_account.address(), app_id)
                .foreign_assets(foreign_assets)
                .foreign_apps(foreign_apps)
                .accounts(accounts)
                .app_arguments(args)
                .build()
        }
        "closeout" => {
            let app_id = w.app_id.unwrap();
            CloseApplication::new(transient_account.address(), app_id)
                .foreign_assets(foreign_assets)
                .foreign_apps(foreign_apps)
                .accounts(accounts)
                .app_arguments(args)
                .build()
        }
        "delete" => {
            let app_id = w.app_id.unwrap();
            DeleteApplication::new(transient_account.address(), app_id)
                .foreign_assets(foreign_assets)
                .foreign_apps(foreign_apps)
                .accounts(accounts)
                .app_arguments(args)
                .build()
        }

        _ => Err(format!("Invalid str: {}", operation))?,
    };

    w.tx = Some(TxnBuilder::with(params, tx_type).build()?);

    Ok(())
}

#[given(
    regex = r#"I sign and submit the transaction, saving the txid\. If there is an error it is "([^"]*)"\.$"#
)]
#[then(
    regex = r#"I sign and submit the transaction, saving the txid\. If there is an error it is "([^"]*)"\.$"#
)]
async fn i_sign_and_submit_the_transaction_saving_the_tx_id_if_there_is_an_error_it_is(
    w: &mut World,
    err: String,
) {
    let algod = w.algod.as_ref().unwrap();
    let transient_account = w.transient_account.as_ref().unwrap();
    let tx = w.tx.as_ref().unwrap();

    let s_tx = transient_account.sign_transaction(&tx).unwrap();

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
async fn i_wait_for_the_transaction_to_be_confirmed(w: &mut World) {
    let algod = w.algod.as_ref().unwrap();
    let tx_id = w.tx_id.as_ref().unwrap();

    wait_for_pending_transaction(&algod, &tx_id).await.unwrap();
}

#[given(expr = "I remember the new application ID.")]
async fn i_remember_the_new_application_id(w: &mut World) {
    let algod = w.algod.as_ref().unwrap();
    let tx_id = w.tx_id.as_ref().unwrap();

    let p_tx = algod.pending_transaction_with_id(tx_id).await.unwrap();
    assert!(p_tx.application_index.is_some());

    w.app_id = p_tx.application_index;
}

#[then(
    regex = r#"^The transient account should have the created app "([^"]*)" and total schema byte-slices (\d+) and uints (\d+), the application "([^"]*)" state contains key "([^"]*)" with value "([^"]*)"$"#
)]
async fn the_transient_account_should_have(
    w: &mut World,
    app_created: bool,
    byte_slices: u64,
    uints: u64,
    application_state: String,
    key: String,
    value: String,
) -> Result<(), Box<dyn Error>> {
    let algod = w.algod.as_ref().unwrap();
    let transient_account = w.transient_account.as_ref().unwrap();
    let app_id = w.app_id.unwrap();

    let account_infos = algod
        .account_information(&transient_account.address())
        .await
        .unwrap();

    assert!(account_infos.apps_total_schema.is_some());
    let total_schema = account_infos.apps_total_schema.unwrap();

    assert_eq!(byte_slices, total_schema.num_byte_slice);
    assert_eq!(uints, total_schema.num_uint);

    let app_in_account = account_infos.created_apps.iter().any(|a| a.id == app_id);

    match (app_created, app_in_account) {
        (true, false) => Err(format!("AppId {} is not found in the account", app_id))?,
        (false, true) => {
            // If no app was created, we don't expect it to be in the account
            Err("AppId is not expected to be in the account")?
        }
        _ => {}
    }

    if key.is_empty() {
        return Ok(());
    }

    let key_values = match application_state.to_lowercase().as_ref() {
        "local" => {
            let local_state = account_infos
                .apps_local_state
                .iter()
                .filter(|s| s.id == app_id)
                .collect::<Vec<&ApplicationLocalState>>();

            let len = local_state.len();
            if len == 1 {
                local_state[0].key_value.clone()
            } else {
                Err(format!(
                    "Expected only one matching local state, found {}",
                    len
                ))?
            }
        }
        "global" => {
            let apps = account_infos
                .created_apps
                .iter()
                .filter(|s| s.id == app_id)
                .collect::<Vec<&Application>>();

            let len = apps.len();
            if len == 1 {
                apps[0].params.global_state.clone()
            } else {
                Err(format!("Expected only one matching app, found {}", len))?
            }
        }
        _ => Err(format!("Unknown application state: {}", application_state))?,
    };

    if key_values.is_empty() {
        Err("Expected key values length to be greater than 0")?
    }

    let mut key_value_found = false;
    for key_value in key_values.iter().filter(|kv| kv.key == key) {
        if key_value.value.value_type == 1 {
            let value_bytes = BASE64.decode(value.as_bytes())?;
            if key_value.value.bytes != value_bytes {
                Err(format!(
                    "Value mismatch (bytes): expected: '{:?}', got '{:?}'",
                    value_bytes, key_value.value.bytes
                ))?
            }
        } else if key_value.value.value_type == 0 {
            let int_value = value.parse::<u64>()?;

            if key_value.value.uint != int_value {
                Err(format!(
                    "Value mismatch (uint): expected: '{}', got '{}'",
                    value, key_value.value.uint
                ))?
            }
        }
        key_value_found = true;
    }

    if !key_value_found {
        Err(format!("Couldn't find key: '{}'", key))?
    }

    Ok(())
}

fn load_teal(file_name: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(fs::read(format!("tests/features/resources/{}", file_name))?)
}
