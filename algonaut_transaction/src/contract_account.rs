use crate::error::TransactionError;
use crate::transaction::{SignedTransaction, Transaction, TransactionSignature};
use algonaut_core::{Address, CompiledTeal, LogicSignature, SignedLogic};
use serde::{Deserialize, Serialize};

/// Convenience CompiledTeal "view", used to sign as contract account.
/// The program hash is interpreted as an address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractAccount {
    address: Address,
    pub program: CompiledTeal,
}

impl ContractAccount {
    pub fn new(program: CompiledTeal) -> ContractAccount {
        ContractAccount {
            address: program.hash().into(),
            program,
        }
    }

    pub fn address(&self) -> &Address {
        &self.address
    }

    pub fn sign(
        &self,
        transaction: Transaction,
        args: Vec<Vec<u8>>,
    ) -> Result<SignedTransaction, TransactionError> {
        Ok(SignedTransaction {
            transaction_id: transaction.id()?,
            transaction,
            sig: TransactionSignature::Logic(SignedLogic {
                logic: self.program.clone(),
                args,
                sig: LogicSignature::ContractAccount,
            }),
        })
    }
}
