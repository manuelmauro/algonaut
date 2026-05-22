//! The atomic-group typestate chain.
//!
//! An atomic group advances through three types — calls that don't make
//! sense in a given state simply don't compile (see the
//! `atomic-transaction-composer-typestate` ADR):
//!
//! [`AtomicGroupBuilder`] (accepts `add_*`) →
//! [`UnsignedAtomicGroup`] (group ids stamped; sign or simulate) →
//! [`SignedAtomicGroup`] (submit or execute).

use std::collections::HashMap;
use std::sync::Arc;

use algonaut_abi::abi_interactions::{AbiMethod, TransactionArgType};
use algonaut_algod::models::{
    SimulateRequest, SimulateRequestTransactionGroup, SimulateTransaction200Response,
};
use algonaut_transaction::{SignedTransaction, Signer, Transaction, tx_group};

use crate::{
    Error,
    algod::v2::{Algod, PendingSubmission},
    simulate::SimulateResponse,
};

use super::MAX_ATOMIC_GROUP_SIZE;
use super::encode::{process_method_call, validate_tx};
use super::method_call::MethodCall;
use super::outcome::{
    AbiMethodResult, AbiReturnDecodeError, ExecuteOutcome, SimulateOutcome,
    get_return_value_with_return_type,
};
use super::signing::{placeholder_group, poll_until_confirmed, sign_group, tx_ids};

/// Represents an unsigned transaction and a signer that can authorize that transaction.
///
/// `signer` is optional. `None` marks a **simulate-only** slot: it has no
/// signer, so [`UnsignedAtomicGroup::sign`] rejects it with
/// [`Error::MissingSigner`], but
/// [`simulate`](UnsignedAtomicGroup::simulate) accepts it (simulate
/// placeholder-signs every slot and asks algod to allow empty
/// signatures). Use it to dry-run a transaction you cannot sign — e.g.
/// one sent from an account you do not control.
#[derive(Debug, Clone)]
pub struct TransactionWithSigner {
    /// An unsigned transaction
    pub tx: Transaction,
    /// A transaction signer that can authorize the transaction, or
    /// `None` for a simulate-only slot.
    pub signer: Option<Arc<dyn Signer>>,
}

impl TransactionWithSigner {
    /// Build a `TransactionWithSigner` from a transaction and a signer
    /// that will authorize it.
    pub fn new(tx: Transaction, signer: Arc<dyn Signer>) -> Self {
        Self {
            tx,
            signer: Some(signer),
        }
    }

    /// Build a `TransactionWithSigner` with no signer attached, for
    /// **simulate only**. [`UnsignedAtomicGroup::sign`] rejects a group
    /// containing such a slot ([`Error::MissingSigner`]); only
    /// [`simulate`](UnsignedAtomicGroup::simulate) accepts it.
    pub fn unsigned(tx: Transaction) -> Self {
        Self { tx, signer: None }
    }
}

/// A pending entry in an [`AtomicGroupBuilder`]. Entries are only assembled and
/// validated when [`AtomicGroupBuilder::build`] is called, so the `add_*`
/// methods can stay infallible.
#[derive(Debug, Clone)]
enum AtomicGroupEntry {
    Transaction(TransactionWithSigner),
    MethodCall(MethodCall),
}

/// Builder state of an atomic transaction group.
///
/// Add pre-built transactions and ABI method calls in any mix, then
/// [`build`](AtomicGroupBuilder::build) to stamp the group id and advance to
/// the [`UnsignedAtomicGroup`] state. The `add_*` methods are infallible and
/// only record intent; all validation — group-size limit, ABI argument
/// counts, per-transaction checks — happens in `build`.
///
/// `AtomicGroupBuilder` is `Clone`: clone it to snapshot a common prefix and
/// build several groups from it (this replaces the old
/// `clone_composer`).
#[derive(Debug, Clone, Default)]
pub struct AtomicGroupBuilder {
    entries: Vec<AtomicGroupEntry>,
}

impl AtomicGroupBuilder {
    /// Start a new, empty group builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a pre-built transaction with its signer to the group.
    pub fn add_transaction(mut self, txn_with_signer: TransactionWithSigner) -> Self {
        self.entries
            .push(AtomicGroupEntry::Transaction(txn_with_signer));
        self
    }

    /// Add an ABI method call to the group. Build the [`MethodCall`]
    /// with [`MethodCall::builder`] and the [`MethodCallBuilder`](super::MethodCallBuilder) setters.
    pub fn add_method_call(mut self, call: MethodCall) -> Self {
        self.entries.push(AtomicGroupEntry::MethodCall(call));
        self
    }

    /// Finalize the group: assemble every entry, enforce the size and
    /// ABI-argument invariants, stamp the group id, and produce an
    /// [`UnsignedAtomicGroup`].
    ///
    /// Returns [`Error::EmptyTransactionGroup`] if no entries were
    /// added, or [`Error::ComposerGroupFull`] if the assembled group
    /// would exceed the protocol's 16-transaction limit.
    pub fn build(self) -> Result<UnsignedAtomicGroup, Error> {
        let mut txs: Vec<TransactionWithSigner> = Vec::new();
        let mut method_map: HashMap<usize, AbiMethod> = HashMap::new();

        for entry in self.entries {
            match entry {
                AtomicGroupEntry::Transaction(txn_with_signer) => {
                    if txs.len() == MAX_ATOMIC_GROUP_SIZE {
                        return Err(Error::ComposerGroupFull {
                            max: MAX_ATOMIC_GROUP_SIZE,
                        });
                    }
                    validate_tx(&txn_with_signer.tx, TransactionArgType::Any)?;
                    txs.push(txn_with_signer);
                }
                AtomicGroupEntry::MethodCall(call) => {
                    process_method_call(call, &mut txs, &mut method_map)?;
                }
            }
        }

        if txs.is_empty() {
            return Err(Error::EmptyTransactionGroup);
        }
        if txs.len() > 1 {
            let mut group_txs: Vec<&mut Transaction> = txs.iter_mut().map(|t| &mut t.tx).collect();
            tx_group::assign_in_place(&mut group_txs)?;
        }

        Ok(UnsignedAtomicGroup { txs, method_map })
    }
}

/// A built, group-id-stamped transaction group, ready to sign or
/// simulate. Reach this state via [`AtomicGroupBuilder::build`].
#[derive(Debug, Clone)]
pub struct UnsignedAtomicGroup {
    txs: Vec<TransactionWithSigner>,
    method_map: HashMap<usize, AbiMethod>,
}

impl UnsignedAtomicGroup {
    /// The group's transactions, in order, each with its signer.
    pub fn transactions(&self) -> &[TransactionWithSigner] {
        &self.txs
    }

    /// Sign every transaction with its attached signer, advancing to the
    /// [`SignedAtomicGroup`] state.
    ///
    /// This is `async`: signers may await user approval (WalletConnect) or
    /// remote I/O (custodial/KMS). Each distinct signer is asked to sign
    /// all of the slots it owns in a single call, so an interactive wallet
    /// prompts once for the whole group.
    ///
    /// Every slot must have a signer. A slot with no signer
    /// ([`TransactionWithSigner::unsigned`]) is an
    /// [`Error::MissingSigner`]: it cannot produce a submittable
    /// signature, and the resulting group is meant for `submit`/`execute`.
    /// Unsigned slots are only valid for
    /// [`simulate`](Self::simulate)/[`simulate_with`](Self::simulate_with).
    pub async fn sign(self) -> Result<SignedAtomicGroup, Error> {
        let signed_txs = sign_group(&self.txs).await?;
        Ok(SignedAtomicGroup {
            signed_txs,
            method_map: self.method_map,
        })
    }

    /// Simulate the group through algod's `/v2/transactions/simulate`
    /// endpoint with no power-pack overrides.
    ///
    /// Takes `&self`: simulation is non-destructive, so the same group
    /// can still be signed and executed afterwards.
    pub async fn simulate(&self, algod: &Algod) -> Result<SimulateOutcome, Error> {
        self.simulate_with(algod, SimulateRequest::new(vec![]))
            .await
    }

    /// Simulate with a caller-supplied [`SimulateRequest`] — use this to
    /// toggle the power-pack fields (extra opcode budget,
    /// allow-more-logging, exec-trace-config, etc.).
    ///
    /// Simulate never invokes the real signers: it signs every slot with
    /// the all-zero placeholder signature and asks algod to allow empty
    /// signatures, so dry-running a group that carries an interactive
    /// signer (e.g. WalletConnect) never pops an approval prompt. To
    /// simulate the actually-signed group, [`sign`](Self::sign) it first
    /// and simulate that.
    ///
    /// The request's `txn_groups` and `allow_empty_signatures` are
    /// overwritten; any other field is forwarded as given.
    pub async fn simulate_with(
        &self,
        algod: &Algod,
        mut request: SimulateRequest,
    ) -> Result<SimulateOutcome, Error> {
        let signed_txs = placeholder_group(&self.txs)?;

        request.txn_groups = vec![SimulateRequestTransactionGroup::new(signed_txs.clone())];
        request.allow_empty_signatures = Some(true);

        let response: SimulateTransaction200Response = algod.simulate_txns(request).await?;

        // Build per-method ABI return values from the pending-txn
        // payloads embedded in the simulate response (mirrors execute()).
        let mut method_results: Vec<AbiMethodResult> = Vec::new();
        if let Some(group) = response.txn_groups.first() {
            for (i, txn_result) in group.txn_results.iter().enumerate() {
                if !self.method_map.contains_key(&i) {
                    continue;
                }
                let tx_id = signed_txs[i].transaction_id().clone();
                let return_type = self.method_map[&i].returns.clone().type_()?;
                method_results.push(get_return_value_with_return_type(
                    &txn_result.txn_result,
                    &tx_id,
                    return_type,
                )?);
            }
        }

        Ok(SimulateOutcome {
            tx_ids: tx_ids(&signed_txs),
            method_results,
            simulate_response: SimulateResponse::new(response),
        })
    }
}

/// A signed transaction group, ready to submit or execute. Reach this
/// state via [`UnsignedAtomicGroup::sign`].
#[derive(Debug, Clone)]
pub struct SignedAtomicGroup {
    signed_txs: Vec<SignedTransaction>,
    method_map: HashMap<usize, AbiMethod>,
}

impl SignedAtomicGroup {
    /// The signed transactions, in group order.
    pub fn signed_transactions(&self) -> &[SignedTransaction] {
        &self.signed_txs
    }

    /// Broadcast the group and return a [`PendingSubmission`] handle.
    /// Call [`PendingSubmission::confirm`] to await finality, or hold the
    /// handle and confirm later.
    pub async fn submit(self, algod: &Algod) -> Result<PendingSubmission, Error> {
        algod.submit_txns(&self.signed_txs).await
    }

    /// Submit the group, wait for it to be confirmed, and decode each ABI
    /// method call's return value.
    pub async fn execute(self, algod: &Algod) -> Result<ExecuteOutcome, Error> {
        algod.send_txns(&self.signed_txs).await?;

        let index_to_wait = (0..self.signed_txs.len())
            .find(|i| self.method_map.contains_key(i))
            .unwrap_or(0);

        let tx_id = self.signed_txs[index_to_wait].transaction_id().clone();
        let pending_tx = poll_until_confirmed(algod, &tx_id).await?;

        let mut method_results: Vec<AbiMethodResult> = vec![];

        for i in 0..self.signed_txs.len() {
            if !self.method_map.contains_key(&i) {
                continue;
            }

            let return_type = self.method_map[&i].returns.clone().type_()?;

            // The slot we already polled needs no extra fetch — decode it
            // straight from the confirmed response we are holding.
            if i == index_to_wait {
                method_results.push(get_return_value_with_return_type(
                    &pending_tx,
                    &tx_id,
                    return_type,
                )?);
                continue;
            }

            // Other method calls in the group: fetch each one's own pending
            // transaction. A fetch failure is surfaced per-result rather
            // than failing the whole group.
            let other_tx_id = self.signed_txs[i].transaction_id().clone();
            match algod.pending_txn(&other_tx_id).await {
                Ok(other_pending_tx) => method_results.push(get_return_value_with_return_type(
                    &other_pending_tx,
                    &other_tx_id,
                    return_type,
                )?),
                Err(e) => method_results.push(AbiMethodResult {
                    tx_id: other_tx_id,
                    tx_info: pending_tx.clone(),
                    return_value: Err(AbiReturnDecodeError(format!("{e:?}"))),
                }),
            }
        }

        Ok(ExecuteOutcome {
            confirmed_round: pending_tx.confirmed_round,
            tx_ids: tx_ids(&self.signed_txs),
            method_results,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use algonaut_core::{Address, MicroAlgos};
    use algonaut_crypto::HashDigest;
    use algonaut_transaction::account::Account;
    use algonaut_transaction::builder::{Pay, TransactionParams};
    use algonaut_transaction::{SigningFuture, SigningRequest};
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct StubParams {
        genesis_id: String,
    }

    impl TransactionParams for StubParams {
        fn last_round(&self) -> u64 {
            1
        }
        fn min_fee(&self) -> u64 {
            1_000
        }
        fn genesis_hash(&self) -> HashDigest {
            HashDigest([0; 32])
        }
        fn genesis_id(&self) -> &String {
            &self.genesis_id
        }
    }

    fn pay(sender: &Account, receiver: Address) -> Transaction {
        let params = StubParams {
            genesis_id: "testnet-v1.0".to_owned(),
        };
        Pay::new(sender.address(), receiver, MicroAlgos(1_000))
            .build(&params)
            .expect("failed to build payment transaction")
    }

    /// Wraps an [`Account`] and counts how many times the composer calls
    /// it, so a test can assert that slots sharing one `Arc` are signed in
    /// a single request.
    #[derive(Debug)]
    struct CountingSigner {
        inner: Account,
        calls: AtomicUsize,
    }

    impl Signer for CountingSigner {
        fn sign_transactions<'a>(&'a self, request: SigningRequest<'a>) -> SigningFuture<'a> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move {
                request
                    .indexes
                    .iter()
                    .map(|&i| self.inner.sign_transaction(request.transactions[i].clone()))
                    .collect()
            })
        }
    }

    /// Returns a fixed, caller-supplied output regardless of the request —
    /// used to drive the composer's signer-output validation.
    #[derive(Debug)]
    struct CannedSigner {
        output: Vec<SignedTransaction>,
    }

    impl Signer for CannedSigner {
        fn sign_transactions<'a>(&'a self, _request: SigningRequest<'a>) -> SigningFuture<'a> {
            let output = self.output.clone();
            Box::pin(async move { Ok(output) })
        }
    }

    /// Two slots that share one `Arc<dyn Signer>` are grouped by identity
    /// and signed in a single call — one approval round-trip per wallet.
    #[tokio::test]
    async fn sign_groups_slots_sharing_one_signer_into_one_call() {
        let alice = Account::generate();
        let bob = Account::generate();
        let signer = Arc::new(CountingSigner {
            inner: alice.clone(),
            calls: AtomicUsize::new(0),
        });

        let signed = AtomicGroupBuilder::new()
            .add_transaction(TransactionWithSigner::new(
                pay(&alice, bob.address()),
                signer.clone(),
            ))
            .add_transaction(TransactionWithSigner::new(
                pay(&alice, alice.address()),
                signer.clone(),
            ))
            .build()
            .unwrap()
            .sign()
            .await
            .unwrap();

        assert_eq!(signed.signed_transactions().len(), 2);
        assert_eq!(
            signer.calls.load(Ordering::SeqCst),
            1,
            "slots sharing one signer instance must be signed in a single call"
        );
    }

    /// A signer that returns a signature for a transaction other than the
    /// one requested is rejected, not submitted.
    #[tokio::test]
    async fn sign_rejects_signer_returning_wrong_transaction() {
        let alice = Account::generate();
        let bob = Account::generate();
        // A signed transaction over an unrelated transaction.
        let decoy = bob.sign_transaction(pay(&bob, alice.address())).unwrap();
        let signer = Arc::new(CannedSigner {
            output: vec![decoy],
        });

        let err = AtomicGroupBuilder::new()
            .add_transaction(TransactionWithSigner::new(
                pay(&alice, bob.address()),
                signer,
            ))
            .build()
            .unwrap()
            .sign()
            .await
            .unwrap_err();

        assert!(matches!(err, Error::SignerOutputInvalid { .. }));
    }

    /// A signer that returns the wrong number of signed transactions is
    /// rejected.
    #[tokio::test]
    async fn sign_rejects_signer_returning_wrong_count() {
        let alice = Account::generate();
        let bob = Account::generate();
        // Asked to sign one slot, returns none.
        let signer = Arc::new(CannedSigner { output: vec![] });

        let err = AtomicGroupBuilder::new()
            .add_transaction(TransactionWithSigner::new(
                pay(&alice, bob.address()),
                signer,
            ))
            .build()
            .unwrap()
            .sign()
            .await
            .unwrap_err();

        assert!(matches!(err, Error::SignerOutputInvalid { .. }));
    }

    /// Signing a group that contains an unsigned (simulate-only) slot is
    /// rejected: an unsigned slot cannot produce a submittable signature.
    #[tokio::test]
    async fn sign_rejects_unsigned_slot() {
        let alice = Account::generate();
        let bob = Account::generate();

        let err = AtomicGroupBuilder::new()
            .add_transaction(TransactionWithSigner::new(
                pay(&alice, bob.address()),
                Arc::new(alice.clone()),
            ))
            .add_transaction(TransactionWithSigner::unsigned(pay(&bob, alice.address())))
            .build()
            .unwrap()
            .sign()
            .await
            .unwrap_err();

        assert!(matches!(err, Error::MissingSigner { index: 1 }));
    }

    /// Signing a built group whose transactions are authorized by
    /// *different* signers must yield exactly one signed transaction per
    /// input, in input order.
    #[tokio::test]
    async fn sign_signs_every_tx_with_distinct_signers() {
        let alice = Account::generate();
        let bob = Account::generate();

        let signed = AtomicGroupBuilder::new()
            .add_transaction(TransactionWithSigner::new(
                pay(&alice, bob.address()),
                Arc::new(alice.clone()),
            ))
            .add_transaction(TransactionWithSigner::new(
                pay(&bob, alice.address()),
                Arc::new(bob.clone()),
            ))
            .build()
            .unwrap()
            .sign()
            .await
            .unwrap();

        let txs = signed.signed_transactions();
        assert_eq!(
            txs.len(),
            2,
            "every input transaction must be signed exactly once"
        );
        assert_eq!(txs[0].transaction().sender(), alice.address());
        assert_eq!(txs[1].transaction().sender(), bob.address());
    }

    /// `build` rejects a group with no entries.
    #[test]
    fn build_rejects_empty_group() {
        let err = AtomicGroupBuilder::new().build().unwrap_err();
        assert!(matches!(err, Error::EmptyTransactionGroup));
    }
}
