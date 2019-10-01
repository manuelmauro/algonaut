use serde::{Deserialize, Serialize, Serializer};

use crate::account::Account;
use crate::crypto::{Address, MultisigSignature, Signature};
use crate::{Error, HashDigest, MicroAlgos, Round, VotePK, VRFPK};

const MIN_TXN_FEE: MicroAlgos = MicroAlgos(1000);

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

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(tag = "type")]
pub enum TransactionType {
    #[serde(rename = "pay")]
    Payment(Payment),
    #[serde(rename = "keyreg")]
    KeyRegistration(KeyRegistration),
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub struct Payment {
    #[serde(rename = "amt", default)]
    amount: MicroAlgos,
    #[serde(rename = "rcv")]
    receiver: Address,
    #[serde(rename = "close")]
    close_remainder_to: Option<Address>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub struct KeyRegistration {
    #[serde(rename = "votekey")]
    vote_pk: VotePK,
    #[serde(rename = "selkey")]
    selection_pk: VRFPK,
    #[serde(rename = "votefst")]
    vote_first: Round,
    #[serde(rename = "votelst")]
    vote_last: Round,
    #[serde(rename = "votekd")]
    vote_key_dilution: u64,
}

impl Transaction {
    pub fn new_payment(
        sender: Address,
        fee_per_byte: MicroAlgos,
        first_valid: Round,
        last_valid: Round,
        note: Vec<u8>,
        genesis_id: &str,
        genesis_hash: HashDigest,
        receiver: Address,
        amount: MicroAlgos,
        close_remainder_to: Option<Address>,
    ) -> Result<Transaction, Error> {
        let payment = Payment {
            amount,
            receiver,
            close_remainder_to,
        };
        let mut transaction = Transaction {
            sender,
            fee: MicroAlgos(0),
            first_valid,
            last_valid,
            note,
            genesis_id: genesis_id.to_string(),
            genesis_hash,
            txn_type: TransactionType::Payment(payment),
        };
        transaction.fee = MIN_TXN_FEE.max(fee_per_byte * transaction.estimate_size()?);
        Ok(transaction)
    }
    pub fn new_payment_flat_fee(
        sender: Address,
        fee: MicroAlgos,
        first_valid: Round,
        last_valid: Round,
        note: Vec<u8>,
        genesis_id: &str,
        genesis_hash: HashDigest,
        receiver: Address,
        amount: MicroAlgos,
        close_remainder_to: Option<Address>,
    ) -> Transaction {
        let payment = Payment {
            amount,
            receiver,
            close_remainder_to,
        };
        let transaction = Transaction {
            sender,
            fee,
            first_valid,
            last_valid,
            note,
            genesis_id: genesis_id.to_string(),
            genesis_hash,
            txn_type: TransactionType::Payment(payment),
        };
        transaction
    }
    pub fn new_key_registration(
        sender: Address,
        fee_per_byte: MicroAlgos,
        first_valid: Round,
        last_valid: Round,
        note: Vec<u8>,
        genesis_id: &str,
        genesis_hash: HashDigest,
        vote_pk: VotePK,
        selection_pk: VRFPK,
        vote_first: Round,
        vote_last: Round,
        vote_key_dilution: u64,
    ) -> Result<Transaction, Error> {
        let key_registration = KeyRegistration {
            vote_pk,
            selection_pk,
            vote_first,
            vote_last,
            vote_key_dilution,
        };
        let mut transaction = Transaction {
            sender,
            fee: MicroAlgos(0),
            first_valid,
            last_valid,
            note,
            genesis_id: genesis_id.to_string(),
            genesis_hash,
            txn_type: TransactionType::KeyRegistration(key_registration),
        };
        transaction.fee = MIN_TXN_FEE.max(fee_per_byte * transaction.estimate_size()?);
        Ok(transaction)
    }

    fn estimate_size(&self) -> Result<u64, Error> {
        let account = Account::generate();
        let len = rmp_serde::to_vec_named(&account.sign_transaction(self)?)?.len() as u64;
        Ok(len)
    }
}

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
