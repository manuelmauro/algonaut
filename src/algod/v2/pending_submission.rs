use crate::{Error, algod::v2::Algod};
use algonaut_algod::models::PendingTransactionResponse;
use algonaut_core::TxId;
use instant::Instant;
use std::time::Duration;

/// Default timeout for [`PendingSubmission::confirm`].
const DEFAULT_CONFIRM_TIMEOUT: Duration = Duration::from_secs(60);

/// Handle returned by [`Algod::submit`], [`Algod::submit_txns`] and
/// [`Algod::submit_raw`]. Carries the transaction id of the broadcast
/// transaction and knows how to poll algod for finality.
#[derive(Debug, Clone)]
pub struct PendingSubmission {
    algod: Algod,
    tx_id: TxId,
}

impl PendingSubmission {
    pub(crate) fn new(algod: Algod, tx_id: TxId) -> Self {
        Self { algod, tx_id }
    }

    /// Transaction id of the broadcast transaction.
    pub fn tx_id(&self) -> &TxId {
        &self.tx_id
    }

    /// Poll algod until the transaction is confirmed. Uses a 60s default
    /// timeout — call [`PendingSubmission::confirm_with`] to override.
    pub async fn confirm(self) -> Result<PendingTransactionResponse, Error> {
        self.confirm_with(DEFAULT_CONFIRM_TIMEOUT).await
    }

    /// Poll algod until the transaction is confirmed, returning
    /// `Error::Msg("Pending transaction timed out (..)")` if the
    /// supplied `timeout` elapses first.
    ///
    /// Between checks, waits on algod's `wait-for-block-after/{round}`
    /// long-poll rather than a fixed timer — the future stays parked
    /// until the next block lands, so the cadence is network-driven
    /// (~3–4s) instead of an arbitrary 250ms tick.
    ///
    /// If algod reports the transaction was kicked out of its pool
    /// (`pool-error` non-empty — e.g. expired, underfunded, group
    /// invalid), this returns `Error::Msg("Transaction pool error:
    /// ..")` immediately rather than waiting out the timeout.
    pub async fn confirm_with(
        self,
        timeout: Duration,
    ) -> Result<PendingTransactionResponse, Error> {
        let start = Instant::now();
        let mut last_round = self.algod.status().await?.last_round;
        loop {
            let pending = self.algod.pending_txn(&self.tx_id).await?;
            if pending.confirmed_round.is_some() {
                return Ok(pending);
            }
            if !pending.pool_error.is_empty() {
                return Err(Error::Msg(format!(
                    "Transaction pool error: {}",
                    pending.pool_error
                )));
            }
            if start.elapsed() >= timeout {
                return Err(Error::Msg(format!(
                    "Pending transaction timed out ({timeout:?})"
                )));
            }
            last_round = self.algod.status_after_block(last_round).await?.last_round;
        }
    }
}
