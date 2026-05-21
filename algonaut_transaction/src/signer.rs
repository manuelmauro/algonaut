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

use algonaut_core::{MultisigAddress, MultisigSignature};

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
    let mut session_accounts = accounts.iter();
    let first_account = session_accounts
        .next()
        .ok_or(TransactionError::NoAccountsToSign)?;
    let mut session = MultisigSigningSession::new(address.clone()).sign(tx, first_account)?;
    for account in session_accounts {
        session = session.sign_more(account)?;
    }
    session.finish()
}

/// Fluent builder that accumulates subsignatures from multiple
/// [`Account`]s and finalises them into a [`SignedTransaction`].
///
/// Use this for one-off multisig signing flows where the signers are
/// known up front; for repeated signing inside the atomic transaction
/// composer use the [`MultisigSigner`] [`Signer`] implementation
/// instead.
///
/// ```ignore
/// use algonaut_transaction::signer::MultisigSigningSession;
/// use algonaut_core::MultisigAddress;
///
/// let multisig_address = MultisigAddress::new(1, 2, &[alice.address(), bob.address()])?;
/// let signed = MultisigSigningSession::new(multisig_address)
///     .sign(txn, &alice)?
///     .sign_more(&bob)?
///     .finish()?;
/// ```
#[derive(Debug, Clone)]
pub struct MultisigSigningSession {
    address: MultisigAddress,
}

impl MultisigSigningSession {
    /// Begin a new signing session against `address`. No accounts have
    /// signed yet; transition into the in-progress state by calling
    /// [`sign`](Self::sign).
    pub fn new(address: MultisigAddress) -> Self {
        Self { address }
    }

    /// Initialise the session with the first signer. The session
    /// transitions into [`InProgressMultisigSigningSession`], which
    /// holds the transaction plus the partial multisig signature.
    pub fn sign(
        self,
        transaction: Transaction,
        account: &Account,
    ) -> Result<InProgressMultisigSigningSession, TransactionError> {
        let msig = account.init_transaction_msig(&transaction, &self.address)?;
        Ok(InProgressMultisigSigningSession {
            address: self.address,
            transaction,
            msig,
        })
    }
}

/// A [`MultisigSigningSession`] that has at least one signer attached.
/// Add more signers with [`sign_more`](Self::sign_more) and finalise
/// with [`finish`](Self::finish).
#[derive(Debug, Clone)]
pub struct InProgressMultisigSigningSession {
    address: MultisigAddress,
    transaction: Transaction,
    msig: MultisigSignature,
}

impl InProgressMultisigSigningSession {
    /// Add another subsignature from `account` to the in-progress
    /// multisig signature.
    pub fn sign_more(mut self, account: &Account) -> Result<Self, TransactionError> {
        self.msig = account.append_to_transaction_msig(&self.transaction, self.msig)?;
        Ok(self)
    }

    /// Finalise the session into a [`SignedTransaction`]. The
    /// transaction id is computed from the carried transaction.
    pub fn finish(self) -> Result<SignedTransaction, TransactionError> {
        Ok(SignedTransaction {
            transaction_id: self.transaction.id()?,
            transaction: self.transaction,
            sig: TransactionSignature::Multi(self.msig),
            auth_address: None,
        })
    }

    /// The multisig address being signed against.
    pub fn address(&self) -> &MultisigAddress {
        &self.address
    }

    /// The transaction being signed.
    pub fn transaction(&self) -> &Transaction {
        &self.transaction
    }
}
