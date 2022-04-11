use algonaut_core::MultisigAddress;
use algonaut_transaction::{
    account::Account, contract_account::ContractAccount, error::TransactionError,
    transaction::TransactionSignature, SignedTransaction, Transaction,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionSigner {
    BasicAccount(Account),
    ContractAccount(ContractAccount),
    MultisigAccount {
        address: MultisigAddress,
        accounts: Vec<Account>,
    },
}

impl TransactionSigner {
    pub fn sign_transactions(
        &self,
        tx_group: Vec<Transaction>,
    ) -> Result<Vec<SignedTransaction>, TransactionError> {
        match self {
            TransactionSigner::BasicAccount(account) => {
                let mut signed_txs = vec![];
                for tx in tx_group {
                    signed_txs.push(account.sign_transaction(tx)?);
                }
                Ok(signed_txs)
            }

            TransactionSigner::ContractAccount(account) => {
                let mut signed_txs = vec![];
                for tx in tx_group {
                    signed_txs.push(account.sign(tx, vec![])?);
                }
                Ok(signed_txs)
            }

            TransactionSigner::MultisigAccount { address, accounts } => {
                let mut signed_txs = vec![];
                for tx in tx_group {
                    signed_txs.push(sign_msig_tx(address, accounts, tx)?);
                }
                Ok(signed_txs)
            }
        }
    }
}

fn sign_msig_tx(
    address: &MultisigAddress,
    accounts: &[Account],
    tx: Transaction,
) -> Result<SignedTransaction, TransactionError> {
    if let Some(first_account) = accounts.first() {
        let mut msig = first_account.init_transaction_msig(&tx, address)?;
        for account in &accounts[1..accounts.len()] {
            msig = account.append_to_transaction_msig(&tx, msig)?;
        }

        let signed_t = SignedTransaction {
            transaction_id: tx.id()?,
            transaction: tx,
            sig: TransactionSignature::Multi(msig),
        };

        Ok(signed_t)
    } else {
        Err(TransactionError::NoAccountsToSign)
    }
}
