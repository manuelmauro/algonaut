use crate::account::Account;
use crate::error::TransactionError;
use algonaut_core::CompiledTeal;
use algonaut_core::SignedLogic;
use algonaut_core::SuggestedTransactionParams;
use algonaut_core::ToMsgPack;
use algonaut_core::{Address, MultisigSignature};
use algonaut_core::{MicroAlgos, Round, VotePk, VrfPk};
use algonaut_crypto::HashDigest;
use algonaut_crypto::Signature;
use data_encoding::BASE32_NOPAD;
use sha2::Digest;

/// Enum containing the types of transactions and their specific fields
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TransactionType {
    Payment(Payment),
    KeyRegistration(KeyRegistration),
    AssetConfigurationTransaction(AssetConfigurationTransaction),
    AssetTransferTransaction(AssetTransferTransaction),
    AssetAcceptTransaction(AssetAcceptTransaction),
    AssetClawbackTransaction(AssetClawbackTransaction),
    AssetFreezeTransaction(AssetFreezeTransaction),
    ApplicationCallTransaction(ApplicationCallTransaction),
}

/// A transaction that can appear in a block
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Transaction {
    /// Paid by the sender to the FeeSink to prevent denial-of-service. The minimum fee on Algorand
    /// is currently 1000 microAlgos.
    pub fee: MicroAlgos,

    /// The first round for when the transaction is valid. If the transaction is sent prior to this
    /// round it will be rejected by the network.
    pub first_valid: Round,

    /// The hash of the genesis block of the network for which the transaction is valid. See the
    /// genesis hash for MainNet, TestNet, and BetaNet.
    pub genesis_hash: HashDigest,

    /// The ending round for which the transaction is valid. After this round, the transaction will
    /// be rejected by the network.
    pub last_valid: Round,

    /// Specifies the type of transaction. This value is automatically generated using any of the
    /// developer tools.
    pub txn_type: TransactionType,

    /// The human-readable string that identifies the network for the transaction. The genesis ID is
    /// found in the genesis block. See the genesis ID for MainNet, TestNet, and BetaNet.
    pub genesis_id: Option<String>,

    /// The group specifies that the transaction is part of a group and, if so, specifies the hash of
    /// the transaction group. Assign a group ID to a transaction through the workflow described in
    /// the Atomic Transfers Guide.
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
    pub lease: Option<HashDigest>,

    /// Any data up to 1000 bytes.
    pub note: Option<Vec<u8>>,

    /// Specifies the authorized address. This address will be used to authorize all future transactions.
    /// Learn more about Rekeying accounts.
    pub rekey_to: Option<Address>,
}

impl Transaction {
    pub fn bytes_to_sign(&self) -> Result<Vec<u8>, TransactionError> {
        let encoded_tx = self.to_owned().to_msg_pack()?;
        let mut prefix_encoded_tx = b"TX".to_vec();
        prefix_encoded_tx.extend_from_slice(&encoded_tx);
        Ok(prefix_encoded_tx)
    }

    pub fn raw_id(&self) -> Result<HashDigest, TransactionError> {
        let hashed = sha2::Sha512_256::digest(&self.bytes_to_sign()?);
        Ok(HashDigest(hashed.into()))
    }

    pub fn id(&self) -> Result<String, TransactionError> {
        Ok(BASE32_NOPAD.encode(&self.raw_id()?.0))
    }

    pub fn assign_group_id(&mut self, group_id: HashDigest) {
        self.group = Some(group_id)
    }

    // Estimates the size of the encoded transaction, used in calculating the fee
    pub fn estimate_size(&self) -> Result<u64, TransactionError> {
        let account = Account::generate();
        let signed_transaction = account.sign_transaction(self)?;
        Ok(signed_transaction.to_msg_pack()?.len() as u64)
    }

    pub fn estimate_fee(
        &self,
        fee_per_byte: MicroAlgos,
        min_fee: MicroAlgos,
    ) -> Result<MicroAlgos, TransactionError> {
        let total_fee = fee_per_byte * self.estimate_size()?;
        Ok(min_fee.max(total_fee))
    }

    pub fn estimate_fee_with_params(
        &self,
        params: &SuggestedTransactionParams,
    ) -> Result<MicroAlgos, TransactionError> {
        self.estimate_fee(params.fee_per_byte, params.min_fee)
    }

    /// The address of the account that signs and pays the fee.
    /// It can have additional responsibilities dependending on the transaction type.
    /// We keep it in the transaction types to be able to document (and possibly name) it separately.
    pub fn sender(&self) -> Address {
        match &self.txn_type {
            TransactionType::Payment(t) => t.sender,
            TransactionType::KeyRegistration(t) => t.sender,
            TransactionType::AssetConfigurationTransaction(t) => t.sender,
            TransactionType::AssetTransferTransaction(t) => t.sender,
            TransactionType::AssetAcceptTransaction(t) => t.sender,
            TransactionType::AssetClawbackTransaction(t) => t.sender,
            TransactionType::AssetFreezeTransaction(t) => t.sender,
            TransactionType::ApplicationCallTransaction(t) => t.sender,
        }
    }
}

/// Fields for a payment transaction
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Payment {
    /// The address of the account that signs, pays the fee and amount.
    pub sender: Address,

    /// The address of the account that receives the amount.
    pub receiver: Address,

    /// The total amount to be sent in microAlgos.
    pub amount: MicroAlgos,

    /// When set, it indicates that the transaction is requesting that the Sender account should
    /// be closed, and all remaining funds, after the fee and amount are paid, be transferred to
    /// this address.
    pub close_remainder_to: Option<Address>,
}

/// Fields for a key registration transaction
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct KeyRegistration {
    /// The address of the account that signs and pays the fee.
    pub sender: Address,

    /// The root participation public key. See Generate a Participation Key to learn more.
    pub vote_pk: Option<VotePk>,

    /// The VRF public key.
    pub selection_pk: Option<VrfPk>,

    /// The first round that the participation key is valid. Not to be confused with the FirstValid
    /// round of the keyreg transaction.
    pub vote_first: Option<Round>,

    /// The last round that the participation key is valid. Not to be confused with the LastValid
    /// round of the keyreg transaction.
    pub vote_last: Option<Round>,

    /// This is the dilution for the 2-level participation key.
    pub vote_key_dilution: Option<u64>,

    /// All new Algorand accounts are participating by default. This means that they earn rewards.
    /// Mark an account nonparticipating by setting this value to true and this account will no
    /// longer earn rewards. It is unlikely that you will ever need to do this and exists mainly
    /// for economic-related functions on the network.
    pub nonparticipating: Option<bool>,
}

/// This is used to create, configure and destroy an asset depending on which fields are set.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AssetConfigurationTransaction {
    /// The address of the account that signs and pays the fee.
    pub sender: Address,

    /// See AssetParams table for all available fields.
    pub params: Option<AssetParams>,
    /// For re-configure or destroy transactions, this is the unique asset ID. On asset creation,
    /// the ID is set to zero.
    /// NOTE: Algorand's REST documentation seems incorrect. The ID has to be not set for creation to work.
    pub config_asset: Option<u64>,
}

/// This is used to create or configure an asset.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AssetParams {
    /// The name of the asset. Supplied on creation. Example: Tether
    pub asset_name: Option<String>,
    /// The number of digits to use after the decimal point when displaying the asset. If 0,
    /// the asset is not divisible. If 1, the base unit of the asset is in tenths. If 2,
    /// the base unit of the asset is in hundredths.
    pub decimals: Option<u32>,
    /// True to freeze holdings for this asset by default.
    // #[serde(rename = "df", skip_serializing_if = "is_false")]
    pub default_frozen: Option<bool>,

    /// The total number of base units of the asset to create. This number cannot be changed.
    pub total: Option<u64>,

    /// The name of a unit of this asset. Supplied on creation. Example: USDT
    pub unit_name: Option<String>,

    /// This field is intended to be a 32-byte hash of some metadata that is relevant to your asset
    /// and/or asset holders. The format of this metadata is up to the application. This field can only
    /// be specified upon creation. An example might be the hash of some certificate that acknowledges
    /// the digitized asset as the official representation of a particular real-world asset.
    pub meta_data_hash: Option<Vec<u8>>,

    /// Specifies a URL where more information about the asset can be retrieved. Max size is 32 bytes.
    pub url: Option<String>,

    /// The address of the account that can clawback holdings of this asset. If empty, clawback is
    /// not permitted.
    pub clawback: Option<Address>,

    /// The address of the account used to freeze holdings of this asset. If empty, freezing is not
    /// permitted.
    pub freeze: Option<Address>,

    /// The address of the account that can manage the configuration of the asset and destroy it.
    pub manager: Option<Address>,
    /// The address of the account that holds the reserve (non-minted) units of the asset. This address
    /// has no specific authority in the protocol itself. It is used in the case where you want to
    /// signal to holders of your asset that the non-minted units of the asset reside in an account
    /// that is different from the default creator account (the sender).
    pub reserve: Option<Address>,
}

/// This is used to transfer an asset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetTransferTransaction {
    /// The address of the account that signs, pays the fee and sends the asset
    pub sender: Address,

    /// The unique ID of the asset to be transferred.
    pub xfer: u64,

    /// The amount of the asset to be transferred. A zero amount transferred to self allocates that
    /// asset in the account's Asset map.
    pub amount: u64,

    /// The recipient of the asset transfer.
    pub receiver: Address,

    /// Specify this field to remove the asset holding from the sender account and reduce the
    /// account's minimum balance.
    pub close_to: Option<Address>,
}

/// This is a special form of an Asset Transfer Transaction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetAcceptTransaction {
    /// The address of the account that signs, pays the fee and opts-in to the asset.
    pub sender: Address,

    /// The unique ID of the asset to be transferred.
    pub xfer: u64,
}

/// This is a special form of an Asset Transfer Transaction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetClawbackTransaction {
    /// The address of the account that signs, pays the fee and was set as clawback when creating the asset.
    pub sender: Address,

    /// The unique ID of the asset to be transferred.
    pub xfer: u64,

    /// The amount of the asset to be transferred.
    pub asset_amount: u64,

    /// The address from which the funds will be withdrawn.
    pub asset_sender: Address,

    /// The recipient of the asset transfer.
    pub asset_receiver: Address,

    /// Specify this field to remove the entire asset holding balance from the AssetSender
    /// account. It will not remove the asset holding.
    pub asset_close_to: Option<Address>,
}

/// This is a special form of an Asset Transfer Transaction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetFreezeTransaction {
    /// The address of the account that signs, pays the fee and was set as freeze when creating the asset.
    pub sender: Address,

    /// The address of the account whose asset is being frozen or unfrozen.
    pub freeze_account: Address,

    /// The asset ID being frozen or unfrozen.
    pub asset_id: u64,

    /// True to freeze the asset.
    pub frozen: bool,
}

///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationCallTransaction {
    /// The address of the account that signs and pays the fee.
    pub sender: Address,

    /// ID of the application being configured or empty if creating.
    pub app_id: Option<u64>,

    /// Defines what additional actions occur with the transaction. See the OnComplete section of
    /// the TEAL spec for details.
    pub on_complete: ApplicationCallOnComplete,

    /// List of accounts in addition to the sender that may be accessed from the application's
    /// approval-program and clear-state-program.
    pub accounts: Option<Vec<Address>>,

    /// Logic executed for every application transaction, except when on-completion is set to
    /// "clear". It can read and write global state for the application, as well as account-specific
    /// local state. Approval programs may reject the transaction.
    pub approval_program: Option<CompiledTeal>,

    /// Transaction specific arguments accessed from the application's approval-program and
    /// clear-state-program.
    pub app_arguments: Option<Vec<Vec<u8>>>,

    /// Logic executed for application transactions with on-completion set to "clear". It can read
    /// and write global state for the application, as well as account-specific local state. Clear
    /// state programs cannot reject the transaction.
    pub clear_state_program: Option<CompiledTeal>,

    /// Lists the applications in addition to the application-id whose global states may be accessed
    /// by this application's approval-program and clear-state-program. The access is read-only.
    pub foreign_apps: Option<Vec<u64>>,

    /// Lists the assets whose AssetParams may be accessed by this application's approval-program and
    /// clear-state-program. The access is read-only.
    pub foreign_assets: Option<Vec<u64>>,

    /// Holds the maximum number of global state values defined within a StateSchema object.
    pub global_state_schema: Option<StateSchema>,

    /// Holds the maximum number of local state values defined within a StateSchema object.
    pub local_state_schema: Option<StateSchema>,

    // Number of additional pages allocated to the application's approval and clear state programs. Each ExtraProgramPages is 2048 bytes. The sum of ApprovalProgram and ClearStateProgram may not exceed 2048*(1+ExtraProgramPages) bytes.
    pub extra_pages: u64,
}

/// An application transaction must indicate the action to be taken following the execution of its approvalProgram or clearStateProgram. The variants below describe the available actions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationCallOnComplete {
    /// Only execute the ApprovalProgram associated with this application ID, with no additional effects.
    NoOp,
    /// Before executing the ApprovalProgram, allocate local state for this application into the sender's account data.
    OptIn,
    /// After executing the ApprovalProgram, clear any local state for this application out of the sender's account data.
    CloseOut,
    /// Don't execute the ApprovalProgram, and instead execute the ClearStateProgram (which may not reject this transaction). Additionally, clear any local state for this application out of the sender's account data as in CloseOutOC.
    ClearState,
    /// After executing the ApprovalProgram, replace the ApprovalProgram and ClearStateProgram associated with this application ID with the programs specified in this transaction.
    UpdateApplication,
    /// After executing the ApprovalProgram, delete the application parameters from the account data of the application's creator.
    DeleteApplication,
}

/// Storage state schema. The StateSchema object is only required for the create application call
/// transaction. The StateSchema object must be fully populated for both the GlobalStateSchema and
/// LocalStateSchema objects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateSchema {
    /// Maximum number of integer values that may be stored in the [global || local] application
    /// key/value store. Immutable.
    pub number_ints: u64,

    /// Maximum number of byte slices values that may be stored in the [global || local] application
    /// key/value store. Immutable.
    pub number_byteslices: u64,
}

/// Wraps a transaction in a signature. The encoding of this struct is suitable to be broadcast
/// on the network
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedTransaction {
    pub transaction: Transaction,
    pub transaction_id: String,
    pub sig: TransactionSignature,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransactionSignature {
    Single(Signature),
    Multi(MultisigSignature),
    Logic(SignedLogic),
}
