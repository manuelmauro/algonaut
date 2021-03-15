use crate::account::Account;
use crate::crypto::{Address, MultisigSignature, Signature};
use crate::error::AlgorandError;
use crate::models::{HashDigest, MicroAlgos, Round, VotePk, VrfPk};
use serde::{Deserialize, Serialize, Serializer};

const MIN_TXN_FEE: MicroAlgos = MicroAlgos(1000);

/// Fields always used when creating a transaction, used as an argument in creating a Transaction
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BaseTransaction {
    pub sender: Address,
    pub first_valid: Round,
    pub last_valid: Round,
    pub note: Vec<u8>,
    pub genesis_id: String,
    pub genesis_hash: HashDigest,
}

/// Enum containing the types of transactions and their specific fields
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(tag = "type")]
pub enum TransactionType {
    #[serde(rename = "pay")]
    Payment(Payment),
    #[serde(rename = "keyreg")]
    KeyRegistration(KeyRegistration),
}

/// A transaction that can appear in a block
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub struct Transaction {
    #[serde(rename = "snd")]
    pub sender: Address,
    #[serde(rename = "fee")]
    pub fee: MicroAlgos,
    #[serde(rename = "fv")]
    pub first_valid: Round,
    #[serde(rename = "lv")]
    pub last_valid: Round,
    #[serde(with = "serde_bytes", default)]
    pub note: Vec<u8>,
    #[serde(rename = "gen", default)]
    pub genesis_id: String,
    #[serde(rename = "gh")]
    pub genesis_hash: HashDigest,
    #[serde(flatten)]
    pub txn_type: TransactionType,
}

impl Serialize for Transaction {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let type_len = match &self.txn_type {
            TransactionType::Payment(payment) => {
                1 + if payment.close_remainder_to.is_some() {
                    1
                } else {
                    0
                } + if payment.amount.0 != 0 { 1 } else { 0 }
            }
            TransactionType::KeyRegistration(_) => 5,
        };
        let len = 6
            + type_len
            + if self.note.is_empty() { 0 } else { 1 }
            + if self.genesis_id.is_empty() { 0 } else { 1 };
        let mut state = serializer.serialize_struct("Transaction", len)?;
        if let TransactionType::Payment(payment) = &self.txn_type {
            if payment.amount.0 != 0 {
                state.serialize_field("amt", &payment.amount)?;
            }
        }
        if let TransactionType::Payment(payment) = &self.txn_type {
            if payment.close_remainder_to.is_some() {
                state.serialize_field("close", &payment.close_remainder_to)?;
            }
        }
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
        if let TransactionType::Payment(payment) = &self.txn_type {
            state.serialize_field("rcv", &payment.receiver)?;
        }
        if let TransactionType::KeyRegistration(key_registration) = &self.txn_type {
            state.serialize_field("selkey", &key_registration.selection_pk)?;
        }
        state.serialize_field("snd", &self.sender)?;
        match &self.txn_type {
            TransactionType::Payment(_payment) => {
                state.serialize_field("type", "pay")?;
            }
            TransactionType::KeyRegistration(_key_registration) => {
                state.serialize_field("type", "keyreg")?;
            }
        }
        if let TransactionType::KeyRegistration(key_registration) = &self.txn_type {
            state.serialize_field("votefst", &key_registration.vote_first)?;
        }
        if let TransactionType::KeyRegistration(key_registration) = &self.txn_type {
            state.serialize_field("votekd", &key_registration.vote_key_dilution)?;
        }
        if let TransactionType::KeyRegistration(key_registration) = &self.txn_type {
            state.serialize_field("votekey", &key_registration.vote_pk)?;
        }
        if let TransactionType::KeyRegistration(key_registration) = &self.txn_type {
            state.serialize_field("votelst", &key_registration.vote_last)?;
        }
        state.end()
    }
}

/// Fields for a payment transaction
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub struct Payment {
    #[serde(rename = "amt", default)]
    pub amount: MicroAlgos,
    #[serde(rename = "rcv")]
    pub receiver: Address,
    /// When set, it indicates the transaction is requesting the account should be closed,
    /// and all remaining funds be transferred to this address.
    #[serde(rename = "close")]
    pub close_remainder_to: Option<Address>,
}

/// Fields for a key registration transaction
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub struct KeyRegistration {
    #[serde(rename = "votekey")]
    pub vote_pk: VotePk,
    #[serde(rename = "selkey")]
    pub selection_pk: VrfPk,
    #[serde(rename = "votefst")]
    pub vote_first: Round,
    #[serde(rename = "votelst")]
    pub vote_last: Round,
    #[serde(rename = "votekd")]
    pub vote_key_dilution: u64,
}

impl Transaction {
    /// Creates a new transaction with a fee calculated based on `fee_per_byte`.
    pub fn new(
        base: BaseTransaction,
        fee_per_byte: MicroAlgos,
        txn_type: TransactionType,
    ) -> Result<Transaction, AlgorandError> {
        let mut transaction = Transaction {
            sender: base.sender,
            fee: MicroAlgos(0),
            first_valid: base.first_valid,
            last_valid: base.last_valid,
            note: base.note,
            genesis_id: base.genesis_id,
            genesis_hash: base.genesis_hash,
            txn_type,
        };
        transaction.fee = MIN_TXN_FEE.max(fee_per_byte * transaction.estimate_size()?);
        Ok(transaction)
    }

    /// Creates a nw transaction with the specified fee.
    pub fn new_flat_fee(
        base: BaseTransaction,
        fee: MicroAlgos,
        txn_type: TransactionType,
    ) -> Transaction {
        Transaction {
            sender: base.sender,
            fee,
            first_valid: base.first_valid,
            last_valid: base.last_valid,
            note: base.note,
            genesis_id: base.genesis_id,
            genesis_hash: base.genesis_hash,
            txn_type,
        }
    }

    // Estimates the size of the encoded transaction, used in calculating the fee
    fn estimate_size(&self) -> Result<u64, AlgorandError> {
        let account = Account::generate();
        let len = rmp_serde::to_vec_named(&account.sign_transaction(self)?)?.len() as u64;
        Ok(len)
    }
}

/// Wraps a transaction in a signature. The encoding of this struct is suitable to be broadcast on the network
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SignedTransaction {
    #[serde(rename = "msig", skip_serializing_if = "Option::is_none")]
    pub multisig: Option<MultisigSignature>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<Signature>,
    #[serde(rename = "txn")]
    pub transaction: Transaction,
    #[serde(skip)]
    pub transaction_id: String,
}
