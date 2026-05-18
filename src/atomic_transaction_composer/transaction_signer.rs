use algonaut_core::MultisigAddress;
use algonaut_crypto::Signature;
use algonaut_transaction::{
    SignedTransaction, Transaction, account::Account, contract_account::ContractAccount,
    error::TransactionError, transaction::TransactionSignature,
};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionSigner {
    BasicAccount(Account),
    ContractAccount(ContractAccount),
    MultisigAccount {
        address: MultisigAddress,
        accounts: Vec<Account>,
    },
    /// Produces a `SignedTransaction` whose signature is the all-zero
    /// 64-byte placeholder — algod's simulator detects this as a
    /// missing signature and reports `signedtxn has no sig`. Useful for
    /// the `/v2/transactions/simulate` "allow-empty-signatures = false"
    /// scenarios; do not use against the live submit endpoint.
    Empty,
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

            TransactionSigner::Empty => {
                let mut signed_txs = vec![];
                for tx in tx_group {
                    let transaction_id = tx.id()?;
                    signed_txs.push(SignedTransaction {
                        transaction: tx,
                        transaction_id,
                        sig: TransactionSignature::Single(Signature([0; 64])),
                        auth_address: None,
                    });
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
            auth_address: None,
        };

        Ok(signed_t)
    } else {
        Err(TransactionError::NoAccountsToSign)
    }
}
