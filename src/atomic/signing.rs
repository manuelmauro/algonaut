//! Signer orchestration and finality polling for an atomic group.
//!
//! The typestate chain in [`group`](super::group) delegates the actual
//! signing here: [`sign_group`] drives the real signers (one approval
//! round-trip per wallet), [`placeholder_group`] produces the all-zero
//! signatures the simulate path needs, and [`poll_until_confirmed`] waits
//! for a submitted transaction to reach finality.

use std::sync::Arc;
use std::time::Duration;

use algonaut_algod::models::PendingTransactionResponse;
use algonaut_core::TxId;
use algonaut_transaction::{
    SignedTransaction, Signer, SigningRequest, Transaction, signed_transaction,
};

use instant::Instant;

use crate::{Error, algod::v2::Algod};

use super::TransactionWithSigner;

/// Default timeout matching [`crate::algod::v2::PendingSubmission::confirm`].
const COMPOSER_CONFIRM_TIMEOUT: Duration = Duration::from_secs(60);

/// Poll algod for finality of the given transaction id. A signed group
/// already has the tx ids it wants to wait on (post-`send_txns`), so this
/// internal helper is the equivalent of `PendingSubmission::confirm`
/// against an arbitrary id.
pub(super) async fn poll_until_confirmed(
    algod: &Algod,
    tx_id: &TxId,
) -> Result<PendingTransactionResponse, Error> {
    let start = Instant::now();
    let mut last_round = algod.status().await?.last_round;
    loop {
        let pending = algod.pending_txn(tx_id).await?;
        if pending.confirmed_round.is_some() {
            return Ok(pending);
        }
        if !pending.pool_error.is_empty() {
            return Err(Error::PendingTransactionPoolError {
                reason: pending.pool_error,
            });
        }
        if start.elapsed() >= COMPOSER_CONFIRM_TIMEOUT {
            return Err(Error::PendingTransactionTimeout {
                timeout: COMPOSER_CONFIRM_TIMEOUT,
            });
        }
        last_round = algod.status_after_block(last_round).await?.last_round;
    }
}

/// Sign the group with its attached signers, awaiting each one.
///
/// Slots are grouped by signer **identity** (`Arc::ptr_eq`) so every slot
/// a given signer instance owns is signed in a single
/// [`Signer::sign_transactions`] call — one approval round-trip per
/// wallet. Each signer sees the whole group via
/// [`SigningRequest::transactions`] but signs only its
/// [`indexes`](SigningRequest::indexes). Output is validated against the
/// request and reassembled into original group order.
///
/// Every slot must have a signer: a `None`-signer slot
/// ([`TransactionWithSigner::unsigned`]) cannot produce a submittable
/// signature, so it is an [`Error::MissingSigner`] here. Unsigned slots
/// are only valid for [`simulate`](super::UnsignedAtomicGroup::simulate),
/// which uses [`placeholder_group`] instead.
pub(super) async fn sign_group(
    txs: &[TransactionWithSigner],
) -> Result<Vec<SignedTransaction>, Error> {
    let all_txs: Vec<Transaction> = txs.iter().map(|t| t.tx.clone()).collect();
    let mut signed: Vec<Option<SignedTransaction>> = (0..txs.len()).map(|_| None).collect();

    // Group slot indexes by signer identity, preserving first-appearance
    // order so any approval prompts happen in a deterministic order.
    let mut groups: Vec<(Arc<dyn Signer>, Vec<usize>)> = Vec::new();
    for (i, tx_with_signer) in txs.iter().enumerate() {
        match &tx_with_signer.signer {
            Some(signer) => match groups.iter_mut().find(|(s, _)| Arc::ptr_eq(s, signer)) {
                Some((_, indexes)) => indexes.push(i),
                None => groups.push((Arc::clone(signer), vec![i])),
            },
            // An unsigned slot has no signature to produce. Signing is the
            // path to a submittable group, so reject it here rather than
            // emitting an all-zero placeholder that algod would refuse.
            None => return Err(Error::MissingSigner { index: i }),
        }
    }

    for (signer, indexes) in &groups {
        let request = SigningRequest {
            transactions: &all_txs,
            indexes,
        };
        let result = signer.sign_transactions(request).await?;

        if result.len() != indexes.len() {
            return Err(Error::SignerOutputInvalid {
                reason: format!(
                    "signer returned {} signed transaction(s) for {} requested",
                    result.len(),
                    indexes.len()
                ),
            });
        }

        // Each returned transaction must wrap the transaction we asked the
        // signer to sign, in `indexes` order — this catches a signer that
        // returns a valid signature for the wrong transaction (or returns
        // them out of order).
        for (&index, signed_tx) in indexes.iter().zip(result) {
            let expected_id = all_txs[index].id()?;
            if signed_tx.transaction_id() != &expected_id {
                return Err(Error::SignerOutputInvalid {
                    reason: format!(
                        "signer returned a signature for an unexpected transaction at index {index}"
                    ),
                });
            }
            signed[index] = Some(signed_tx);
        }
    }

    signed
        .into_iter()
        .enumerate()
        .map(|(i, slot)| {
            slot.ok_or_else(|| {
                Error::Internal(format!(
                    "no signature produced for transaction at index {i}"
                ))
            })
        })
        .collect()
}

/// Sign every slot with the all-zero placeholder signature, ignoring the
/// attached signers entirely. Used by the simulate path so a dry-run
/// never invokes a real (possibly interactive) signer.
pub(super) fn placeholder_group(
    txs: &[TransactionWithSigner],
) -> Result<Vec<SignedTransaction>, Error> {
    txs.iter()
        .map(|t| signed_transaction::placeholder(t.tx.clone()).map_err(Error::from))
        .collect()
}

/// Collect the base32 transaction ids of a signed group, in order.
pub(super) fn tx_ids(signed_txs: &[SignedTransaction]) -> Vec<String> {
    signed_txs
        .iter()
        .map(|t| t.transaction_id().0.clone())
        .collect()
}
