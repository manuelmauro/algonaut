//! Pluggable transaction signing.
//!
//! The [`Signer`] trait abstracts over how a [`Transaction`] is turned
//! into a [`SignedTransaction`]. Concrete impls are provided for the
//! built-in signer types ([`Account`], [`ContractAccount`], and the
//! [`MultisigSigner`] helper bundle), and third parties can implement
//! the trait against HSMs, remote KMS endpoints, hardware wallets, or
//! anything else.
//!
//! The trait is object-safe: callers typically store an
//! `Arc<dyn Signer>` so that the same signer can be cheaply shared
//! between multiple `TransactionWithSigner` values inside an atomic
//! group.

use algonaut_core::MultisigAddress;

use crate::account::Account;
use crate::contract_account::ContractAccount;
use crate::error::TransactionError;
use crate::transaction::{SignedTransaction, Transaction, TransactionSignature};

/// Anything that can turn a slice of unsigned [`Transaction`]s into a
/// matching `Vec<SignedTransaction>`. Implementors must be `Send +
/// Sync` so the atomic transaction composer can hand them out across
/// threads, and `Debug` so the composer keeps its derived `Debug` impl.
pub trait Signer: std::fmt::Debug + Send + Sync {
    /// Sign every transaction in `txs`, returning one
    /// [`SignedTransaction`] per input in the same order. Implementors
    /// should return an error if any single transaction can't be signed
    /// rather than producing a partial result.
    fn sign_transactions(
        &self,
        txs: &[Transaction],
    ) -> Result<Vec<SignedTransaction>, TransactionError>;
}

impl Signer for Account {
    fn sign_transactions(
        &self,
        txs: &[Transaction],
    ) -> Result<Vec<SignedTransaction>, TransactionError> {
        let mut signed = Vec::with_capacity(txs.len());
        for tx in txs {
            signed.push(self.sign_transaction(tx.clone())?);
        }
        Ok(signed)
    }
}

impl Signer for ContractAccount {
    fn sign_transactions(
        &self,
        txs: &[Transaction],
    ) -> Result<Vec<SignedTransaction>, TransactionError> {
        let mut signed = Vec::with_capacity(txs.len());
        for tx in txs {
            // The previous `TransactionSigner::ContractAccount` enum
            // variant always signed with an empty program-args list.
            // Callers that need non-empty args should call
            // `ContractAccount::sign` directly.
            signed.push(self.sign(tx.clone(), vec![])?);
        }
        Ok(signed)
    }
}

/// Bundles a [`MultisigAddress`] with the [`Account`] signers that
/// together control it, so the pair can be used as a single
/// [`Signer`]. The composer hands every transaction to the same set of
/// accounts in order, mirroring the old
/// `TransactionSigner::MultisigAccount` enum variant.
#[derive(Debug, Clone)]
pub struct MultisigSigner {
    /// The multisig identity whose accounts produce the signatures.
    pub address: MultisigAddress,
    /// The accounts contributing subsignatures, in deterministic order.
    pub accounts: Vec<Account>,
}

impl MultisigSigner {
    pub fn new(address: MultisigAddress, accounts: Vec<Account>) -> Self {
        Self { address, accounts }
    }
}

impl Signer for MultisigSigner {
    fn sign_transactions(
        &self,
        txs: &[Transaction],
    ) -> Result<Vec<SignedTransaction>, TransactionError> {
        let mut signed = Vec::with_capacity(txs.len());
        for tx in txs {
            signed.push(sign_msig_tx(&self.address, &self.accounts, tx.clone())?);
        }
        Ok(signed)
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
