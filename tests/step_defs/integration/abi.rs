use crate::step_defs::{
    integration::world::World,
    util::{read_teal, wait_for_pending_transaction},
};
use algonaut::{
    atomic_transaction_composer::{
        transaction_signer::TransactionSigner, AbiArgValue, AbiMethodReturnValue,
        AbiReturnDecodeError, AddMethodCallParams, AtomicTransactionComposer,
        AtomicTransactionComposerStatus, TransactionWithSigner,
    },
    error::ServiceError,
};
use algonaut_abi::{
    abi_interactions::{AbiArgType, AbiMethod, AbiReturn, AbiReturnType, ReferenceArgType},
    abi_type::{AbiType, AbiValue},
};
use algonaut_algod::models::PendingTransactionResponse;
use algonaut_core::{to_app_address, Address, MicroAlgos};
use algonaut_transaction::{
    transaction::{ApplicationCallOnComplete, StateSchema},
    Pay, TxnBuilder,
};
use cucumber::{codegen::Regex, given, then, when};
use data_encoding::BASE64;
use num_traits::ToPrimitive;
use sha2::Digest;
use std::convert::TryInto;
use std::error::Error;

#[given(regex = r#"^I make a transaction signer for the ([^"]*) account\.$"#)]
#[when(regex = r#"^I make a transaction signer for the ([^"]*) account\.$"#)]
async fn i_make_a_transaction_signer_for_the_account(w: &mut World, account_str: String) {
    let signer = TransactionSigner::BasicAccount(match account_str.as_ref() {
        "transient" => w.transient_account.clone().unwrap(),
        _ => panic!("Not handled account string: {}", account_str),
    });

    w.tx_signer = Some(signer);
}

#[given(expr = "a new AtomicTransactionComposer")]
async fn a_new_atomic_transaction_composer(w: &mut World) {
    w.tx_composer = Some(AtomicTransactionComposer::default());
    w.tx_composer_methods = Some(vec![]);
}

#[when(
    regex = r#"^I build a payment transaction with sender "([^"]*)", receiver "([^"]*)", amount (\d+), close remainder to "([^"]*)"$"#
)]
#[given(
    regex = r#"^I build a payment transaction with sender "([^"]*)", receiver "([^"]*)", amount (\d+), close remainder to "([^"]*)"$"#
)]
async fn i_build_a_payment_transaction_with_sender_receiver_amount_close_remainder_to(
    w: &mut World,
    sender_str: String,
    receiver_str: String,
    amount: u64,
    close_to: String,
) {
    let transient_account = w.transient_account.clone().unwrap();
    let tx_params = w.tx_params.as_ref().unwrap();

    let close_to = if close_to == "" {
        None
    } else {
        Some(close_to.parse::<Address>().unwrap())
    };

    let sender = if sender_str == "transient" {
        transient_account.address()
    } else {
        panic!("sender_str not supported: {}", sender_str);
    };

    let receiver = if receiver_str == "transient" {
        transient_account.address()
    } else {
        panic!("receiver_str not supported: {}", receiver_str);
    };

    let mut payment = Pay::new(sender, receiver, MicroAlgos(amount));
    if let Some(close_to) = close_to {
        payment = payment.close_remainder_to(close_to);
    }

    let tx = TxnBuilder::with(tx_params, payment.build())
        .build()
        .unwrap();

    w.tx = Some(tx);
}

#[when(expr = "I create a transaction with signer with the current transaction.")]
#[given(expr = "I create a transaction with signer with the current transaction.")]
async fn i_create_a_transaction_with_signer_with_the_current_transaction(w: &mut World) {
    let tx = w.tx.clone().unwrap();
    let signer = w.tx_signer.clone().unwrap();

    w.tx_with_signer = Some(TransactionWithSigner { tx, signer });
}

#[when(expr = "I add the current transaction with signer to the composer.")]
async fn i_add_the_current_transaction_with_signer_tothecomposer(w: &mut World) {
    let tx_with_signer = w.tx_with_signer.clone().unwrap();
    let tx_composer = w.tx_composer.as_mut().unwrap();

    tx_composer.add_transaction(tx_with_signer).unwrap();
}

#[then(expr = "I gather signatures with the composer.")]
async fn i_gather_signatures_with_the_composer(w: &mut World) {
    let tx_composer = w.tx_composer.as_mut().unwrap();

    w.signed_txs = Some(tx_composer.gather_signatures().unwrap());
}

#[then(regex = r#"^The composer should have a status of "([^"]*)"\.$"#)]
async fn the_composer_should_have_a_status_of(w: &mut World, status_str: String) {
    let tx_composer = w.tx_composer.as_mut().unwrap();

    let status = match status_str.as_ref() {
        "BUILDING" => AtomicTransactionComposerStatus::Building,
        "BUILT" => AtomicTransactionComposerStatus::Built,
        "SIGNED" => AtomicTransactionComposerStatus::Signed,
        "SUBMITTED" => AtomicTransactionComposerStatus::Submitted,
        "COMMITTED" => AtomicTransactionComposerStatus::Committed,
        _ => panic!("Not handled status string: {}", status_str),
    };

    if status != tx_composer.status() {
        panic!("status doesn't match");
    }
}

#[then(expr = "I clone the composer.")]
async fn i_clone_the_composer(w: &mut World) {
    let tx_composer = w.tx_composer.as_mut().unwrap();

    w.tx_composer = Some(tx_composer.clone_composer());
}

#[when(regex = r#"I create the Method object from method signature "([^"]*)"$"#)]
#[given(regex = r#"I create the Method object from method signature "([^"]*)"$"#)]
async fn create_method_object_from_signature(w: &mut World, method_sig: String) {
    let abi_method = AbiMethod::from_signature(&method_sig).unwrap();
    w.abi_method = Some(abi_method);
}

#[given(expr = "I create a new method arguments array.")]
#[when(expr = "I create a new method arguments array.")]
async fn i_create_a_new_method_arguments_array(w: &mut World) {
    let abi_method = w.abi_method.as_ref().unwrap();
    let mut arg_types = vec![];
    for mut arg_type in abi_method.args.clone() {
        match arg_type.type_().expect("no type") {
            AbiArgType::Tx(_) => continue,
            AbiArgType::Ref(rf) => match rf {
                ReferenceArgType::Account => arg_types.push(AbiType::address()),
                _ => arg_types.push(AbiType::uint(64).expect("couldn't create int type")),
            },
            AbiArgType::AbiObj(obj) => {
                arg_types.push(obj);
            }
        }
    }
    w.abi_method_arg_types = Some(arg_types);
    // w.abi_method_args = Some(arg_types);
    w.abi_method_arg_values = Some(vec![]);
}

#[given(regex = r#"I append the encoded arguments "([^"]*)" to the method arguments array.$"#)]
#[when(regex = r#"I append the encoded arguments "([^"]*)" to the method arguments array.$"#)]
async fn i_append_the_encoded_arguments_to_the_method_arguments_array(
    w: &mut World,
    comma_separated_b64_args: String,
) -> Result<(), Box<dyn Error>> {
    let application_ids: &[u64] = w.app_ids.as_ref();
    let method_args = w.abi_method_arg_values.as_mut().expect("no method args");

    let abi_method_arg_types = w
        .abi_method_arg_types
        .as_ref()
        .expect("No method arg types");

    if comma_separated_b64_args.is_empty() {
        return Ok(());
    }

    let b64_args = comma_separated_b64_args.split(',');
    for (arg_index, b64_arg) in b64_args.into_iter().enumerate() {
        if b64_arg.contains(':') {
            let parts: Vec<&str> = b64_arg.split(':').collect();
            if parts.len() != 2 || parts[0] != "ctxAppIdx" {
                panic!("Cannot process argument: {}", b64_arg);
            }
            let parsed_index = parts[1].parse::<usize>().unwrap();
            if parsed_index >= application_ids.len() {
                panic!(
                    "Application index out of bounds: {}, number of app IDs is {}",
                    parsed_index,
                    application_ids.len()
                );
            }

            let arg = AbiValue::Int(application_ids[parsed_index].into());
            method_args.push(AbiArgValue::AbiValue(arg));
        } else {
            let base64_decoded_arg = BASE64.decode(b64_arg.as_bytes()).unwrap();

            let decoded = abi_method_arg_types[arg_index].decode(&base64_decoded_arg)?;
            method_args.push(AbiArgValue::AbiValue(decoded));
        }
    }
    Ok(())
}

#[when(
    regex = r#"^I add a method call with the ([^"]*) account, the current application, suggested params, on complete "([^"]*)", current transaction signer, current method arguments.$"#
)]
#[given(
    regex = r#"^I add a method call with the ([^"]*) account, the current application, suggested params, on complete "([^"]*)", current transaction signer, current method arguments.$"#
)]
async fn i_add_a_method_call(w: &mut World, account_type: String, on_complete: String) {
    add_method_call(
        w,
        account_type,
        on_complete,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
    )
    .await;
}

#[when(
    regex = r#"^I add a method call with the ([^"]*) account, the current application, suggested params, on complete "([^"]*)", current transaction signer, current method arguments, approval-program "([^"]*)", clear-program "([^"]*)"\.$"#
)]
async fn i_add_a_method_call_for_update(
    w: &mut World,
    account_type: String,
    on_complete: String,
    approval_program: String,
    clear_program: String,
) {
    add_method_call(
        w,
        account_type,
        on_complete,
        Some(approval_program),
        Some(clear_program),
        None,
        None,
        None,
        None,
        None,
        false,
    )
    .await;
}

#[when(
    regex = r#"^I add a method call with the ([^"]*) account, the current application, suggested params, on complete "([^"]*)", current transaction signer, current method arguments, approval-program "([^"]*)", clear-program "([^"]*)", global-bytes (\d+), global-ints (\d+), local-bytes (\d+), local-ints (\d+), extra-pages (\d+)\.$"#
)]
async fn i_add_a_method_call_for_create(
    w: &mut World,
    account_type: String,
    on_complete: String,
    approval_program: String,
    clear_program: String,
    global_bytes: u64,
    global_ints: u64,
    local_bytes: u64,
    local_ints: u64,
    extra_pages: u32,
) {
    add_method_call(
        w,
        account_type,
        on_complete,
        Some(approval_program),
        Some(clear_program),
        Some(global_bytes),
        Some(global_ints),
        Some(local_bytes),
        Some(local_ints),
        Some(extra_pages),
        false,
    )
    .await;
}

#[given(
    regex = r#"^I add a nonced method call with the ([^"]*) account, the current application, suggested params, on complete "([^"]*)", current transaction signer, current method arguments\.$"#
)]
async fn i_add_method_call_with_nonce(w: &mut World, account_type: String, on_complete: String) {
    add_method_call(
        w,
        account_type,
        on_complete,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        true,
    )
    .await;
}

async fn add_method_call(
    w: &mut World,
    account_type: String,
    on_complete: String,
    approval_program: Option<String>,
    clear_program: Option<String>,
    global_bytes: Option<u64>,
    global_ints: Option<u64>,
    local_bytes: Option<u64>,
    local_ints: Option<u64>,
    extra_pages: Option<u32>,
    use_nonce: bool,
) {
    let algod = w.algod.as_ref().unwrap();
    let transient_account = w.transient_account.clone().unwrap();
    let abi_method = w.abi_method.as_ref().unwrap();
    let abi_method_args = w.abi_method_arg_values.as_mut().unwrap();
    let application_id = w.app_id.clone().unwrap();
    let tx_params = w.tx_params.clone().unwrap();
    let tx_signer = w.tx_signer.clone().unwrap();
    let tx_composer = w.tx_composer.as_mut().unwrap();
    let tx_composer_methods = w.tx_composer_methods.as_mut().unwrap();

    let extra_pages = extra_pages.unwrap_or(0);

    let global_schema = match (global_ints, global_bytes) {
        (Some(ints), Some(bytes)) => Some(StateSchema {
            number_ints: ints,
            number_byteslices: bytes,
        }),
        _ => None,
    };

    let local_schema = match (local_ints, local_bytes) {
        (Some(ints), Some(bytes)) => Some(StateSchema {
            number_ints: ints,
            number_byteslices: bytes,
        }),
        _ => None,
    };

    let on_complete = match on_complete.as_ref() {
        "crate" | "noop" | "call" => ApplicationCallOnComplete::NoOp,
        "update" => ApplicationCallOnComplete::UpdateApplication,
        "optin" => ApplicationCallOnComplete::OptIn,
        "clear" => ApplicationCallOnComplete::ClearState,
        "closeout" => ApplicationCallOnComplete::CloseOut,
        "delete" => ApplicationCallOnComplete::DeleteApplication,
        _ => panic!("invalid onComplete value"),
    };

    let use_account = match account_type.as_ref() {
        "transient" => transient_account,
        _ => panic!("Not handled account string: {}", account_type),
    };

    let approval = match approval_program {
        Some(p) => Some(read_teal(algod, &p).await),
        None => None,
    };

    let clear = match clear_program {
        Some(p) => Some(read_teal(algod, &p).await),
        None => None,
    };

    // populate args from methodArgs

    if abi_method_args.len() != abi_method.args.len() {
        panic!(
            "Provided argument count is incorrect. Expected {}, got {}",
            abi_method_args.len(),
            abi_method.args.len()
        );
    };

    let note_opt = if use_nonce {
        Some(
            w.note
                .clone()
                .expect("note should be set if using use_nonce"),
        )
    } else {
        None
    };

    let mut params = AddMethodCallParams {
        app_id: application_id,
        method: abi_method.to_owned(),
        method_args: abi_method_args.to_owned(),
        fee: MicroAlgos(tx_params.min_fee),
        sender: use_account.address(),
        suggested_params: tx_params,
        on_complete,
        approval_program: approval,
        clear_program: clear,
        global_schema,
        local_schema,
        extra_pages,
        note: note_opt,
        lease: None,
        rekey_to: None,
        signer: tx_signer,
    };

    tx_composer_methods.push(abi_method.to_owned());

    tx_composer.add_method_call(&mut params).unwrap();
}

#[given(regex = r#"I add the nonce "([^"]*)"$"#)]
fn i_add_the_nonce(w: &mut World, nonce: String) {
    w.note = Some(
        format!("I should be unique thanks to this nonce: {nonce}")
            .as_bytes()
            .to_vec(),
    );
}

#[given(expr = "I append the current transaction with signer to the method arguments array.")]
#[when(expr = "I append the current transaction with signer to the method arguments array.")]
fn i_append_the_current_transaction_with_signer_to_the_method_arguments_array(w: &mut World) {
    let method_args = w.abi_method_arg_values.as_mut().expect("no method args");
    let tx_with_signer = w.tx_with_signer.clone().expect("no tx signer");

    method_args.push(AbiArgValue::TxWithSigner(tx_with_signer));
}

#[when(
    regex = r#"^I build the transaction group with the composer. If there is an error it is "([^"]*)".$"#
)]
#[then(
    regex = r#"^I build the transaction group with the composer. If there is an error it is "([^"]*)".$"#
)]
#[given(
    regex = r#"^I build the transaction group with the composer. If there is an error it is "([^"]*)".$"#
)]
fn i_build_the_transaction_group_with_the_composer(w: &mut World, error_type: String) {
    let tx_composer = w.tx_composer.as_mut().unwrap();

    let build_res = tx_composer.build_group();

    match error_type.as_ref() {
        "" => {
            // no error expected
            build_res.unwrap();
        }
        "zero group size error" => {
            let message = match build_res {
                Ok(_) => None,
                Err(e) => match e {
                    ServiceError::Msg(m) => Some(m),
                    _ => None,
                },
            };

            match message.as_deref() {
                Some("attempting to build group with zero transactions") => {}
                _ => panic!("expected error, but got: {:?}", message),
            }
        }
        _ => panic!("Unknown error type: {}", error_type),
    }
}

#[then(expr = "I execute the current transaction group with the composer.")]
async fn i_execute_the_current_transaction_group_with_the_composer(w: &mut World) {
    let algod = w.algod.as_ref().unwrap();
    let tx_composer = w.tx_composer.as_mut().unwrap();

    let res = tx_composer.execute(algod).await;

    let res = res.expect("Failed executing");

    w.tx_composer_res = Some(res)
}

#[then(regex = r#"^The app should have returned "([^"]*)"\.$"#)]
async fn the_app_should_have_returned(w: &mut World, comma_separated_b64_results: String) {
    let tx_composer_res = w.tx_composer_res.as_ref().unwrap();
    let tx_composer_methods = w.tx_composer_methods.as_ref().unwrap();

    let b64_expected_results: Vec<&str> = comma_separated_b64_results.split(',').collect();

    if b64_expected_results.len() != tx_composer_res.method_results.len() {
        panic!(
            "length of expected results doesn't match actual: {:?} != {}",
            b64_expected_results,
            tx_composer_res.method_results.len()
        );
    }

    if tx_composer_methods.len() != tx_composer_res.method_results.len() {
        panic!(
            "length of composer's methods doesn't match results: {:?} != {}",
            tx_composer_methods,
            tx_composer_res.method_results.len()
        );
    }

    for (i, b64_expected_result) in b64_expected_results.into_iter().enumerate() {
        let expected_res_bytes = BASE64
            .decode(b64_expected_result.as_bytes())
            .expect("couldn't decode b64");
        match &tx_composer_res.method_results[i].return_value {
            Ok(AbiMethodReturnValue::Some(value)) => {
                let mut method = tx_composer_methods[i].clone();
                match method.returns.type_().expect("error retrieving type") {
                    AbiReturnType::Some(type_) => {
                        let expected_value = type_
                            .decode(&expected_res_bytes)
                            .expect("the expected value doesn't match the actual result");

                        assert_eq!(&expected_value, value);
                    }
                    AbiReturnType::Void => panic!("unexpected void return type"),
                }
            }
            Ok(AbiMethodReturnValue::Void) => {
                if !expected_res_bytes.is_empty() {
                    panic!("Expected result should be empty")
                }
            }
            Err(AbiReturnDecodeError(e)) => panic!("decode error: {:?}", e),
        }
    }
}

#[then(regex = r#"^The app should have returned ABI types "([^"]*)"\.$"#)]
async fn the_app_should_have_returned_abi_types(w: &mut World, expected_type_strings_str: String) {
    let tx_composer_res = w.tx_composer_res.as_ref().unwrap();

    let expected_type_strings: Vec<&str> = expected_type_strings_str.split(':').collect();

    if expected_type_strings.len() != tx_composer_res.method_results.len() {
        panic!(
            "length of expected results doesn't match actual: {} != {}",
            expected_type_strings.len(),
            tx_composer_res.method_results.len()
        );
    }

    for (i, expected_type_string) in expected_type_strings.into_iter().enumerate() {
        let actual_res = &tx_composer_res.method_results[i];

        match &actual_res.return_value {
            Ok(AbiMethodReturnValue::Some(value)) => {
                let expected_type = expected_type_string.parse::<AbiType>().unwrap();

                let encoded = expected_type
                    .encode(value.clone())
                    .expect("couldn't encode value");

                let decoded = expected_type
                    .decode(&encoded)
                    .expect("couldn't decode value");

                assert_eq!(
                    &decoded, value,
                    "The round trip result does not match the original result"
                )
            }
            Ok(AbiMethodReturnValue::Void) => {
                if !AbiReturn::is_void_str(expected_type_string) {
                    panic!("Not a void return type:  {:?}", actual_res.return_value);
                }
            }
            Err(e) => {
                panic!("Decode error: {:?}", e)
            }
        }
    }
}

// The 1th atomic result for randomInt(1337) proves correct
// #[then(regex = r#"^The (\d+)th atomic result for randomInt\((\d+)\) proves correct$"#)]
#[then(regex = r#"^The (\d+)th atomic result for randomInt\((\d+)\) proves correct$"#)]
async fn check_random_int_result(w: &mut World, result_index: usize, input: u64) {
    let tx_composers = w.tx_composer_res.as_ref().expect("No tx composer res");
    let tx_composer_res = &tx_composers.method_results[result_index];

    let value = match &tx_composer_res.return_value {
        Ok(AbiMethodReturnValue::Some(value)) => value,
        _ => panic!("No decoded res"),
    };

    let (rand_int, witness) = match value {
        AbiValue::Array(array) => match &array[0] {
            AbiValue::Int(i) => match &array[1] {
                AbiValue::Array(nested_array) => (
                    i.to_u64().expect("couldn't convert bigint to int"),
                    nested_array.clone(),
                ),
                _ => panic!("nested abi value isn't an array"),
            },
            _ => panic!("nested abi value isn't an int"),
        },
        _ => panic!("abi value isn't an array"),
    };

    let mut witness_bytes = vec![];
    for (_, value) in witness.into_iter().enumerate() {
        witness_bytes.push(match value {
            AbiValue::Byte(b) => b,
            _ => panic!("abi value isn't a byte"),
        });
    }

    let x = sha2::Sha512_256::digest(&witness_bytes);
    let int = u64::from_be_bytes(x[..8].try_into().expect("couldn't get slice from hash"));
    let quotient = int % input as u64;
    if quotient != rand_int {
        panic!(
            "Unexpected result: quotient is {} and randInt is {}",
            quotient, rand_int
        );
    }
}

#[then(regex = r#"^The (\d+)th atomic result for randElement\("([^"]*)"\) proves correct$"#)]
async fn check_random_element_result(w: &mut World, result_index: usize, input: String) {
    let tx_composers = w.tx_composer_res.as_ref().expect("No tx composer res");
    let tx_composer_res = &tx_composers.method_results[result_index];

    let value = match &tx_composer_res.return_value {
        Ok(value) => match value {
            AbiMethodReturnValue::Some(value) => value,
            AbiMethodReturnValue::Void => panic!("No decoded res"),
        },
        _ => panic!("No decoded res"),
    };

    let (rand_el, witness) = match value {
        AbiValue::Array(array) => match &array[0] {
            AbiValue::Byte(b) => match &array[1] {
                AbiValue::Array(nested_array) => (b.clone(), nested_array.clone()),
                _ => panic!("nested abi value isn't an array"),
            },
            _ => panic!("nested abi value isn't an byte"),
        },
        _ => panic!("abi value isn't an array"),
    };

    let mut witness_bytes = vec![];
    for (_, value) in witness.into_iter().enumerate() {
        witness_bytes.push(match value {
            AbiValue::Byte(b) => b,
            _ => panic!(),
        });
    }

    let x = sha2::Sha512_256::digest(&witness_bytes);
    let int = usize::from_be_bytes(x[..8].try_into().expect("couldn't get slice from hash"));
    let quotient = int % input.len();
    if input.as_bytes()[quotient] != rand_el {
        panic!(
            "Unexpected result: quotient is {} and randInt is {}",
            quotient, rand_el
        );
    }
}

#[then(
    regex = r#"^I dig into the paths "([^"]*)" of the resulting atomic transaction tree I see group ids and they are all the same$"#
)]
async fn check_inner_txn_group_ids(w: &mut World, colon_separated_paths_string: String) {
    let tx_composer_res = w.tx_composer_res.as_ref().expect("No tx composer res");

    let mut paths: Vec<Vec<usize>> = vec![];

    let comma_separated_path_strings = colon_separated_paths_string.split(':');
    for comma_separated_path_string in comma_separated_path_strings {
        let path_of_strings = comma_separated_path_string.split(',');
        let mut path = vec![];
        for string_component in path_of_strings {
            let int_component = string_component.parse().unwrap();
            path.push(int_component)
        }
        paths.push(path)
    }

    let mut tx_infos_to_check = vec![];

    for path in paths {
        let mut current: PendingTransactionResponse =
            tx_composer_res.method_results[0].tx_info.clone();
        for path_index in 1..path.len() {
            let inner_txn_index = path[path_index];
            if path_index == 0 {
                current = tx_composer_res.method_results[inner_txn_index]
                    .tx_info
                    .clone();
            } else {
                current = current
                    .inner_txns
                    .unwrap()
                    .get(inner_txn_index)
                    .unwrap()
                    .clone();
            }
        }

        tx_infos_to_check.push(current);
    }

    // TODO https://github.com/manuelmauro/algonaut/issues/156
    // let mut group;
    // for (i, txInfo) in txInfosToCheck.into_iter().enumerate() {
    // if i == 0 {
    //     group = txInfo.txn.group
    // }
    // if group != txInfo.txn.group {
    //     panic!("Group hashes differ: {} != {}", group, txInfo.txn.group);
    // }
    // }
}

#[then(
    regex = r#"^I can dig the (\d+)th atomic result with path "([^"]*)" and see the value "([^"]*)"$"#
)]
async fn check_atomic_result_against_value(
    _w: &mut World,
    _result_index: u64,
    _path: String,
    _expected_value: String,
) {

    // TODO https://github.com/manuelmauro/algonaut/issues/156
}

#[given(regex = r#"^an application id (\d+)$"#)]
async fn an_application_id(w: &mut World, app_id: u64) {
    w.app_id = Some(app_id);
}

#[then(regex = r#"^The (\d+)th atomic result for "([^"]*)" satisfies the regex "([^"]*)"$"#)]
async fn check_spin_result(w: &mut World, result_index: usize, method: String, r: String) {
    let tx_composer_res = w.tx_composer_res.as_ref().expect("No tx composer res");

    if method != "spin()" {
        panic!("Incorrect method name, expected 'spin()', got '{}'", method);
    }

    let result = &tx_composer_res.method_results[result_index];

    let decoded_result = match &result.return_value {
        Ok(AbiMethodReturnValue::Some(value)) => match value {
            AbiValue::Array(array) => array,
            _ => panic!("return value isn't an array"),
        },
        _ => panic!("unexpected return value: {:?}", result.return_value),
    };

    let spin = match &decoded_result[0] {
        AbiValue::Array(array) => array,
        _ => panic!("first spin element isn't an array"),
    };

    let mut spin_bytes = vec![];
    for value in spin {
        spin_bytes.push(match value {
            AbiValue::Byte(b) => *b,
            _ => panic!("non-byte in spin array"),
        });
    }

    let regex: Regex = Regex::new(&r).expect(&format!("couldn't create regex for: {}", r));
    let str = String::from_utf8(spin_bytes).expect("couldn't convert bytes to string");
    let matched = regex.is_match(&str);

    if !matched {
        panic!("Result did not match regex. spin str: {}", str);
    }
}

#[given(regex = r#"^I fund the current application's address with (\d+) microalgos\.$"#)]
async fn i_fund_the_current_applications_address(w: &mut World, micro_algos: u64) {
    let algod = w.algod.as_ref().expect("no algod");
    let app_id = w.app_id.expect("no app id");
    let accounts = w.accounts.as_ref().expect("no accounts");
    let kmd = w.kmd.as_ref().expect("no kmd");
    let kmd_handle = w.handle.as_ref().expect("no kmd handle");
    let kmd_pw = w.password.as_ref().expect("no kmd pw");

    let first_account = accounts[0];

    let app_address = to_app_address(app_id);

    let tx_params = algod
        .transaction_params()
        .await
        .expect("couldn't get params");

    let tx = TxnBuilder::with(
        &tx_params,
        Pay::new(first_account, app_address, MicroAlgos(micro_algos)).build(),
    )
    .build()
    .unwrap();

    let signed_tx = kmd
        .sign_transaction(kmd_handle, kmd_pw, &tx)
        .await
        .expect("couldn't sign tx");

    let res = algod
        .raw_transaction(&signed_tx.signed_transaction)
        .await
        .expect("couldn't send tx");

    let _ = wait_for_pending_transaction(algod, &res.tx_id);
}

#[given(regex = r#"^I reset the array of application IDs to remember\.$"#)]
async fn i_reset_the_array_of_application_ids_to_remember(w: &mut World) {
    w.app_ids = vec![];
}
