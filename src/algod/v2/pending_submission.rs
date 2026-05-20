use crate::{Error, algod::v2::Algod, util::sleep};
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
    pub async fn confirm_with(
        self,
        timeout: Duration,
    ) -> Result<PendingTransactionResponse, Error> {
        let start = Instant::now();
        loop {
            let pending = self.algod.pending_txn(&self.tx_id).await?;
            if pending.confirmed_round.is_some() {
                return Ok(pending);
            } else if start.elapsed() >= timeout {
                return Err(Error::Msg(format!(
                    "Pending transaction timed out ({timeout:?})"
                )));
            }
            sleep(250).await;
        }
    }
}
