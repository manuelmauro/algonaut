use crate::core::{MicroAlgos, Round};
use crate::crypto::HashDigest;
use crate::serialization::{deserialize_bytes, deserialize_hash};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    /// The account public key
    pub address: String,

    /// The total number of MicroAlgos in the account
    pub amount: MicroAlgos,

    /// Specifies the amount of MicroAlgos in the account, without the pending rewards.
    #[serde(rename = "amount-without-pending-rewards")]
    pub amount_without_pending_rewards: u64,

    /// [appl] applications local data stored in this account.
    #[serde(rename = "apps-local-state")]
    pub apps_local_state: Vec<ApplicationLocalState>,

    /// [tsch] stores the sum of all of the local schemas and global schemas in this account.
    /// Note: the raw account uses StateSchema for this type.
    #[serde(rename = "apps-total-schema")]
    pub apps_total_schema: ApplicationStateSchema,

    /// [asset] assets held by this account.
    /// Note the raw object uses map[int] -> AssetHolding for this type.
    pub assets: Vec<AssetHolding>,

    /// [spend] the address against which signing should be checked. If empty, the address of the
    /// current account is used. This field can be updated in any transaction by setting the
    /// RekeyTo field.
    #[serde(rename = "auth-addr")]
    pub auth_addr: String,

    /// [appp] parameters of applications created by this account including app global data.
    /// Note: the raw account uses map[int] -> AppParams for this type.
    #[serde(rename = "created-apps")]
    pub created_apps: Vec<Application>,

    /// [apar] parameters of assets created by this account.
    /// Note: the raw account uses map[int] -> Asset for this type.
    #[serde(rename = "created-assets")]
    pub created_assets: Vec<Asset>,

    ///
    pub participation: AccountParticipation,

    /// Amount of MicroAlgos of pending rewards in this account.
    #[serde(rename = "pending-rewards")]
    pub pending_rewards: u64,

    /// [ebase] used as part of the rewards computation. Only applicable to accounts which
    /// are participating.
    #[serde(rename = "reward-base")]
    pub reward_base: u64,

    /// [ern] total rewards of MicroAlgos the account has received, including pending rewards.
    pub rewards: u64,

    /// The round for which this information is relevant.
    pub round: Round,

    /// Indicates what type of signature is used by this account, must be one of:
    /// * sig
    /// * msig
    /// * lsig
    #[serde(rename = "sig-type")]
    pub sig_type: SignatureType,

    /// [onl] delegation status of the account's MicroAlgos
    /// * Offline - indicates that the associated account is delegated.
    /// * Online - indicates that the associated account used as part of the delegation pool.
    /// * NotParticipating - indicates that the associated account is neither a delegator nor a delegate.
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SignatureType {
    Sig,
    MultiSig,
    LSig,
}

/// AccountParticipation describes the parameters used by this account in consensus protocol.
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountParticipation {
    /// [sel] Selection public key (if any) currently registered for this round.
    /// Pattern : "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==\|[A-Za-z0-9+/]{3}=)?$"
    #[serde(
        rename = "selection-participation-key",
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "deserialize_bytes"
    )]
    pub selection_participation_key: Vec<u8>,

    /// [voteFst] First round for which this participation is valid.
    #[serde(rename = "vote-first-valid")]
    pub vote_first_valid: u64,

    /// [voteKD] Number of subkeys in each batch of participation keys.
    #[serde(rename = "vote-key-dilution")]
    pub vote_key_dilution: u64,

    /// [voteLst] Last round for which this participation is valid.
    #[serde(rename = "vote-last-valid")]
    pub vote_last_valid: u64,

    /// [vote] root participation public key (if any) currently registered for this round.
    /// Pattern : "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==\|[A-Za-z0-9+/]{3}=)?$"
    #[serde(
        rename = "vote-participation-key",
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "deserialize_bytes"
    )]
    pub vote_participation_key: Vec<u8>,
}

/// Application state delta.
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountStateDelta {
    /// Address
    pub address: String,

    /// Delta
    pub delta: StateDelta,
}

/// Application index and its parameters
#[derive(Debug, Serialize, Deserialize)]
pub struct Application {
    /// [appidx] application index.
    pub id: u64,

    /// [appparams] application parameters.
    pub params: ApplicationParams,
}

/// Stores local state associated with an application.
#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationLocalState {
    /// The application which this local state is for.
    pub id: u64,

    /// [tkv] storage.
    #[serde(rename = "key-value")]
    pub key_value: TealKeyValueStore,

    /// [hsch] schema.
    #[serde(rename = "key-value")]
    pub schema: ApplicationStateSchema,
}

/// Stores the global information associated with an application.
#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationParams {
    /// [approv] approval program.
    /// Pattern : "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==\|[A-Za-z0-9+/]{3}=)?$"
    #[serde(
        rename = "approval-program",
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "deserialize_bytes"
    )]
    pub approval_program: Vec<u8>,

    /// [clearp] approval program.
    /// Pattern : "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==\|[A-Za-z0-9+/]{3}=)?$"
    #[serde(
        rename = "clear-state-program",
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "deserialize_bytes"
    )]
    pub clear_state_program: Vec<u8>,

    /// The address that created this application. This is the address where the parameters and
    /// global state for this application can be found.
    pub creator: String,

    ///[\gs] global schema
    #[serde(rename = "global-state")]
    pub global_state: TealKeyValueStore,

    /// [\lsch] global schema
    #[serde(rename = "global-state-schema")]
    pub global_state_schema: ApplicationStateSchema,

    /// [\lsch] local schema

    #[serde(rename = "local-state-schema")]
    pub local_state_schema: ApplicationStateSchema,
}

/// Specifies maximums on the number of each type that may be stored.
#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationStateSchema {
    /// [nbs] num of byte slices.
    #[serde(rename = "num-byte-slice")]
    pub num_byte_slice: u64,

    /// [nui] num of uints.
    #[serde(rename = "num-uint")]
    pub num_uint: u64,
}

/// Specifies both the unique identifier and the parameters for an asset
#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
    /// unique asset identifier
    pub index: u64,

    /// Params.
    pub params: AssetParams,
}

/// Describes an asset held by an account.
/// Definition: data/basics/userBalance.go : AssetHolding
#[derive(Debug, Serialize, Deserialize)]
pub struct AssetHolding {
    /// [a] number of units held.
    pub amount: u64,

    ///Asset ID of the holding.
    #[serde(rename = "asset-id")]
    pub asset_id: u64,

    /// Address that created this asset. This is the address where the parameters for this asset can
    /// be found, and also the address where unwanted asset units can be sent in the worst case.
    pub creator: String,

    /// [f] whether or not the holding is frozen.
    #[serde(rename = "is-frozen")]
    pub is_frozen: bool,
}

/// AssetParams specifies the parameters for an asset.
/// [apar] when part of an AssetConfig transaction.
/// Definition: data/transactions/asset.go : AssetParams
#[derive(Debug, Serialize, Deserialize)]
pub struct AssetParams {
    /// [c] Address of account used to clawback holdings of this asset. If empty, clawback is not
    /// permitted.
    pub clawback: String,

    /// The address that created this asset. This is the address where the parameters for this
    /// asset can be found, and also the address where unwanted asset units can be sent in the worst
    /// case.
    pub creator: String,

    /// [dc] The number of digits to use after the decimal point when displaying this asset.
    /// If 0, the asset is not divisible. If 1, the base unit of the asset is in tenths.
    /// If 2, the base unit of the asset is in hundredths, and so on. This value must be
    /// between 0 and 19 (inclusive).
    /// Minimum value : 0
    /// Maximum value : 19
    pub decimals: u64,

    /// [df] Whether holdings of this asset are frozen by default.
    #[serde(rename = "default-frozen")]
    pub default_frozen: bool,

    /// [f] Address of account used to freeze holdings of this asset. If empty, freezing is not
    /// permitted.
    pub freeze: String,

    /// [m] Address of account used to manage the keys of this asset and to destroy it.
    pub manager: String,

    /// [am] A commitment to some unspecified asset metadata. The format of this metadata is up
    /// to the application.
    /// Pattern : "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==\|[A-Za-z0-9+/]{3}=)?$"
    #[serde(
        rename = "metadata-hash",
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "deserialize_bytes"
    )]
    pub metadata_hash: Vec<u8>,

    /// [an] Name of this asset, as supplied by the creator.
    pub name: String,

    /// [r] Address of account holding reserve (non-minted) units of this asset.
    pub reserve: String,

    /// [t] The total number of units of this asset.
    pub total: u64,

    /// [un] Name of a unit of this asset, as supplied by the creator.
    #[serde(rename = "unit-name")]
    pub unit_name: String,

    /// [au] URL where more information about the asset can be retrieved.
    pub url: String,
}

/// BuildVersion
#[derive(Debug, Serialize, Deserialize)]
pub struct BuildVersion {
    pub branch: String,
    pub build_number: u64,
    pub channel: String,
    pub commit_hash: String,
    pub major: u64,
    pub minor: u64,
}

/// Request data type for dryrun endpoint. Given the Transactions and simulated ledger state
/// upload, run TEAL scripts and return debugging information.
#[derive(Debug, Serialize, Deserialize)]
pub struct DryrunRequest {
    pub accounts: Vec<Account>,

    pub apps: Vec<Application>,

    /// LatestTimestamp is available to some TEAL scripts. Defaults to the latest confirmed
    /// timestamp this algod is attached to.
    #[serde(rename = "latest-timestamp")]
    pub latest_timestamp: u64,

    /// ProtocolVersion specifies a specific version string to operate under, otherwise whatever
    /// the current protocol of the network this algod is running in.
    #[serde(rename = "protocol-version")]
    pub protocol_version: String,

    /// Round is available to some TEAL scripts. Defaults to the current round on the network
    /// this algod is attached to.
    pub round: Round,

    pub sources: Vec<DryrunSource>,

    pub txns: Vec<String>,
}

/// DryrunSource is TEAL source text that gets uploaded, compiled, and inserted into transactions
/// or application state.
#[derive(Debug, Serialize, Deserialize)]
pub struct DryrunSource {
    #[serde(rename = "app-index")]
    pub app_index: u64,

    /// FieldName is what kind of sources this is. If lsig then it goes into the
    /// transactions[this.TxnIndex].LogicSig.
    /// If approv or clearp it goes into the Approval Program or Clear State Program of
    /// application[this.AppIndex].
    #[serde(rename = "field-name")]
    pub field_name: String,

    pub source: String,

    #[serde(rename = "txn-index")]
    pub txn_index: u64,
}

/// Stores the TEAL eval step data
#[derive(Debug, Serialize, Deserialize)]
pub struct DryrunState {
    /// Evaluation error if any
    pub error: String,

    /// Line number
    pub line: u64,

    /// Program counter
    pub pc: u64,

    pub scratch: Vec<TealValue>,

    pub stack: Vec<TealValue>,
}

/// DryrunTxnResult contains any LogicSig or ApplicationCall program debug information
/// and state updates from a dryrun.
#[derive(Debug, Serialize, Deserialize)]
pub struct DryrunTxnResult {
    #[serde(rename = "app-call-messages")]
    pub app_call_messages: Vec<String>,

    #[serde(rename = "app-call-trace")]
    pub app_call_trace: Vec<DryrunState>,

    /// Disassembled program line by line.
    pub disassembly: Vec<String>,

    #[serde(rename = "global-delta")]
    pub global_delta: StateDelta,

    #[serde(rename = "local-deltas")]
    pub local_deltas: Vec<AccountStateDelta>,

    #[serde(rename = "logic-sig-messages")]
    pub logic_sig_messages: Vec<String>,

    #[serde(rename = "logic-sig-trace")]
    pub logic_sig_trace: Vec<DryrunState>,
}

/// An error response with optional data field.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub data: String,
    pub message: String,
}

/// Represents a TEAL value delta.
#[derive(Debug, Serialize, Deserialize)]
pub struct EvalDelta {
    /// [at] delta action.
    pub action: u64,

    /// [bs] bytes value.
    pub bytes: String,

    /// [ui] uint value.
    pub uint: u64,
}

/// Key-value pairs for StateDelta.
#[derive(Debug, Serialize, Deserialize)]
pub struct EvalDeltaKeyValue {
    pub key: String,
    pub value: EvalDelta,
}

/// Application state delta.
pub type StateDelta = Vec<EvalDeltaKeyValue>;

/// Represents a key-value pair in an application store.
#[derive(Debug, Serialize, Deserialize)]
pub struct TealKeyValue {
    pub key: String,
    pub value: TealValue,
}

/// Represents a key-value store for use in an application.
pub type TealKeyValueStore = Vec<TealKeyValue>;

/// Represents a TEAL value.
#[derive(Debug, Serialize, Deserialize)]
pub struct TealValue {
    /// [tb] bytes value.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "deserialize_bytes"
    )]
    pub bytes: Vec<u8>,

    /// [tt] value type.
    #[serde(rename = "type")]
    pub value_type: u64,

    /// [ui] uint value.
    pub uint: u64,
}

/// Version contains the current algod version.
#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub versions: Vec<String>,
    pub genesis_id: String,
    #[serde(rename = "genesis_hash_b64", deserialize_with = "deserialize_hash")]
    pub genesis_hash: HashDigest,
}
