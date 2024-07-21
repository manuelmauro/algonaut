use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionHeader {
    pub hgi: Option<bool>,
    pub sig: Option<String>,
    pub msig: Option<String>,
    pub lsig: Option<String>,

    #[serde(flatten)]
    pub apply_data: Option<ApplyData>,
    pub txn: Option<Transaction>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Transaction {
    #[serde(rename = "pay")]
    Payment {
        #[serde(rename = "fee")]
        fee: Option<u64>,
        #[serde(rename = "fv")]
        first_valid: Option<u64>,
        #[serde(rename = "gh")]
        genesis_hash: Option<String>,
        #[serde(rename = "lv")]
        last_valid: Option<u64>,
        #[serde(rename = "snd")]
        sender: Option<String>,
        #[serde(rename = "gen")]
        genesis_id: Option<String>,
        #[serde(rename = "grp")]
        group: Option<String>,
        #[serde(rename = "lx")]
        lease: Option<String>,
        #[serde(rename = "note")]
        note: Option<String>,
        #[serde(rename = "rekey")]
        rekey: Option<String>,
        // type specific fields
        #[serde(rename = "rcv")]
        receiver: Option<String>,
        #[serde(rename = "amt")]
        amount: Option<u64>,
        #[serde(rename = "close")]
        close_remainder_to: Option<String>,
    },
    #[serde(rename = "keyreg")]
    KeyRegistration {
        #[serde(rename = "fee")]
        fee: Option<u64>,
        #[serde(rename = "fv")]
        first_valid: Option<u64>,
        #[serde(rename = "gh")]
        genesis_hash: Option<String>,
        #[serde(rename = "lv")]
        last_valid: Option<u64>,
        #[serde(rename = "snd")]
        sender: Option<String>,
        #[serde(rename = "gen")]
        genesis_id: Option<String>,
        #[serde(rename = "grp")]
        group: Option<String>,
        #[serde(rename = "lx")]
        lease: Option<String>,
        #[serde(rename = "note")]
        note: Option<String>,
        #[serde(rename = "rekey")]
        rekey: Option<String>,
        // type specific fields
        #[serde(rename = "votekey")]
        vote_pk: Option<String>,
        #[serde(rename = "selkey")]
        selection_pk: Option<String>,
        #[serde(rename = "sprfkey")]
        state_proof_pk: Option<String>,
        #[serde(rename = "votefst")]
        vote_first: Option<u64>,
        #[serde(rename = "votelst")]
        vote_last: Option<u64>,
        #[serde(rename = "votekd")]
        vote_key_dilution: Option<u64>,
        #[serde(rename = "nonpart")]
        nonparticipating: Option<bool>,
    },
    #[serde(rename = "acfg")]
    AssetConfig {
        #[serde(rename = "fee")]
        fee: Option<u64>,
        #[serde(rename = "fv")]
        first_valid: Option<u64>,
        #[serde(rename = "gh")]
        genesis_hash: Option<String>,
        #[serde(rename = "lv")]
        last_valid: Option<u64>,
        #[serde(rename = "snd")]
        sender: Option<String>,
        #[serde(rename = "gen")]
        genesis_id: Option<String>,
        #[serde(rename = "grp")]
        group: Option<String>,
        #[serde(rename = "lx")]
        lease: Option<String>,
        #[serde(rename = "note")]
        note: Option<String>,
        #[serde(rename = "rekey")]
        rekey: Option<String>,
        // type specific fields
        #[serde(rename = "caid")]
        config_asset: Option<u64>,
        #[serde(rename = "apar")]
        params: Option<AssetParams>,
    },
    #[serde(rename = "axfer")]
    AssetTransfer {
        #[serde(rename = "fee")]
        fee: Option<u64>,
        #[serde(rename = "fv")]
        first_valid: Option<u64>,
        #[serde(rename = "gh")]
        genesis_hash: Option<String>,
        #[serde(rename = "lv")]
        last_valid: Option<u64>,
        #[serde(rename = "snd")]
        sender: Option<String>,
        #[serde(rename = "gen")]
        genesis_id: Option<String>,
        #[serde(rename = "grp")]
        group: Option<String>,
        #[serde(rename = "lx")]
        lease: Option<String>,
        #[serde(rename = "note")]
        note: Option<String>,
        #[serde(rename = "rekey")]
        rekey: Option<String>,
        // type specific fields
        #[serde(rename = "xaid")]
        asset_xfer: Option<u64>,
        #[serde(rename = "aamt")]
        asset_amount: Option<u64>,
        #[serde(rename = "asnd")]
        asset_sender: Option<String>,
        #[serde(rename = "arcv")]
        asset_receiver: Option<String>,
        #[serde(rename = "close")]
        asset_close_remainder_to: Option<String>,
    },
    #[serde(rename = "afrz")]
    AssetFreeze {
        #[serde(rename = "fee")]
        fee: Option<u64>,
        #[serde(rename = "fv")]
        first_valid: Option<u64>,
        #[serde(rename = "gh")]
        genesis_hash: Option<String>,
        #[serde(rename = "lv")]
        last_valid: Option<u64>,
        #[serde(rename = "snd")]
        sender: Option<String>,
        #[serde(rename = "gen")]
        genesis_id: Option<String>,
        #[serde(rename = "grp")]
        group: Option<String>,
        #[serde(rename = "lx")]
        lease: Option<String>,
        #[serde(rename = "note")]
        note: Option<String>,
        #[serde(rename = "rekey")]
        rekey: Option<String>,
        // type specific fields
        #[serde(rename = "fadd")]
        freeze_account: Option<String>,
        #[serde(rename = "faid")]
        asset_id: Option<u64>,
        #[serde(rename = "ffrz")]
        frozen: Option<bool>,
    },
    #[serde(rename = "appl")]
    Application {
        #[serde(rename = "fee")]
        fee: Option<u64>,
        #[serde(rename = "fv")]
        first_valid: Option<u64>,
        #[serde(rename = "gh")]
        genesis_hash: Option<String>,
        #[serde(rename = "lv")]
        last_valid: Option<u64>,
        #[serde(rename = "snd")]
        sender: Option<String>,
        #[serde(rename = "gen")]
        genesis_id: Option<String>,
        #[serde(rename = "grp")]
        group: Option<String>,
        #[serde(rename = "lx")]
        lease: Option<String>,
        #[serde(rename = "note")]
        note: Option<String>,
        #[serde(rename = "rekey")]
        rekey: Option<String>,
        // type specific fields
        #[serde(rename = "apid")]
        app_id: Option<u64>,
        #[serde(rename = "apan")]
        on_complete: Option<u64>,
        #[serde(rename = "apat")]
        accounts: Option<Vec<String>>,
        #[serde(rename = "apap")]
        approval_program: Option<String>,
        #[serde(rename = "apaa")]
        app_arguments: Option<Vec<String>>,
        #[serde(rename = "apsu")]
        clear_state_program: Option<String>,
        #[serde(rename = "apfa")]
        foreign_apps: Option<Vec<u64>>,
        #[serde(rename = "apas")]
        foreign_assets: Option<Vec<u64>>,
        #[serde(rename = "apgs")]
        global_state_schema: Option<StateSchema>,
        #[serde(rename = "apls")]
        local_state_schema: Option<StateSchema>,
        #[serde(rename = "apep")]
        extra_program_pages: Option<u64>,
        #[serde(rename = "apbx")]
        boxes: Option<Vec<BoxReference>>,
    },
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct AssetParams {
    #[serde(rename = "am")]
    pub meta_data_hash: Option<Vec<u8>>,
    #[serde(rename = "an")]
    pub asset_name: Option<String>,
    #[serde(rename = "au")]
    pub url: Option<String>,
    #[serde(rename = "c")]
    pub clawback: Option<String>,
    #[serde(rename = "dc")]
    pub decimals: Option<u32>,
    #[serde(rename = "df")]
    pub default_frozen: Option<bool>,
    #[serde(rename = "f")]
    pub freeze: Option<String>,
    #[serde(rename = "m")]
    pub manager: Option<String>,
    #[serde(rename = "r")]
    pub reserve: Option<String>,
    #[serde(rename = "t")]
    pub total: Option<u64>,
    #[serde(rename = "un")]
    pub unit_name: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct BoxReference {
    #[serde(rename = "n")]
    name: String
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct StateSchema {
    #[serde(rename = "nui")]
    pub ints: Option<u64>,
    #[serde(rename = "nbs")]
    pub byte_slices: Option<u64>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum DeltaAction {
    SetBytesAction = 1,
    SetUintAction = 2,
    DeleteAction = 3,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct ValueDelta {
    #[serde(rename = "at")]
    pub action: Option<DeltaAction>,
    #[serde(rename = "bs")]
    pub bytes: Option<String>,
    #[serde(rename = "ui")]
    pub uint: Option<u64>,
}

type StateDelta = HashMap<String, ValueDelta>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EvalDelta {
    #[serde(rename = "gd")]
    pub global_delta: Option<StateDelta>,
    #[serde(rename = "ld")]
    pub local_deltas: Option<HashMap<String, StateDelta>>,
    #[serde(rename = "lg")]
    pub logs: Option<Vec<String>>,
    #[serde(rename = "itx")]
    pub inner_txns: Option<Vec<TransactionHeader>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApplyData {
    #[serde(rename = "dt")]
    pub delta: Option<EvalDelta>,
    #[serde(rename = "ca")]
    pub closing_amount: Option<u64>,
    #[serde(rename = "aca")]
    pub asset_closing_amount: Option<u64>
}
