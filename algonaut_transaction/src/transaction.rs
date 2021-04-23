use crate::account::Account;
use crate::error::AlgorandError;
use algonaut_core::{Address, MultisigSignature, Signature};
use algonaut_core::{MicroAlgos, Round, VotePk, VrfPk};
use algonaut_crypto::HashDigest;
use serde::{Deserialize, Serialize, Serializer};

const MIN_TXN_FEE: MicroAlgos = MicroAlgos(1000);

/// Enum containing the types of transactions and their specific fields
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub enum TransactionType {
    #[serde(rename = "pay")]
    Payment(Payment),
    #[serde(rename = "keyreg")]
    KeyRegistration(KeyRegistration),
    #[serde(rename = "acfg")]
    AssetConfigurationTransaction(AssetConfigurationTransaction),
    #[serde(rename = "axfer")]
    AssetTransferTransaction(AssetTransferTransaction),
    #[serde(rename = "axfer")]
    AssetAcceptTransaction(AssetAcceptTransaction),
    #[serde(rename = "axfer")]
    AssetClawbackTransaction(AssetClawbackTransaction),
    #[serde(rename = "afrz")]
    AssetFreezeTransaction(AssetFreezeTransaction),
    #[serde(rename = "appl")]
    ApplicationCallTransaction(ApplicationCallTransaction),
}

/// A transaction that can appear in a block
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub struct Transaction {
    /// Paid by the sender to the FeeSink to prevent denial-of-service. The minimum fee on Algorand
    /// is currently 1000 microAlgos.
    #[serde(rename = "fee")]
    pub fee: MicroAlgos,

    /// The first round for when the transaction is valid. If the transaction is sent prior to this
    /// round it will be rejected by the network.
    #[serde(rename = "fv")]
    pub first_valid: Round,

    /// The hash of the genesis block of the network for which the transaction is valid. See the
    /// genesis hash for MainNet, TestNet, and BetaNet.
    #[serde(rename = "gh")]
    pub genesis_hash: HashDigest,

    /// The ending round for which the transaction is valid. After this round, the transaction will
    /// be rejected by the network.
    #[serde(rename = "lv")]
    pub last_valid: Round,

    /// The address of the account that pays the fee and amount.
    #[serde(rename = "snd")]
    pub sender: Address,

    /// Specifies the type of transaction. This value is automatically generated using any of the
    /// developer tools.
    #[serde(flatten)]
    pub txn_type: TransactionType,

    /// The human-readable string that identifies the network for the transaction. The genesis ID is
    /// found in the genesis block. See the genesis ID for MainNet, TestNet, and BetaNet.
    #[serde(rename = "gen", default)]
    pub genesis_id: String,

    /// The group specifies that the transaction is part of a group and, if so, specifies the hash of
    /// the transaction group. Assign a group ID to a transaction through the workflow described in
    /// the Atomic Transfers Guide.
    #[serde(rename = "grp", default)]
    pub group: Option<HashDigest>,

    /// A lease enforces mutual exclusion of transactions. If this field is nonzero, then once the
    /// transaction is confirmed, it acquires the lease identified by the (Sender, Lease) pair of
    /// the transaction until the LastValid round passes. While this transaction possesses the
    /// lease, no other transaction specifying this lease can be confirmed. A lease is often used
    /// in the context of Algorand Smart Contracts to prevent replay attacks. Read more about
    /// Algorand Smart Contracts and see the Delegate Key Registration TEAL template for an example
    /// implementation of leases. Leases can also be used to safeguard against unintended duplicate
    /// spends. For example, if I send a transaction to the network and later realize my fee was too
    /// low, I could send another transaction with a higher fee, but the same lease value. This would
    /// ensure that only one of those transactions ends up getting confirmed during the validity period.
    #[serde(rename = "lx", default)]
    pub lease: Option<HashDigest>,

    /// Any data up to 1000 bytes.
    #[serde(with = "serde_bytes", default)]
    pub note: Vec<u8>,

    /// Specifies the authorized address. This address will be used to authorize all future transactions.
    /// Learn more about Rekeying accounts.
    #[serde(rename = "rekey", default)]
    pub rekey_to: Option<Address>,
}

impl Transaction {
    /// Creates a new transaction with a fee calculated based on `fee_per_byte`.
    pub fn fee_per_byte(mut self, fee_per_byte: MicroAlgos) -> Result<Transaction, AlgorandError> {
        self.fee = MIN_TXN_FEE.max(fee_per_byte * self.estimate_size()?);
        Ok(self)
    }

    // Estimates the size of the encoded transaction, used in calculating the fee
    fn estimate_size(&self) -> Result<u64, AlgorandError> {
        let account = Account::generate();
        let len = rmp_serde::to_vec_named(&account.sign_transaction(self)?)?.len() as u64;
        Ok(len)
    }
}

impl Serialize for Transaction {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;

        // transaction the number of fields in the transaction
        let type_len = match &self.txn_type {
            TransactionType::Payment(payment) => {
                1 + if payment.close_remainder_to.is_some() {
                    1
                } else {
                    0
                } + if payment.amount.0 != 0 { 1 } else { 0 }
            }
            TransactionType::KeyRegistration(_) => 5,
            TransactionType::AssetConfigurationTransaction(_) => 2,
            TransactionType::AssetTransferTransaction(_) => 5,
            TransactionType::AssetAcceptTransaction(_) => 3,
            TransactionType::AssetClawbackTransaction(_) => 6,
            TransactionType::AssetFreezeTransaction(_) => 3,
            TransactionType::ApplicationCallTransaction(_) => 10,
        };

        let len = 6
            + type_len
            + if self.note.is_empty() { 0 } else { 1 }
            + if self.genesis_id.is_empty() { 0 } else { 1 };

        // serialize fields common to all transactions
        let mut state = serializer.serialize_struct("Transaction", len)?;
        state.serialize_field("fee", &self.fee)?;
        state.serialize_field("fv", &self.first_valid)?;
        if !self.genesis_id.is_empty() {
            state.serialize_field("gen", &self.genesis_id)?;
        }
        state.serialize_field("gh", &self.genesis_hash)?;
        state.serialize_field("lv", &self.last_valid)?;
        if !self.note.is_empty() {
            state.serialize_field("note", &serde_bytes::ByteBuf::from(self.note.clone()))?;
        }
        state.serialize_field("snd", &self.sender)?;

        // serialize different transaction types
        match &self.txn_type {
            TransactionType::Payment(p) => {
                state.serialize_field("type", "pay")?;
                // transaction's specific fields
                if p.amount.0 != 0 {
                    state.serialize_field("amt", &p.amount)?;
                }
                if p.close_remainder_to.is_some() {
                    state.serialize_field("close", &p.close_remainder_to)?;
                }
                state.serialize_field("rcv", &p.receiver)?;
            }
            TransactionType::KeyRegistration(kr) => {
                state.serialize_field("type", "keyreg")?;
                // transaction's specific fields
                state.serialize_field("selkey", &kr.selection_pk)?;
                state.serialize_field("votefst", &kr.vote_first)?;
                state.serialize_field("votekd", &kr.vote_key_dilution)?;
                state.serialize_field("votekey", &kr.vote_pk)?;
                state.serialize_field("votelst", &kr.vote_last)?;
            }
            TransactionType::AssetConfigurationTransaction(asa_conf) => {
                state.serialize_field("type", "acfg")?;
                // transaction's specific fields
                state.serialize_field("caid", &asa_conf.config_asset)?;
                state.serialize_field("apar", &asa_conf.params)?;
            }
            TransactionType::AssetTransferTransaction(asa_transf) => {
                state.serialize_field("type", "axfer")?;
                // transaction's specific fields
                state.serialize_field("xaid", &asa_transf.xfer)?;
                state.serialize_field("aamt", &asa_transf.amount)?;
                state.serialize_field("asnd", &asa_transf.sender)?;
                state.serialize_field("arcv", &asa_transf.receiver)?;
                state.serialize_field("aclose", &asa_transf.close_to)?;
            }
            TransactionType::AssetAcceptTransaction(asa_accept) => {
                state.serialize_field("type", "axfer")?;
                // transaction's specific fields
                state.serialize_field("xaid", &asa_accept.xfer)?;
                state.serialize_field("asnd", &asa_accept.sender)?;
                state.serialize_field("arcv", &asa_accept.receiver)?;
            }
            TransactionType::AssetClawbackTransaction(asa_claw) => {
                state.serialize_field("type", "axfer")?;
                // transaction's specific fields
                state.serialize_field("snd", &asa_claw.sender)?;
                state.serialize_field("xaid", &asa_claw.xfer)?;
                state.serialize_field("aamt", &asa_claw.asset_amount)?;
                state.serialize_field("asnd", &asa_claw.asset_sender)?;
                state.serialize_field("arcv", &asa_claw.asset_receiver)?;
                state.serialize_field("aclose", &asa_claw.asset_close_to)?;
            }
            TransactionType::AssetFreezeTransaction(asa_frz) => {
                state.serialize_field("type", "afrz")?;
                // transaction's specific fields
                state.serialize_field("fadd", &asa_frz.freeze_account)?;
                state.serialize_field("faid", &asa_frz.asset_id)?;
                state.serialize_field("afrz", &asa_frz.frozen)?;
            }
            TransactionType::ApplicationCallTransaction(app_call) => {
                state.serialize_field("type", "appl")?;
                // transaction's specific fields
                state.serialize_field("apid", &app_call.app_id)?;
                state.serialize_field("apan", &app_call.on_complete)?;
                state.serialize_field("apat", &app_call.accounts)?;
                state.serialize_field("apap", &app_call.approval_program)?;
                state.serialize_field("apaa", &app_call.app_arguments)?;
                state.serialize_field("apsu", &app_call.clear_state_program)?;
                state.serialize_field("apfa", &app_call.foreign_apps)?;
                state.serialize_field("apas", &app_call.foreign_assets)?;
                state.serialize_field("apgs", &app_call.global_state_schema)?;
                state.serialize_field("apls", &app_call.local_state_schema)?;
            }
        }

        state.end()
    }
}

/// Fields for a payment transaction
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub struct Payment {
    /// The address of the account that receives the amount.
    #[serde(rename = "rcv")]
    pub receiver: Address,

    /// The total amount to be sent in microAlgos.
    #[serde(rename = "amt", default)]
    pub amount: MicroAlgos,

    /// When set, it indicates that the transaction is requesting that the Sender account should
    /// be closed, and all remaining funds, after the fee and amount are paid, be transferred to
    /// this address.
    #[serde(rename = "close")]
    pub close_remainder_to: Option<Address>,
}

/// Fields for a key registration transaction
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub struct KeyRegistration {
    /// The root participation public key. See Generate a Participation Key to learn more.
    #[serde(rename = "votekey")]
    pub vote_pk: VotePk,

    /// The VRF public key.
    #[serde(rename = "selkey")]
    pub selection_pk: VrfPk,

    /// The first round that the participation key is valid. Not to be confused with the FirstValid
    /// round of the keyreg transaction.
    #[serde(rename = "votefst")]
    pub vote_first: Round,

    /// The last round that the participation key is valid. Not to be confused with the LastValid
    /// round of the keyreg transaction.
    #[serde(rename = "votelst")]
    pub vote_last: Round,

    /// This is the dilution for the 2-level participation key.
    #[serde(rename = "votekd")]
    pub vote_key_dilution: u64,

    /// All new Algorand accounts are participating by default. This means that they earn rewards.
    /// Mark an account nonparticipating by setting this value to true and this account will no
    /// longer earn rewards. It is unlikely that you will ever need to do this and exists mainly
    /// for economic-related functions on the network.
    #[serde(rename = "nonpart", skip_serializing_if = "Option::is_none")]
    pub nonparticipating: Option<bool>,
}

/// This is used to create, configure and destroy an asset depending on which fields are set.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub struct AssetConfigurationTransaction {
    /// For re-configure or destroy transactions, this is the unique asset ID. On asset creation,
    /// the ID is set to zero.
    #[serde(rename = "caid")]
    pub config_asset: u64,

    /// See AssetParams table for all available fields.
    #[serde(rename = "apar")]
    pub params: AssetParams,
}

/// This is used to create or configure an asset.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct AssetParams {
    /// The total number of base units of the asset to create. This number cannot be changed.
    #[serde(rename = "t")]
    pub total: u64,

    /// The number of digits to use after the decimal point when displaying the asset. If 0,
    /// the asset is not divisible. If 1, the base unit of the asset is in tenths. If 2,
    /// the base unit of the asset is in hundredths.
    #[serde(rename = "dc")]
    pub decimals: u32,

    /// True to freeze holdings for this asset by default.
    #[serde(rename = "df")]
    pub default_frozen: bool,

    /// The name of a unit of this asset. Supplied on creation. Example: USDT
    #[serde(rename = "un", skip_serializing_if = "Option::is_none")]
    pub unit_name: Option<String>,

    /// The name of the asset. Supplied on creation. Example: Tether
    #[serde(rename = "an", skip_serializing_if = "Option::is_none")]
    pub asset_name: Option<String>,

    /// Specifies a URL where more information about the asset can be retrieved. Max size is 32 bytes.
    #[serde(rename = "au", skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// This field is intended to be a 32-byte hash of some metadata that is relevant to your asset
    /// and/or asset holders. The format of this metadata is up to the application. This field can only
    /// be specified upon creation. An example might be the hash of some certificate that acknowledges
    /// the digitized asset as the official representation of a particular real-world asset.
    #[serde(rename = "am", skip_serializing_if = "Option::is_none")]
    pub meta_data_hash: Option<Vec<u8>>,

    /// The address of the account that can manage the configuration of the asset and destroy it.
    #[serde(rename = "m", skip_serializing_if = "Option::is_none")]
    pub manager: Option<Address>,

    /// The address of the account that holds the reserve (non-minted) units of the asset. This address
    /// has no specific authority in the protocol itself. It is used in the case where you want to
    /// signal to holders of your asset that the non-minted units of the asset reside in an account
    /// that is different from the default creator account (the sender).
    #[serde(rename = "r", skip_serializing_if = "Option::is_none")]
    pub reserve: Option<Address>,

    /// The address of the account used to freeze holdings of this asset. If empty, freezing is not
    /// permitted.
    #[serde(rename = "f", skip_serializing_if = "Option::is_none")]
    pub freeze: Option<Address>,

    /// The address of the account that can clawback holdings of this asset. If empty, clawback is
    /// not permitted.
    #[serde(rename = "c", skip_serializing_if = "Option::is_none")]
    pub clawback: Option<Address>,
}

/// This is used to transfer an asset.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AssetTransferTransaction {
    /// The unique ID of the asset to be transferred.
    #[serde(rename = "xaid")]
    pub xfer: u64,

    /// The amount of the asset to be transferred. A zero amount transferred to self allocates that
    /// asset in the account's Asset map.
    #[serde(rename = "aamt")]
    pub amount: u64,

    /// The sender of the transfer. The regular sender field should be used and this one set to the
    /// zero value for regular transfers between accounts. If this value is nonzero, it indicates a
    /// clawback transaction where the sender is the asset's clawback address and the asset sender
    /// is the address from which the funds will be withdrawn.
    #[serde(rename = "asnd")]
    pub sender: Address,

    /// The recipient of the asset transfer.
    #[serde(rename = "arcv")]
    pub receiver: Address,

    /// Specify this field to remove the asset holding from the sender account and reduce the
    /// account's minimum balance.
    #[serde(rename = "aclose")]
    pub close_to: Address,
}

/// This is a special form of an Asset Transfer Transaction.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AssetAcceptTransaction {
    /// The unique ID of the asset to be transferred.
    #[serde(rename = "xaid")]
    pub xfer: u64,

    /// The account which is allocating the asset to their account's Asset map.
    #[serde(rename = "asnd")]
    pub sender: Address,

    /// The account which is allocating the asset to their account's Asset map.
    #[serde(rename = "arcv")]
    pub receiver: Address,
}

/// This is a special form of an Asset Transfer Transaction.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AssetClawbackTransaction {
    /// The sender of this transaction must be the clawback account specified in the asset
    /// configuration.
    #[serde(rename = "snd")]
    pub sender: Address,

    /// The unique ID of the asset to be transferred.
    #[serde(rename = "xaid")]
    pub xfer: u64,

    /// The amount of the asset to be transferred.
    #[serde(rename = "aamt")]
    pub asset_amount: u64,

    /// The address from which the funds will be withdrawn.
    #[serde(rename = "asnd")]
    pub asset_sender: Address,

    /// The recipient of the asset transfer.
    #[serde(rename = "arcv")]
    pub asset_receiver: Address,

    /// Specify this field to remove the entire asset holding balance from the AssetSender
    /// account. It will not remove the asset holding.
    #[serde(rename = "aclose")]
    pub asset_close_to: Address,
}

/// This is a special form of an Asset Transfer Transaction.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AssetFreezeTransaction {
    /// The address of the account whose asset is being frozen or unfrozen.
    #[serde(rename = "fadd")]
    pub freeze_account: Address,

    /// The asset ID being frozen or unfrozen.
    #[serde(rename = "faid")]
    pub asset_id: u64,

    /// True to freeze the asset.
    #[serde(rename = "afrz")]
    pub frozen: bool,
}

///
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApplicationCallTransaction {
    /// ID of the application being configured or empty if creating.
    #[serde(rename = "apid")]
    pub app_id: u64,

    /// Defines what additional actions occur with the transaction. See the OnComplete section of
    /// the TEAL spec for details.
    #[serde(rename = "apan")]
    pub on_complete: u64,

    /// List of accounts in addition to the sender that may be accessed from the application's
    /// approval-program and clear-state-program.
    #[serde(rename = "apat")]
    pub accounts: Option<Vec<Address>>,

    /// Logic executed for every application transaction, except when on-completion is set to
    /// "clear". It can read and write global state for the application, as well as account-specific
    /// local state. Approval programs may reject the transaction.
    #[serde(rename = "apap")]
    pub approval_program: Option<Address>,

    /// Transaction specific arguments accessed from the application's approval-program and
    /// clear-state-program.
    #[serde(rename = "apaa")]
    pub app_arguments: Option<Vec<u8>>,

    /// Logic executed for application transactions with on-completion set to "clear". It can read
    /// and write global state for the application, as well as account-specific local state. Clear
    /// state programs cannot reject the transaction.
    #[serde(rename = "apsu")]
    pub clear_state_program: Option<Address>,

    /// Lists the applications in addition to the application-id whose global states may be accessed
    /// by this application's approval-program and clear-state-program. The access is read-only.
    #[serde(rename = "apfa")]
    pub foreign_apps: Option<Address>,

    /// Lists the assets whose AssetParams may be accessed by this application's approval-program and
    /// clear-state-program. The access is read-only.
    #[serde(rename = "apas")]
    pub foreign_assets: Option<Address>,

    /// Holds the maximum number of global state values defined within a StateSchema object.
    #[serde(rename = "apgs")]
    pub global_state_schema: Option<StateSchema>,

    /// Holds the maximum number of local state values defined within a StateSchema object.
    #[serde(rename = "apls")]
    pub local_state_schema: Option<StateSchema>,
}

/// Storage state schema. The StateSchema object is only required for the create application call
/// transaction. The StateSchema object must be fully populated for both the GlobalStateSchema and
/// LocalStateSchema objects.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StateSchema {
    /// Maximum number of integer values that may be stored in the [global || local] application
    /// key/value store. Immutable.
    #[serde(rename = "nui")]
    pub number_ints: u64,

    /// Maximum number of byte slices values that may be stored in the [global || local] application
    /// key/value store. Immutable.
    #[serde(rename = "nbs")]
    pub number_byteslices: u64,
}

/// Wraps a transaction in a signature. The encoding of this struct is suitable to be broadcast
/// on the network
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SignedTransaction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<Signature>,

    #[serde(rename = "msig", skip_serializing_if = "Option::is_none")]
    pub multisig: Option<MultisigSignature>,

    #[serde(rename = "txn")]
    pub transaction: Transaction,

    #[serde(skip)]
    pub transaction_id: String,
}
