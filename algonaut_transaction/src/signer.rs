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

use std::future::Future;
use std::pin::Pin;

use algonaut_core::{MultisigAddress, MultisigSignature};

use crate::account::Account;
use crate::contract_account::ContractAccount;
use crate::error::TransactionError;
use crate::transaction::{SignedTransaction, Transaction, TransactionSignature};

/// The future returned by [`Signer::sign_transactions`].
///
/// Spelled out as an explicit boxed future rather than `async fn` in the
/// trait so that `Signer` stays object safe (the composer stores signers
/// behind `Arc<dyn Signer>`, and native `async fn` in traits is
/// `dyn`-incompatible). The future is `Send` so that the composer's
/// signing step stays `Send` on multi-threaded executors.
pub type SigningFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Vec<SignedTransaction>, TransactionError>> + Send + 'a>>;

/// What a [`Signer`] is asked to sign: the whole group, plus the slots it
/// owns.
///
/// `transactions` is the full, group-id-stamped array, so a remote wallet
/// can display the entire atomic group; `indexes` are the positions in it
/// this signer must produce signatures for. Group-awareness lets a
/// WalletConnect-style signer sign every slot it controls in a single
/// approval round-trip while still seeing the full context.
pub struct SigningRequest<'a> {
    /// The full group, after group ids have been assigned.
    pub transactions: &'a [Transaction],
    /// Positions in `transactions` this signer is expected to sign.
    pub indexes: &'a [usize],
}

/// Anything that can sign the requested slots of an atomic group,
/// possibly awaiting remote I/O or user approval. Implementors must be
/// `Send + Sync` so the atomic transaction composer can hand them out
/// across threads, and `Debug` so the composer keeps its derived `Debug`
/// impl.
pub trait Signer: std::fmt::Debug + Send + Sync {
    /// Sign the slots named by `request.indexes`, returning one
    /// [`SignedTransaction`] per requested index, in `request.indexes`
    /// order. Implementors should return an error if any requested
    /// transaction can't be signed rather than producing a partial
    /// result.
    fn sign_transactions<'a>(&'a self, request: SigningRequest<'a>) -> SigningFuture<'a>;
}

impl Signer for Account {
    fn sign_transactions<'a>(&'a self, request: SigningRequest<'a>) -> SigningFuture<'a> {
        Box::pin(async move {
            request
                .indexes
                .iter()
                .map(|&i| self.sign_transaction(request.transactions[i].clone()))
                .collect()
        })
    }
}

impl Signer for ContractAccount {
    fn sign_transactions<'a>(&'a self, request: SigningRequest<'a>) -> SigningFuture<'a> {
        Box::pin(async move {
            request
                .indexes
                .iter()
                // The previous `TransactionSigner::ContractAccount` enum
                // variant always signed with an empty program-args list.
                // Callers that need non-empty args should call
                // `ContractAccount::sign` directly.
                .map(|&i| self.sign(request.transactions[i].clone(), vec![]))
                .collect()
        })
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
    fn sign_transactions<'a>(&'a self, request: SigningRequest<'a>) -> SigningFuture<'a> {
        Box::pin(async move {
            request
                .indexes
                .iter()
                .map(|&i| {
                    sign_msig_tx(
                        &self.address,
                        &self.accounts,
                        request.transactions[i].clone(),
                    )
                })
                .collect()
        })
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
