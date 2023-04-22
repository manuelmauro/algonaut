use crate::{algod::v2::Algod, error::ServiceError};
use algonaut_algod::models::{
    Application, ApplicationParams, ApplicationStateSchema, DryrunRequest, DryrunState,
    DryrunTxnResult, TealValue,
};
use algonaut_core::{to_app_address, Address};
use algonaut_encoding::Bytes;
use algonaut_transaction::{
    transaction::{ApplicationCallTransaction, StateSchema},
    SignedTransaction, TransactionType,
};
use data_encoding::{DecodeError, HEXLOWER};
use std::{collections::HashSet, convert::TryInto};

const DEFAULT_APP_ID: u64 = 1380011588;
const DEFAULT_MAX_WIDTH: usize = 30;

pub async fn create_dryrun(
    algod: &Algod,
    signed_txs: &[SignedTransaction],
) -> Result<DryrunRequest, ServiceError> {
    create_dryrun_with_settings(algod, signed_txs, "", 0, 0).await
}

pub async fn create_dryrun_with_settings(
    algod: &Algod,
    signed_txs: &[SignedTransaction],
    protocol_version: &str,
    latest_timestamp: u64,
    round: u64,
) -> Result<DryrunRequest, ServiceError> {
    if signed_txs.is_empty() {
        return Err(ServiceError::Msg("No txs".to_owned()));
    }

    // The details we need to add to DryrunRequest object
    let mut app_infos = vec![];
    let mut acct_infos = vec![];

    // These are populated from the transactions passed
    let mut apps = HashSet::new();
    let mut assets = HashSet::new();
    let mut accts = HashSet::new();

    for signed_tx in signed_txs {
        let tx = &signed_tx.transaction;

        if let TransactionType::ApplicationCallTransaction(app_call) = &tx.txn_type {
            if let Some(app_id) = app_call.app_id {
                apps.insert(app_id);
                accts.insert(to_app_address(app_id));
            } else {
                // Prepare and set param fields for Application being created
                app_infos.push(to_application(app_call, &tx.sender()))
            }

            if let Some(foreign_apps) = &app_call.foreign_apps {
                apps.extend(foreign_apps);
            }
            if let Some(foreign_assets) = &app_call.foreign_assets {
                assets.extend(foreign_assets);
            }
            if let Some(accounts) = &app_call.accounts {
                accts.extend(accounts);
            }
        }
        // No other tx types - we're only interested to pull state for app calls
    }

    for asset_id in assets {
        let asset = algod.get_asset_by_id(asset_id).await?;
        accts.insert(asset.params.creator.parse().unwrap());
    }

    for app_id in apps {
        let app = algod.get_application_by_id(app_id).await?;
        accts.insert(app.params.creator.parse().unwrap());
        app_infos.push(app);
    }

    for address in accts {
        let acc = algod
            .account_information(&address.to_string().as_str())
            .await?;
        acct_infos.push(acc);
    }

    Ok(DryrunRequest {
        accounts: acct_infos,
        apps: app_infos,
        latest_timestamp,
        protocol_version: protocol_version.to_owned(),
        round,
        sources: vec![],
        txns: signed_txs.iter().map(|t| t.clone().into()).collect(),
    })
}

fn to_application(app_call: &ApplicationCallTransaction, sender: &Address) -> Application {
    let params = ApplicationParams {
        approval_program: Bytes(
            app_call
                .approval_program
                .clone()
                .map(|p| p.0)
                .unwrap_or_default(),
        ),
        clear_state_program: Bytes(
            app_call
                .clear_state_program
                .clone()
                .map(|p| p.0)
                .unwrap_or_default(),
        ),
        creator: (*sender).to_string(),
        global_state: Some(vec![]),
        global_state_schema: app_call
            .global_state_schema
            .clone()
            .map(to_application_state_schema)
            .map(|o| Box::new(o)),
        local_state_schema: app_call
            .local_state_schema
            .clone()
            .map(to_application_state_schema)
            .map(|o| Box::new(o)),
        // TODO add this
        extra_program_pages: None,
    };

    Application {
        id: DEFAULT_APP_ID,
        params: Box::new(params),
    }
}

/// StackPrinterConfig contains configuration parameters for
/// printing the trace from a DryrunTxnResult.
#[derive(Debug, Clone)]
pub struct StackPrinterConfig {
    pub max_column_widths: MaxColumnWidths,
    pub top_of_stack_first: bool,
    pub bytes_format: BytesFormat,
}

#[derive(Debug, Clone)]
pub enum BytesFormat {
    /// Displays byte values as hex
    Hex,
    /// Tries to decode byte values as addresses, if it fails, uses hex
    AddressOrHex,
}

#[derive(Debug, Clone)]
pub struct MaxColumnWidths {
    pub source: usize,
    pub scratch: usize,
    pub stack: usize,
}

impl Default for MaxColumnWidths {
    fn default() -> Self {
        Self {
            source: DEFAULT_MAX_WIDTH,
            scratch: DEFAULT_MAX_WIDTH,
            stack: DEFAULT_MAX_WIDTH,
        }
    }
}

impl Default for StackPrinterConfig {
    fn default() -> Self {
        Self {
            max_column_widths: MaxColumnWidths::default(),
            top_of_stack_first: false,
            bytes_format: BytesFormat::Hex,
        }
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    match s.char_indices().nth(max_len) {
        None => s.to_owned(),
        Some((index, _)) => {
            format!("{}...", &s[..index])
        }
    }
}

fn stack_to_str(stack: &[TealValue], bytes_format: &BytesFormat) -> Result<String, ServiceError> {
    let mut elems = vec![];
    for value in stack {
        match value.value_type {
            1 => elems.push(bytes_to_str(&value.bytes, bytes_format)),
            2 => elems.push(value.uint.to_string()),
            _ => {}
        }
    }

    Ok(format!("[{}]", elems.join(", ")))
}

fn bytes_to_str(bytes: &[u8], format: &BytesFormat) -> String {
    match format {
        BytesFormat::Hex => to_hex_str(bytes),
        BytesFormat::AddressOrHex => bytes
            .try_into()
            .map(|array| Address(array).to_string())
            .unwrap_or_else(|_| to_hex_str(bytes)),
    }
}

fn scratch_to_str(
    prev_scratch: &[TealValue],
    cur_scratch: &[TealValue],
    bytes_format: &BytesFormat,
) -> Result<String, ServiceError> {
    if cur_scratch.is_empty() {
        return Ok("".to_owned());
    }

    let mut new_index = None;
    for i in 0..cur_scratch.len() {
        if i >= prev_scratch.len() {
            new_index = Some(i)
        }
        if prev_scratch[i] != cur_scratch[i] {
            new_index = Some(i);
        }
    }

    Ok(if let Some(new_index) = new_index {
        let value = &cur_scratch[new_index];
        if !value.bytes.is_empty() {
            let str = bytes_to_str(&value.bytes, bytes_format);
            format!("{} = {}", new_index, str)
        } else {
            format!("{} = {}", new_index, value.uint)
        }
    } else {
        "".to_owned()
    })
}

fn trace(
    state: &[DryrunState],
    disassembly: &[String],
    config: &StackPrinterConfig,
) -> Result<String, ServiceError> {
    let mut lines = vec![vec![
        "pc#".to_owned(),
        "ln#".to_owned(),
        "source".to_owned(),
        "scratch".to_owned(),
        "stack".to_owned(),
    ]];

    // Create lines for trace
    for (i, s) in state.iter().enumerate() {
        let src = if let Some(error) = &s.error {
            format!("!! {} !!", error)
        } else {
            disassembly[s.line as usize].clone()
        };

        let cur_scratch = &s.scratch;
        let prev_scratch = if i > 0 {
            state[i - 1].clone().scratch.unwrap()
        } else {
            vec![]
        };

        let mut stack = s.stack.clone();
        if config.top_of_stack_first {
            stack.reverse()
        };

        lines.push(vec![
            format!("{:3}", s.pc.to_string()),
            format!("{:3}", s.line.to_string()),
            truncate(&src, config.max_column_widths.source),
            truncate(
                &scratch_to_str(
                    &prev_scratch,
                    &cur_scratch.clone().unwrap()[..],
                    &config.bytes_format,
                )?,
                config.max_column_widths.scratch,
            ),
            truncate(
                &stack_to_str(&stack, &config.bytes_format)?,
                config.max_column_widths.stack,
            ),
        ]);
    }

    // Get max length of each column
    let columns = lines[0].len();
    let mut max_lens = vec![0; columns];
    for line in &lines {
        for j in 0..columns {
            if line[j].len() > max_lens[j] {
                max_lens[j] = line[j].len();
            }
        }
    }

    Ok(lines
        .iter()
        .map(|line| to_line_str(line, &max_lens))
        .collect::<Vec<_>>()
        .join("\n"))
}

fn to_line_str(line: &[String], max_lens: &[usize]) -> String {
    line.iter()
        .enumerate()
        .map(|(i, w)| pad(w, max_lens[i]))
        .collect::<Vec<_>>()
        .join(" | ")
}

fn pad(s: &str, len: usize) -> String {
    format!("{s}{}", str::repeat(" ", len - s.len()))
}

pub fn app_trace(dryrun_res: &DryrunTxnResult) -> Result<String, ServiceError> {
    trace(
        &dryrun_res.app_call_trace.clone().unwrap(),
        &dryrun_res.disassembly,
        &StackPrinterConfig::default(),
    )
}

pub fn app_trace_with_config(
    dryrun_res: &DryrunTxnResult,
    config: &StackPrinterConfig,
) -> Result<String, ServiceError> {
    trace(
        &dryrun_res.app_call_trace.clone().unwrap(),
        &dryrun_res.disassembly,
        config,
    )
}

pub fn lsig_trace(dryrun_res: &DryrunTxnResult) -> Result<String, ServiceError> {
    lsig_trace_with_config(dryrun_res, &StackPrinterConfig::default())
}

pub fn lsig_trace_with_config(
    dryrun_res: &DryrunTxnResult,
    config: &StackPrinterConfig,
) -> Result<String, ServiceError> {
    trace(
        &dryrun_res.logic_sig_trace.clone().unwrap(),
        &dryrun_res.disassembly,
        config,
    )
}

fn to_hex_str(bytes: &[u8]) -> String {
    format!("0x{}", HEXLOWER.encode(bytes))
}

fn to_application_state_schema(schema: StateSchema) -> ApplicationStateSchema {
    ApplicationStateSchema {
        num_byte_slice: schema.number_byteslices,
        num_uint: schema.number_ints,
    }
}

impl From<DecodeError> for ServiceError {
    fn from(e: DecodeError) -> Self {
        ServiceError::Msg(format!("Decoding error: {e}"))
    }
}
