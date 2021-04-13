use crate::{
    transaction::{Transaction, TransactionType},
    Payment,
};
use algonaut_core::Address;
use algonaut_core::{MicroAlgos, Round};
use algonaut_crypto::HashDigest;

#[derive(Default)]
pub struct Txn {
    fee: MicroAlgos,
    first_valid: Round,
    genesis_hash: Option<HashDigest>,
    last_valid: Round,
    sender: Option<Address>,
    txn_type: Option<TransactionType>,
    genesis_id: String,
    group: Option<HashDigest>,
    lease: Option<HashDigest>,
    note: Vec<u8>,
    rekey_to: Option<Address>,
}

impl Txn {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fee(mut self, fee: MicroAlgos) -> Self {
        self.fee = fee;
        self
    }

    pub fn first_valid(mut self, first_valid: Round) -> Self {
        self.first_valid = first_valid;
        self
    }

    pub fn genesis_hash(mut self, genesis_hash: HashDigest) -> Self {
        self.genesis_hash = Some(genesis_hash);
        self
    }

    pub fn last_valid(mut self, last_valid: Round) -> Self {
        self.last_valid = last_valid;
        self
    }

    pub fn sender(mut self, sender: Address) -> Self {
        self.sender = Some(sender);
        self
    }

    pub fn txn_type(mut self, txn_type: TransactionType) -> Self {
        self.txn_type = Some(txn_type);
        self
    }

    pub fn payment(mut self, payment: Payment) -> Self {
        self.txn_type = Some(TransactionType::Payment(payment));
        self
    }

    pub fn genesis_id(mut self, genesis_id: String) -> Self {
        self.genesis_id = genesis_id;
        self
    }

    pub fn group(mut self, group: HashDigest) -> Self {
        self.group = Some(group);
        self
    }

    pub fn lease(mut self, lease: HashDigest) -> Self {
        self.lease = Some(lease);
        self
    }

    pub fn note(mut self, note: Vec<u8>) -> Self {
        self.note = note;
        self
    }

    pub fn rekey_to(mut self, rekey_to: Address) -> Self {
        self.rekey_to = Some(rekey_to);
        self
    }

    pub fn build(self) -> Transaction {
        Transaction {
            fee: self.fee,
            first_valid: self.first_valid,
            genesis_hash: self.genesis_hash.unwrap(),
            last_valid: self.last_valid,
            sender: self.sender.unwrap(),
            txn_type: self.txn_type.unwrap(),
            genesis_id: self.genesis_id,
            group: self.group,
            lease: self.lease,
            note: self.note,
            rekey_to: self.rekey_to,
        }
    }
}

#[derive(Default)]
pub struct Pay {
    receiver: Option<Address>,
    amount: MicroAlgos,
    close_remainder_to: Option<Address>,
}

impl Pay {
    pub fn new() -> Self {
        Pay::default()
    }

    pub fn to(mut self, receiver: Address) -> Self {
        self.receiver = Some(receiver);
        self
    }

    pub fn amount(mut self, amount: MicroAlgos) -> Self {
        self.amount = amount;
        self
    }

    pub fn close_remainder_to(mut self, close_remainder_to: Address) -> Self {
        self.close_remainder_to = Some(close_remainder_to);
        self
    }

    pub fn build(self) -> Payment {
        Payment {
            receiver: self.receiver.unwrap(),
            amount: self.amount,
            close_remainder_to: self.close_remainder_to,
        }
    }
}
