use algonaut_core::{Address, MicroAlgos, MultisigSignature, Round, ToMsgPack, VotePk, VrfPk};
use algonaut_crypto::{HashDigest, HashType, Signature};
use algonaut_encoding::{deserialize_bytes64, serialize_bytes};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// IMPORTANT:
/// When serializing:
/// - Fields have to be sorted alphabetically.
/// - Keys must be excluded if they've a "zero value" (e.g. the number 0 or an empty vector).
/// otherwise the node's signature validation will fail.
/// When deserializing:
/// - Non existent keys can mean None or a semantic zero value, depending on context.
///
/// Note that to date the REST API documentation specifies explicitly zero values for some fields, which is incorrect.
/// https://github.com/algorand/docs/pull/454, https://github.com/algorand/docs/issues/415 (not comprehensive)
///
/// We intentionally don't use `skip_serializing_if` for values other than `Option` for a consistent representation of optionals.
///
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiTransaction {
    #[serde(rename = "aamt", skip_serializing_if = "Option::is_none")]
    pub asset_amount: Option<u64>,

    #[serde(rename = "aclose", skip_serializing_if = "Option::is_none")]
    pub asset_close_to: Option<Address>,

    #[serde(rename = "afrz", skip_serializing_if = "Option::is_none")]
    pub frozen: Option<bool>,

    #[serde(rename = "amt", skip_serializing_if = "Option::is_none")]
    pub amount: Option<u64>,

    #[serde(rename = "apaa", skip_serializing_if = "Option::is_none")]
    pub app_arguments: Option<Vec<AppArgument>>,

    #[serde(rename = "apan", skip_serializing_if = "Option::is_none")]
    pub on_complete: Option<u32>,

    #[serde(
        default,
        rename = "apap",
        with = "serde_bytes",
        skip_serializing_if = "Option::is_none"
    )]
    pub approval_program: Option<Vec<u8>>,

    #[serde(rename = "apar", skip_serializing_if = "Option::is_none")]
    pub asset_params: Option<ApiAssetParams>,

    #[serde(rename = "apas", skip_serializing_if = "Option::is_none")]
    pub foreign_assets: Option<Vec<u64>>,

    #[serde(rename = "apat", skip_serializing_if = "Option::is_none")]
    pub accounts: Option<Vec<Address>>,

    #[serde(rename = "apep", skip_serializing_if = "Option::is_none")]
    pub extra_pages: Option<u32>,

    #[serde(rename = "apfa", skip_serializing_if = "Option::is_none")]
    pub foreign_apps: Option<Vec<u64>>,

    #[serde(rename = "apgs", skip_serializing_if = "Option::is_none")]
    pub global_state_schema: Option<ApiStateSchema>,

    #[serde(rename = "apid", skip_serializing_if = "Option::is_none")]
    pub app_id: Option<u64>,

    #[serde(rename = "apls", skip_serializing_if = "Option::is_none")]
    pub local_state_schema: Option<ApiStateSchema>,

    #[serde(
        default,
        rename = "apsu",
        with = "serde_bytes",
        skip_serializing_if = "Option::is_none"
    )]
    pub clear_state_program: Option<Vec<u8>>,

    #[serde(rename = "arcv", skip_serializing_if = "Option::is_none")]
    pub asset_receiver: Option<Address>,

    #[serde(rename = "asnd", skip_serializing_if = "Option::is_none")]
    pub asset_sender: Option<Address>,

    #[serde(rename = "caid", skip_serializing_if = "Option::is_none")]
    pub config_asset: Option<u64>,

    #[serde(rename = "close", skip_serializing_if = "Option::is_none")]
    pub close_reminder_to: Option<Address>,

    #[serde(rename = "fadd", skip_serializing_if = "Option::is_none")]
    pub freeze_account: Option<Address>,

    #[serde(rename = "faid", skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<u64>,

    #[serde(rename = "fee", skip_serializing_if = "Option::is_none")]
    pub fee: Option<MicroAlgos>, // optional for serialization zero value omission

    #[serde(rename = "fv", skip_serializing_if = "Option::is_none")]
    pub first_valid: Option<Round>, // optional for serialization zero value (technically possible) omission

    #[serde(rename = "gen", skip_serializing_if = "Option::is_none")]
    pub genesis_id: Option<String>,

    #[serde(rename = "gh")]
    pub genesis_hash: String,

    #[serde(rename = "grp", skip_serializing_if = "Option::is_none")]
    pub group: Option<HashDigest>,

    #[serde(rename = "lv", skip_serializing_if = "Option::is_none")]
    pub last_valid: Option<Round>, // optional for serialization zero value (technically possible) omission

    #[serde(rename = "lx", skip_serializing_if = "Option::is_none")]
    pub lease: Option<HashDigest>,

    #[serde(rename = "nonpart", skip_serializing_if = "Option::is_none")]
    pub nonparticipating: Option<bool>,

    #[serde(
        default,
        rename = "note",
        with = "serde_bytes",
        skip_serializing_if = "Option::is_none"
    )]
    pub note: Option<Vec<u8>>,

    #[serde(rename = "rcv", skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Address>,

    #[serde(rename = "rekey", skip_serializing_if = "Option::is_none")]
    pub rekey_to: Option<Address>,

    #[serde(rename = "selkey", skip_serializing_if = "Option::is_none")]
    pub selection_pk: Option<VrfPk>,

    #[serde(rename = "snd")]
    pub sender: Address,

    #[serde(rename = "sp", skip_serializing_if = "Option::is_none")]
    pub state_proof: Option<StateProof>,

    #[serde(rename = "spmsg", skip_serializing_if = "Option::is_none")]
    pub state_proof_message: Option<StateProofMessage>,

    #[serde(rename = "sptype", skip_serializing_if = "Option::is_none")]
    pub state_proof_type: Option<StateProofType>,

    #[serde(rename = "type")]
    pub type_: String,

    #[serde(rename = "votefst", skip_serializing_if = "Option::is_none")]
    pub vote_first: Option<Round>,

    #[serde(rename = "votekd", skip_serializing_if = "Option::is_none")]
    pub vote_key_dilution: Option<u64>,

    #[serde(rename = "votekey", skip_serializing_if = "Option::is_none")]
    pub vote_pk: Option<VotePk>,

    #[serde(rename = "votelst", skip_serializing_if = "Option::is_none")]
    pub vote_last: Option<Round>,

    #[serde(rename = "xaid", skip_serializing_if = "Option::is_none")]
    pub xfer: Option<u64>,
}

#[derive(Default, Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct ApiSignedLogicArg(#[serde(with = "serde_bytes")] pub Vec<u8>);

#[derive(Default, Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct ApiSignedLogic {
    #[serde(rename = "arg")]
    pub args: Vec<ApiSignedLogicArg>,
    #[serde(rename = "l", with = "serde_bytes")]
    pub logic: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msig: Option<MultisigSignature>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<Signature>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiSignedTransaction {
    #[serde(rename = "lsig", skip_serializing_if = "Option::is_none")]
    pub lsig: Option<ApiSignedLogic>,

    #[serde(rename = "msig", skip_serializing_if = "Option::is_none")]
    pub msig: Option<MultisigSignature>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<Signature>,

    #[serde(rename = "txn")]
    pub transaction: ApiTransaction,

    #[serde(rename = "sgnr")]
    pub auth_address: Option<Address>,

    #[serde(skip)]
    pub transaction_id: String,
}

#[derive(Default, Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct AppArgument(#[serde(with = "serde_bytes")] pub Vec<u8>);

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct ApiAssetParams {
    #[serde(rename = "am", skip_serializing_if = "Option::is_none")]
    pub meta_data_hash: Option<Vec<u8>>,

    #[serde(rename = "an", skip_serializing_if = "Option::is_none")]
    pub asset_name: Option<String>,

    #[serde(rename = "au", skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    #[serde(rename = "c", skip_serializing_if = "Option::is_none")]
    pub clawback: Option<Address>,

    #[serde(rename = "dc", skip_serializing_if = "Option::is_none")]
    pub decimals: Option<u32>,

    #[serde(rename = "df", skip_serializing)]
    pub default_frozen: Option<bool>,

    #[serde(rename = "f", skip_serializing_if = "Option::is_none")]
    pub freeze: Option<Address>,

    #[serde(rename = "m", skip_serializing_if = "Option::is_none")]
    pub manager: Option<Address>,

    #[serde(rename = "r", skip_serializing_if = "Option::is_none")]
    pub reserve: Option<Address>,

    #[serde(rename = "t", skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,

    #[serde(rename = "un", skip_serializing_if = "Option::is_none")]
    pub unit_name: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApiStateSchema {
    #[serde(rename = "nbs", skip_serializing_if = "Option::is_none")]
    pub number_byteslices: Option<u64>,

    #[serde(rename = "nui", skip_serializing_if = "Option::is_none")]
    pub number_ints: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StateProof {
    #[serde(rename = "c")]
    pub sig_commit: HashDigest,

    #[serde(rename = "w")]
    pub signed_weight: u64,

    #[serde(rename = "S")]
    pub sig_proofs: MerkleArrayProof,

    #[serde(rename = "P")]
    pub part_proofs: MerkleArrayProof,

    #[serde(rename = "v")]
    pub merkle_signature_salt_version: u8,

    /// Reveals is a sparse map from the position being revealed
    /// to the corresponding elements from the sigs and participants
    /// arrays.
    #[serde(rename = "r")]
    pub reveals: HashMap<u64, Reveal>,

    #[serde(rename = "pr")]
    pub positions_to_reveal: Vec<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SigSlotCommit {
    /// Sig is a signature by the participant on the expected message.
    #[serde(rename = "s")]
    pub sig: Signature,

    /// l is the total weight of signatures in lower-numbered slots.
    /// This is initialized once the builder has collected a sufficient
    /// number of signatures.
    #[serde(rename = "l")]
    pub l: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Verifier {
    #[serde(
        rename = "cmt",
        serialize_with = "serialize_bytes",
        deserialize_with = "deserialize_bytes64"
    )]
    pub commitment: [u8; 64],

    #[serde(rename = "lf")]
    pub key_lifetime: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Participant {
    /// PK is the identifier used to verify the signature for a specific participant
    #[serde(rename = "p")]
    pub pk: Verifier,

    /// Weight is AccountData.MicroAlgos.
    #[serde(rename = "w")]
    pub weight: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Reveal {
    #[serde(rename = "s")]
    pub sig_slot: SigSlotCommit,
    pub part: Participant,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MerkleArrayProof {
    /// Path is bounded by MaxNumLeavesOnEncodedTree since there could be multiple reveals, and
    /// given the distribution of the elt positions and the depth of the tree,
    /// the path length can increase up to 2^MaxEncodedTreeDepth / 2
    #[serde(rename = "pth")]
    pub path: Vec<HashDigest>,

    #[serde(rename = "hsh")]
    pub hash_factory: HashFactory,

    /// TreeDepth represents the depth of the tree that is being proven.
    /// It is the number of edges from the root to a leaf.
    #[serde(rename = "td")]
    pub tree_depth: u8,
}

#[derive(Clone, Eq, Debug, PartialEq, Serialize, Deserialize)]
pub struct HashFactory {
    #[serde(rename = "t")]
    pub hash_type: HashType,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StateProofMessage {
    /// BlockHeadersCommitment contains a commitment on all light block headers within a state proof interval.
    #[serde(rename = "b")]
    pub block_headers_commitment: Vec<u8>,

    #[serde(rename = "v")]
    pub voters_commitment: Vec<u8>,

    #[serde(rename = "P")]
    pub ln_proven_weight: u64,

    #[serde(rename = "f")]
    pub first_attested_round: u64,

    #[serde(rename = "l")]
    pub last_attested_round: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum StateProofType {
    /// StateProofBasic is our initial state proof setup.
    /// using falcon keys and subset-sum hash
    StateProofBasic,
}

impl ToMsgPack for ApiTransaction {}
impl ToMsgPack for ApiSignedTransaction {}
