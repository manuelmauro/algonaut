use crate::step_defs::integration::world::World;
use crate::step_defs::util::{parse_app_args, read_teal, split_addresses, split_uint64};
use algonaut_algod::models::{Application, ApplicationLocalState};
use algonaut_transaction::builder::{
    CallApplication, ClearApplication, CloseApplication, DeleteApplication, OptInApplication,
    UpdateApplication,
};
use algonaut_transaction::transaction::StateSchema;
use algonaut_transaction::{CreateApplication, TxnBuilder};
use cucumber::{given, then, when};
use data_encoding::BASE64;
use std::error::Error;

#[given(
    regex = r#"^I build an application transaction with the transient account, the current application, suggested params, operation "([^"]*)", approval-program "([^"]*)", clear-program "([^"]*)", global-bytes (\d+), global-ints (\d+), local-bytes (\d+), local-ints (\d+), app-args "([^"]*)", foreign-apps "([^"]*)", foreign-assets "([^"]*)", app-accounts "([^"]*)", extra-pages (\d+)$"#
)]
#[then(
    regex = r#"^I build an application transaction with the transient account, the current application, suggested params, operation "([^"]*)", approval-program "([^"]*)", clear-program "([^"]*)", global-bytes (\d+), global-ints (\d+), local-bytes (\d+), local-ints (\d+), app-args "([^"]*)", foreign-apps "([^"]*)", foreign-assets "([^"]*)", app-accounts "([^"]*)", extra-pages (\d+)$"#
)]
#[when(
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
    extra_pages: u32,
) -> Result<(), Box<dyn Error>> {
    let algod = w.algod.as_ref().unwrap();
    let transient_account = w.transient_account.as_ref().unwrap();

    let args = parse_app_args(app_args)?;

    let accounts = split_addresses(app_accounts)?;

    let foreign_apps = split_uint64(&foreign_apps)?;
    let foreign_assets = split_uint64(&foreign_assets)?;

    let params = algod.txn_params().await?;

    let tx_type = match operation.as_str() {
        "create" => {
            let approval_program = read_teal(algod, &approval_program_file).await;
            let clear_program = read_teal(algod, &clear_program_file).await;

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
                approval_program,
                clear_program,
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

            let approval_program = read_teal(algod, &approval_program_file).await;
            let clear_program = read_teal(algod, &clear_program_file).await;

            UpdateApplication::new(
                transient_account.address(),
                app_id,
                approval_program,
                clear_program,
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

    w.tx = Some(TxnBuilder::with(&params, tx_type).build()?);

    Ok(())
}

#[given(expr = "I remember the new application ID.")]
#[when(expr = "I remember the new application ID.")]
async fn i_remember_the_new_application_id(w: &mut World) {
    let algod = w.algod.as_ref().unwrap();
    let tx_id = w.tx_id.as_ref().unwrap();
    let app_ids: &mut Vec<u64> = w.app_ids.as_mut();

    let p_tx = algod.pending_txn(tx_id).await.unwrap();
    assert!(p_tx.application_index.is_some());
    let app_id = p_tx.application_index.unwrap();

    w.app_id = Some(app_id);
    app_ids.push(app_id);
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
        .account(&transient_account.address().to_string())
        .await
        .unwrap();

    assert!(account_infos.clone().apps_total_schema.is_some());
    let total_schema = account_infos.clone().apps_total_schema.unwrap();

    assert_eq!(byte_slices, total_schema.num_byte_slice);
    assert_eq!(uints, total_schema.num_uint);

    let app_in_account = account_infos
        .clone()
        .created_apps
        .unwrap()
        .iter()
        .any(|a| a.id == app_id);

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
                .clone()
                .apps_local_state
                .unwrap()
                .into_iter()
                .filter(|s| s.id == app_id)
                .collect::<Vec<ApplicationLocalState>>();

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
                .clone()
                .created_apps
                .unwrap()
                .into_iter()
                .filter(|s| s.id == app_id)
                .collect::<Vec<Application>>();

            let len = apps.len();
            if len == 1 {
                apps[0].params.global_state.clone()
            } else {
                Err(format!("Expected only one matching app, found {}", len))?
            }
        }
        _ => Err(format!("Unknown application state: {}", application_state))?,
    };

    if key_values.is_none() {
        Err("Expected key values length to be greater than 0")?
    }

    let mut key_value_found = false;
    for key_value in key_values.unwrap().iter().filter(|kv| kv.key == key) {
        if (*key_value.value).value_type == 1 {
            let value_bytes = BASE64.decode(value.as_bytes())?;
            if (*key_value.value).bytes != value_bytes {
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
