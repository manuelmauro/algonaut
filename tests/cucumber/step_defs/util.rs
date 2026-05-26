use std::{convert::TryInto, error::Error, fs, num::ParseIntError};

use algonaut::algod::v2::{Algod, SourceMap};
use algonaut_core::{Address, CompiledTeal};
use algonaut_model::kmd::v1::ExportKeyResponse;
use algonaut_transaction::account::Account;

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
        algod
            .teal_compile(&file_bytes, SourceMap::Skip)
            .await
            .unwrap()
    } else {
        CompiledTeal(file_bytes)
    }
}
