use super::sleep;
use crate::{algod::v2::Algod, Error};
use algonaut_algod::models::PendingTransactionResponse;
use instant::Instant;
use std::time::Duration;

/// Utility to wait for a transaction to be confirmed
pub async fn wait_for_pending_transaction(
    algod: &Algod,
    tx_id: &str,
) -> Result<PendingTransactionResponse, Error> {
    let timeout = Duration::from_secs(60);
    let start = Instant::now();
    loop {
        let pending_transaction = algod.pending_txn(tx_id).await?;
        // If the transaction has been confirmed or we time out, exit.
        if pending_transaction.confirmed_round.is_some() {
            return Ok(pending_transaction);
        } else if start.elapsed() >= timeout {
            return Err(Error::Msg(format!(
                "Pending transaction timed out ({timeout:?})"
            )));
        }
        sleep(250).await;
    }
}
