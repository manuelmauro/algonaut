//! Hand-written algod block / pending-transaction models.
//!
//! These mirror the `SignedTxnInBlock` / `SignedTxn` shapes algod returns
//! inside `/v2/blocks/*` and `/v2/transactions/pending*`. Unlike the
//! OpenAPI-generated models they decode from **both** wire formats algod can
//! answer with:
//!
//! - **JSON** — addresses are base32-checksum strings, byte slices base64,
//!   the `type` discriminant a plain string.
//! - **msgpack** — addresses are raw 32-byte arrays, byte slices raw `bin`,
//!   the `type` discriminant a `bin` value.
//!
//! Address-typed fields use [`algonaut_core::Address`], which already
//! branches on `serializer.is_human_readable()`; binary fields use
//! [`Bytes`] and textual fields [`Text`]. The [`Transaction`] enum is
//! internally tagged on `type`, but serde's tag machinery insists the tag
//! be a string — which msgpack violates by sending it as `bin`.
//! `Transaction` therefore keeps the derived **tagged `Serialize`** but
//! hand-rolls `Deserialize` through a flat [`TxnFields`] struct whose
//! `type` field is a [`Text`] (string or msgpack `bin`).

use algonaut_core::Address;
use algonaut_encoding::{Bytes, Text};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct TransactionHeader {
    pub hgi: Option<bool>,
    pub sig: Option<Bytes>,
    pub msig: Option<MultiSig>,
    pub lsig: Option<LogicSig>,

    #[serde(flatten)]
    pub apply_data: Option<ApplyData>,
    pub txn: Option<Transaction>,
}

/// A logic-signature (`lsig`) attached to a signed transaction.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct LogicSig {
    /// The logic program bytecode.
    #[serde(rename = "l")]
    pub logic: Option<Bytes>,
    /// Program arguments.
    #[serde(rename = "arg")]
    pub args: Option<Vec<Bytes>>,
    /// Delegating account signature.
    #[serde(rename = "sig")]
    pub sig: Option<Bytes>,
    /// Delegating multisig signature.
    #[serde(rename = "msig")]
    pub msig: Option<MultiSig>,
}

/// A multi-signature (`msig`) attached to a signed transaction.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct MultiSig {
    #[serde(rename = "v")]
    pub version: Option<u8>,
    #[serde(rename = "thr")]
    pub threshold: Option<u8>,
    #[serde(rename = "subsig")]
    pub subsigs: Option<Vec<MultiSigSubsig>>,
}

/// One sub-signature within a [`MultiSig`].
#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct MultiSigSubsig {
    #[serde(rename = "pk")]
    pub public_key: Option<Bytes>,
    #[serde(rename = "s")]
    pub signature: Option<Bytes>,
}

#[derive(Debug, PartialEq, Serialize, Clone)]
#[serde(tag = "type")]
pub enum Transaction {
    #[serde(rename = "pay")]
    Payment {
        #[serde(rename = "fee")]
        fee: Option<u64>,
        #[serde(rename = "fv")]
        first_valid: Option<u64>,
        #[serde(rename = "gh")]
        genesis_hash: Option<Bytes>,
        #[serde(rename = "lv")]
        last_valid: Option<u64>,
        #[serde(rename = "snd")]
        sender: Option<Address>,
        #[serde(rename = "gen")]
        genesis_id: Option<Text>,
        #[serde(rename = "grp")]
        group: Option<Bytes>,
        #[serde(rename = "lx")]
        lease: Option<Bytes>,
        #[serde(rename = "note")]
        note: Option<Bytes>,
        #[serde(rename = "rekey")]
        rekey: Option<Address>,
        // type specific fields
        #[serde(rename = "rcv")]
        receiver: Option<Address>,
        #[serde(rename = "amt")]
        amount: Option<u64>,
        #[serde(rename = "close")]
        close_remainder_to: Option<Address>,
    },
    #[serde(rename = "keyreg")]
    KeyRegistration {
        #[serde(rename = "fee")]
        fee: Option<u64>,
        #[serde(rename = "fv")]
        first_valid: Option<u64>,
        #[serde(rename = "gh")]
        genesis_hash: Option<Bytes>,
        #[serde(rename = "lv")]
        last_valid: Option<u64>,
        #[serde(rename = "snd")]
        sender: Option<Address>,
        #[serde(rename = "gen")]
        genesis_id: Option<Text>,
        #[serde(rename = "grp")]
        group: Option<Bytes>,
        #[serde(rename = "lx")]
        lease: Option<Bytes>,
        #[serde(rename = "note")]
        note: Option<Bytes>,
        #[serde(rename = "rekey")]
        rekey: Option<Address>,
        // type specific fields
        #[serde(rename = "votekey")]
        vote_pk: Option<Bytes>,
        #[serde(rename = "selkey")]
        selection_pk: Option<Bytes>,
        #[serde(rename = "sprfkey")]
        state_proof_pk: Option<Bytes>,
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
        genesis_hash: Option<Bytes>,
        #[serde(rename = "lv")]
        last_valid: Option<u64>,
        #[serde(rename = "snd")]
        sender: Option<Address>,
        #[serde(rename = "gen")]
        genesis_id: Option<Text>,
        #[serde(rename = "grp")]
        group: Option<Bytes>,
        #[serde(rename = "lx")]
        lease: Option<Bytes>,
        #[serde(rename = "note")]
        note: Option<Bytes>,
        #[serde(rename = "rekey")]
        rekey: Option<Address>,
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
        genesis_hash: Option<Bytes>,
        #[serde(rename = "lv")]
        last_valid: Option<u64>,
        #[serde(rename = "snd")]
        sender: Option<Address>,
        #[serde(rename = "gen")]
        genesis_id: Option<Text>,
        #[serde(rename = "grp")]
        group: Option<Bytes>,
        #[serde(rename = "lx")]
        lease: Option<Bytes>,
        #[serde(rename = "note")]
        note: Option<Bytes>,
        #[serde(rename = "rekey")]
        rekey: Option<Address>,
        // type specific fields
        #[serde(rename = "xaid")]
        asset_xfer: Option<u64>,
        #[serde(rename = "aamt")]
        asset_amount: Option<u64>,
        #[serde(rename = "asnd")]
        asset_sender: Option<Address>,
        #[serde(rename = "arcv")]
        asset_receiver: Option<Address>,
        #[serde(rename = "close")]
        asset_close_remainder_to: Option<Address>,
    },
    #[serde(rename = "afrz")]
    AssetFreeze {
        #[serde(rename = "fee")]
        fee: Option<u64>,
        #[serde(rename = "fv")]
        first_valid: Option<u64>,
        #[serde(rename = "gh")]
        genesis_hash: Option<Bytes>,
        #[serde(rename = "lv")]
        last_valid: Option<u64>,
        #[serde(rename = "snd")]
        sender: Option<Address>,
        #[serde(rename = "gen")]
        genesis_id: Option<Text>,
        #[serde(rename = "grp")]
        group: Option<Bytes>,
        #[serde(rename = "lx")]
        lease: Option<Bytes>,
        #[serde(rename = "note")]
        note: Option<Bytes>,
        #[serde(rename = "rekey")]
        rekey: Option<Address>,
        // type specific fields
        #[serde(rename = "fadd")]
        freeze_account: Option<Address>,
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
        genesis_hash: Option<Bytes>,
        #[serde(rename = "lv")]
        last_valid: Option<u64>,
        #[serde(rename = "snd")]
        sender: Option<Address>,
        #[serde(rename = "gen")]
        genesis_id: Option<Text>,
        #[serde(rename = "grp")]
        group: Option<Bytes>,
        #[serde(rename = "lx")]
        lease: Option<Bytes>,
        #[serde(rename = "note")]
        note: Option<Bytes>,
        #[serde(rename = "rekey")]
        rekey: Option<Address>,
        // type specific fields
        #[serde(rename = "apid")]
        app_id: Option<u64>,
        #[serde(rename = "apan")]
        on_complete: Option<u64>,
        #[serde(rename = "apat")]
        accounts: Option<Vec<Address>>,
        #[serde(rename = "apap")]
        approval_program: Option<Bytes>,
        #[serde(rename = "apaa")]
        app_arguments: Option<Vec<Bytes>>,
        #[serde(rename = "apsu")]
        clear_state_program: Option<Bytes>,
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
    #[serde(rename = "hb")]
    Heartbeat {
        #[serde(rename = "fee")]
        fee: Option<u64>,
        #[serde(rename = "fv")]
        first_valid: Option<u64>,
        #[serde(rename = "gh")]
        genesis_hash: Option<Bytes>,
        #[serde(rename = "lv")]
        last_valid: Option<u64>,
        #[serde(rename = "snd")]
        sender: Option<Address>,
        #[serde(rename = "gen")]
        genesis_id: Option<Text>,
        #[serde(rename = "grp")]
        group: Option<Bytes>,
        #[serde(rename = "lx")]
        lease: Option<Bytes>,
        #[serde(rename = "note")]
        note: Option<Bytes>,
        #[serde(rename = "rekey")]
        rekey: Option<Address>,
        // type specific fields
        #[serde(rename = "hb")]
        heartbeat: Option<HeartbeatFields>,
    },
}

impl Transaction {
    /// The sender (`snd`) of the transaction, if present.
    pub fn sender(&self) -> Option<&Address> {
        match self {
            Transaction::Payment { sender, .. }
            | Transaction::KeyRegistration { sender, .. }
            | Transaction::AssetConfig { sender, .. }
            | Transaction::AssetTransfer { sender, .. }
            | Transaction::AssetFreeze { sender, .. }
            | Transaction::Application { sender, .. }
            | Transaction::Heartbeat { sender, .. } => sender.as_ref(),
        }
    }

    /// The heartbeat fields (`hb`) — `Some` only for a heartbeat transaction.
    pub fn heartbeat(&self) -> Option<&HeartbeatFields> {
        match self {
            Transaction::Heartbeat { heartbeat, .. } => heartbeat.as_ref(),
            _ => None,
        }
    }

    /// The heartbeat address (`hb.a`) — the account a heartbeat transaction
    /// proves onlineness for. `None` for any non-heartbeat transaction.
    pub fn heartbeat_address(&self) -> Option<&Address> {
        self.heartbeat().and_then(|hb| hb.address.as_ref())
    }
}

/// Flat union of every transaction field across all `Transaction` variants.
///
/// Used purely as the `Deserialize` landing zone for [`Transaction`]: the
/// `type` discriminant is read through [`deserialize_text`] (string *or*
/// msgpack `bin`), then [`Transaction::from`] dispatches on it.
#[derive(Debug, Default, Deserialize)]
struct TxnFields {
    #[serde(rename = "type")]
    txn_type: Text,
    // shared header fields
    #[serde(rename = "fee", default)]
    fee: Option<u64>,
    #[serde(rename = "fv", default)]
    first_valid: Option<u64>,
    #[serde(rename = "gh", default)]
    genesis_hash: Option<Bytes>,
    #[serde(rename = "lv", default)]
    last_valid: Option<u64>,
    #[serde(rename = "snd", default)]
    sender: Option<Address>,
    #[serde(rename = "gen", default)]
    genesis_id: Option<Text>,
    #[serde(rename = "grp", default)]
    group: Option<Bytes>,
    #[serde(rename = "lx", default)]
    lease: Option<Bytes>,
    #[serde(rename = "note", default)]
    note: Option<Bytes>,
    #[serde(rename = "rekey", default)]
    rekey: Option<Address>,
    // payment
    #[serde(rename = "rcv", default)]
    receiver: Option<Address>,
    #[serde(rename = "amt", default)]
    amount: Option<u64>,
    #[serde(rename = "close", default)]
    close_remainder_to: Option<Address>,
    // key registration
    #[serde(rename = "votekey", default)]
    vote_pk: Option<Bytes>,
    #[serde(rename = "selkey", default)]
    selection_pk: Option<Bytes>,
    #[serde(rename = "sprfkey", default)]
    state_proof_pk: Option<Bytes>,
    #[serde(rename = "votefst", default)]
    vote_first: Option<u64>,
    #[serde(rename = "votelst", default)]
    vote_last: Option<u64>,
    #[serde(rename = "votekd", default)]
    vote_key_dilution: Option<u64>,
    #[serde(rename = "nonpart", default)]
    nonparticipating: Option<bool>,
    // asset config
    #[serde(rename = "caid", default)]
    config_asset: Option<u64>,
    #[serde(rename = "apar", default)]
    params: Option<AssetParams>,
    // asset transfer
    #[serde(rename = "xaid", default)]
    asset_xfer: Option<u64>,
    #[serde(rename = "aamt", default)]
    asset_amount: Option<u64>,
    #[serde(rename = "asnd", default)]
    asset_sender: Option<Address>,
    #[serde(rename = "arcv", default)]
    asset_receiver: Option<Address>,
    // asset freeze
    #[serde(rename = "fadd", default)]
    freeze_account: Option<Address>,
    #[serde(rename = "faid", default)]
    asset_id: Option<u64>,
    #[serde(rename = "ffrz", default)]
    frozen: Option<bool>,
    // application
    #[serde(rename = "apid", default)]
    app_id: Option<u64>,
    #[serde(rename = "apan", default)]
    on_complete: Option<u64>,
    #[serde(rename = "apat", default)]
    accounts: Option<Vec<Address>>,
    #[serde(rename = "apap", default)]
    approval_program: Option<Bytes>,
    #[serde(rename = "apaa", default)]
    app_arguments: Option<Vec<Bytes>>,
    #[serde(rename = "apsu", default)]
    clear_state_program: Option<Bytes>,
    #[serde(rename = "apfa", default)]
    foreign_apps: Option<Vec<u64>>,
    #[serde(rename = "apas", default)]
    foreign_assets: Option<Vec<u64>>,
    #[serde(rename = "apgs", default)]
    global_state_schema: Option<StateSchema>,
    #[serde(rename = "apls", default)]
    local_state_schema: Option<StateSchema>,
    #[serde(rename = "apep", default)]
    extra_program_pages: Option<u64>,
    #[serde(rename = "apbx", default)]
    boxes: Option<Vec<BoxReference>>,
    // heartbeat
    #[serde(rename = "hb", default)]
    heartbeat: Option<HeartbeatFields>,
}

impl<'de> Deserialize<'de> for Transaction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let f = TxnFields::deserialize(deserializer)?;
        let tx = match f.txn_type.as_ref() {
            "pay" => Transaction::Payment {
                fee: f.fee,
                first_valid: f.first_valid,
                genesis_hash: f.genesis_hash,
                last_valid: f.last_valid,
                sender: f.sender,
                genesis_id: f.genesis_id,
                group: f.group,
                lease: f.lease,
                note: f.note,
                rekey: f.rekey,
                receiver: f.receiver,
                amount: f.amount,
                close_remainder_to: f.close_remainder_to,
            },
            "keyreg" => Transaction::KeyRegistration {
                fee: f.fee,
                first_valid: f.first_valid,
                genesis_hash: f.genesis_hash,
                last_valid: f.last_valid,
                sender: f.sender,
                genesis_id: f.genesis_id,
                group: f.group,
                lease: f.lease,
                note: f.note,
                rekey: f.rekey,
                vote_pk: f.vote_pk,
                selection_pk: f.selection_pk,
                state_proof_pk: f.state_proof_pk,
                vote_first: f.vote_first,
                vote_last: f.vote_last,
                vote_key_dilution: f.vote_key_dilution,
                nonparticipating: f.nonparticipating,
            },
            "acfg" => Transaction::AssetConfig {
                fee: f.fee,
                first_valid: f.first_valid,
                genesis_hash: f.genesis_hash,
                last_valid: f.last_valid,
                sender: f.sender,
                genesis_id: f.genesis_id,
                group: f.group,
                lease: f.lease,
                note: f.note,
                rekey: f.rekey,
                config_asset: f.config_asset,
                params: f.params,
            },
            "axfer" => Transaction::AssetTransfer {
                fee: f.fee,
                first_valid: f.first_valid,
                genesis_hash: f.genesis_hash,
                last_valid: f.last_valid,
                sender: f.sender,
                genesis_id: f.genesis_id,
                group: f.group,
                lease: f.lease,
                note: f.note,
                rekey: f.rekey,
                asset_xfer: f.asset_xfer,
                asset_amount: f.asset_amount,
                asset_sender: f.asset_sender,
                asset_receiver: f.asset_receiver,
                asset_close_remainder_to: f.close_remainder_to,
            },
            "afrz" => Transaction::AssetFreeze {
                fee: f.fee,
                first_valid: f.first_valid,
                genesis_hash: f.genesis_hash,
                last_valid: f.last_valid,
                sender: f.sender,
                genesis_id: f.genesis_id,
                group: f.group,
                lease: f.lease,
                note: f.note,
                rekey: f.rekey,
                freeze_account: f.freeze_account,
                asset_id: f.asset_id,
                frozen: f.frozen,
            },
            "appl" => Transaction::Application {
                fee: f.fee,
                first_valid: f.first_valid,
                genesis_hash: f.genesis_hash,
                last_valid: f.last_valid,
                sender: f.sender,
                genesis_id: f.genesis_id,
                group: f.group,
                lease: f.lease,
                note: f.note,
                rekey: f.rekey,
                app_id: f.app_id,
                on_complete: f.on_complete,
                accounts: f.accounts,
                approval_program: f.approval_program,
                app_arguments: f.app_arguments,
                clear_state_program: f.clear_state_program,
                foreign_apps: f.foreign_apps,
                foreign_assets: f.foreign_assets,
                global_state_schema: f.global_state_schema,
                local_state_schema: f.local_state_schema,
                extra_program_pages: f.extra_program_pages,
                boxes: f.boxes,
            },
            "hb" => Transaction::Heartbeat {
                fee: f.fee,
                first_valid: f.first_valid,
                genesis_hash: f.genesis_hash,
                last_valid: f.last_valid,
                sender: f.sender,
                genesis_id: f.genesis_id,
                group: f.group,
                lease: f.lease,
                note: f.note,
                rekey: f.rekey,
                heartbeat: f.heartbeat,
            },
            other => {
                return Err(serde::de::Error::custom(format!(
                    "unknown transaction type `{other}`"
                )));
            }
        };
        Ok(tx)
    }
}

/// Fields for a heartbeat transaction.
///
/// Mirrors the indexer's `TransactionHeartbeat` shape but keyed by algod's
/// block / pending-transaction msgpack field names (`a`, `kd`, `prf`, `sd`,
/// `vid`).
#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct HeartbeatFields {
    /// `[hbad]` HbAddress — the account this txn is proving onlineness for.
    #[serde(rename = "a")]
    pub address: Option<Address>,
    /// `[hbkd]` HbKeyDilution — must match HbAddress account's KeyDilution.
    #[serde(rename = "kd")]
    pub key_dilution: Option<u64>,
    /// `[hbprf]` HbProof — a signature showing HbAddress is online.
    #[serde(rename = "prf")]
    pub proof: Option<HeartbeatProof>,
    /// `[hbsd]` HbSeed — the block seed for this txn's firstValid block.
    #[serde(rename = "sd")]
    pub seed: Option<Bytes>,
    /// `[hbvid]` HbVoteID — must match HbAddress account's current VoteID.
    #[serde(rename = "vid")]
    pub vote_id: Option<Bytes>,
}

/// `[hbprf]` HbProof — a one-time signature proving heartbeat onlineness.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct HeartbeatProof {
    #[serde(rename = "p")]
    pub pk: Option<Bytes>,
    #[serde(rename = "p1s")]
    pub pk1_sig: Option<Bytes>,
    #[serde(rename = "p2")]
    pub pk2: Option<Bytes>,
    #[serde(rename = "p2s")]
    pub pk2_sig: Option<Bytes>,
    #[serde(rename = "s")]
    pub sig: Option<Bytes>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct AssetParams {
    #[serde(rename = "am")]
    pub meta_data_hash: Option<Bytes>,
    #[serde(rename = "an")]
    pub asset_name: Option<String>,
    #[serde(rename = "au")]
    pub url: Option<String>,
    #[serde(rename = "c")]
    pub clawback: Option<Address>,
    #[serde(rename = "dc")]
    pub decimals: Option<u32>,
    #[serde(rename = "df")]
    pub default_frozen: Option<bool>,
    #[serde(rename = "f")]
    pub freeze: Option<Address>,
    #[serde(rename = "m")]
    pub manager: Option<Address>,
    #[serde(rename = "r")]
    pub reserve: Option<Address>,
    #[serde(rename = "t")]
    pub total: Option<u64>,
    #[serde(rename = "un")]
    pub unit_name: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct BoxReference {
    #[serde(rename = "n")]
    name: Bytes,
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

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct ApplyData {
    #[serde(rename = "dt")]
    pub delta: Option<EvalDelta>,
    #[serde(rename = "ca")]
    pub closing_amount: Option<u64>,
    #[serde(rename = "aca")]
    pub asset_closing_amount: Option<u64>,
}
