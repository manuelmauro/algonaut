use std::{
    convert::TryInto,
    error::Error,
    fs,
    num::ParseIntError,
    time::{Duration, Instant},
};

use algonaut::algod::v2::Algod;
use algonaut_algod::models::PendingTransactionResponse;
use algonaut_core::{Address, CompiledTeal};
use algonaut_model::kmd::v1::ExportKeyResponse;
use algonaut_transaction::account::Account;

/// Utility function to wait on a transaction to be confirmed
pub async fn wait_for_pending_transaction(
    algod: &Algod,
    txid: &str,
) -> Result<Option<PendingTransactionResponse>, algonaut::Error> {
    let timeout = Duration::from_secs(10);
    let start = Instant::now();
    loop {
        let pending_transaction = algod.pending_txn(txid).await?;
        // If the transaction has been confirmed or we time out, exit.
        if pending_transaction.confirmed_round.is_some() {
            return Ok(Some(pending_transaction));
        } else if start.elapsed() >= timeout {
            return Ok(None);
        }
        std::thread::sleep(Duration::from_millis(250))
    }
}

pub fn split_uint64(args_str: &str) -> Result<Vec<u64>, ParseIntError> {
    if args_str.is_empty() {
        return Ok(vec![]);
    }
    args_str.split(",").map(|a| a.parse()).collect()
}

pub fn split_addresses(args_str: String) -> Result<Vec<Address>, String> {
    if args_str.is_empty() {
        return Ok(vec![]);
    }
    args_str.split(",").map(|a| a.parse()).collect()
}

pub fn parse_app_args(args_str: String) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
    if args_str.is_empty() {
        return Ok(vec![]);
    }

    let args = args_str.split(",");

    let mut args_bytes: Vec<Vec<u8>> = vec![];
    for arg in args {
        let parts = arg.split(":").collect::<Vec<&str>>();
        let type_part = parts[0];
        match type_part {
            "str" => args_bytes.push(parts[1].as_bytes().to_vec()),
            "int" => {
                let int = parts[1].parse::<u64>()?;
                args_bytes.push(int.to_be_bytes().to_vec());
            }
            _ => Err(format!(
                "Applications doesn't currently support argument of type {}",
                type_part
            ))?,
        }
    }

    Ok(args_bytes)
}

pub fn account_from_kmd_response(key_res: &ExportKeyResponse) -> Result<Account, Box<dyn Error>> {
    Ok(Account::from_seed(key_res.private_key[0..32].try_into()?))
}

pub async fn read_teal(algod: &Algod, file_name: &str) -> CompiledTeal {
    let file_bytes = fs::read(&format!("tests/features/resources/{file_name}")).unwrap();

    if file_name.ends_with(".teal") {
        algod.teal_compile(&file_bytes, None).await.unwrap()
    } else {
        CompiledTeal(file_bytes)
    }
}
