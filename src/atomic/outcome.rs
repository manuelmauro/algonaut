//! Result types returned by an atomic group, and the decoding of ABI
//! method-call return values out of a confirmed transaction's logs.
//!
//! [`ExecuteOutcome`] and [`SimulateOutcome`] are the group's two terminal
//! results; both carry a [`AbiMethodResult`] per method call.
//! [`get_return_value_with_return_type`] is the shared decoder both the
//! execute and simulate paths in [`group`](super::group) use to turn a
//! pending-transaction payload into a typed return value.

use algonaut_abi::{
    abi_interactions::AbiReturnType,
    abi_type::{AbiType, AbiValue},
};
use algonaut_core::{AppId, TransactionId};
use algonaut_model::algod::PendingTransactionResponse;

use crate::{Error, simulate::SimulateResponse};

/// 4-byte prefix for logged return values, from https://github.com/algorandfoundation/ARCs/blob/main/ARCs/arc-0004.md#standard-format
const ABI_RETURN_HASH: [u8; 4] = [0x15, 0x1f, 0x7c, 0x75];

/// Represents the output from a successful ABI method call.
#[derive(Debug, Clone)]
pub struct AbiMethodResult {
    /// The TxID of the transaction that invoked the ABI method call.
    pub transaction_id: TransactionId,
    /// Information about the confirmed transaction that invoked the ABI method call.
    pub transaction_info: PendingTransactionResponse,
    /// The method's return value
    pub return_value: Result<AbiMethodReturnValue, AbiReturnDecodeError>,
}

#[derive(Debug, Clone)]
pub struct AbiReturnDecodeError(pub String);

#[derive(Debug, Clone)]
pub enum AbiMethodReturnValue {
    Some(AbiValue),
    Void,
}

/// Results of successfully [`executing`](super::SignedAtomicGroup::execute) a
/// transaction group: the confirmed round, the group's transaction ids,
/// and the decoded ABI return value for each method call.
#[derive(Debug, Clone)]
pub struct ExecuteOutcome {
    /// The round in which the executed transaction group was confirmed on chain
    /// (optional, because the transaction's confirmed round is optional).
    pub confirmed_round: Option<u64>,
    /// A list of the TxIDs for each transaction in the executed group
    pub transaction_ids: Vec<TransactionId>,
    /// Return values for all the ABI method calls in the executed group
    pub method_results: Vec<AbiMethodResult>,
    /// The id of the application created by the awaited transaction, if it
    /// created one (e.g. a bare app-create). `None` otherwise. The awaited
    /// transaction is the first ABI method call, or the first transaction when
    /// the group has none — so this captures the common "deploy" case of a
    /// lone create transaction.
    ///
    /// For a create at another index in a mixed group, see
    /// [`created_app_ids`](Self::created_app_ids).
    pub created_app_id: Option<AppId>,
    /// Every application created by the group, paired with the index of the
    /// transaction that created it, in group order. Captures app-creates at any
    /// slot of a mixed group — not just the awaited transaction that
    /// [`created_app_id`](Self::created_app_id) reflects. Empty when the group
    /// created no application.
    pub created_app_ids: Vec<(usize, AppId)>,
}

impl ExecuteOutcome {
    /// The application created by the transaction at group index `index`, if
    /// that transaction created one.
    pub fn created_app_id_at(&self, index: usize) -> Option<AppId> {
        self.created_app_ids
            .iter()
            .find(|(i, _)| *i == index)
            .map(|(_, app_id)| *app_id)
    }
}

/// Result of [`simulating`](super::UnsignedAtomicGroup::simulate) a group. Mirrors
/// [`ExecuteOutcome`] with the raw simulate response attached. Because
/// simulate borrows the group (`&self`), the same group can still be
/// signed and executed afterwards.
#[derive(Debug, Clone)]
pub struct SimulateOutcome {
    /// TxIDs for each transaction in the simulated group.
    pub transaction_ids: Vec<TransactionId>,
    /// ABI return values per method call. Errors are surfaced
    /// per-result (the same way [`ExecuteOutcome`] does it) so callers
    /// can inspect partial successes.
    pub method_results: Vec<AbiMethodResult>,
    /// Hand-named view over algod's simulate response (success flag,
    /// per-group failure messages, budget overrides, …), exposing typed
    /// accessors rather than the generated response type.
    pub simulate_response: SimulateResponse,
}

pub(super) fn get_return_value_with_return_type(
    pending_tx: &PendingTransactionResponse,
    transaction_id: &TransactionId, // our txn in PendingTransaction currently has no fields, so the tx id is passed separately
    return_type: AbiReturnType,
) -> Result<AbiMethodResult, Error> {
    let return_value = match return_type {
        AbiReturnType::Some(return_type) => {
            get_return_value_with_abi_type(pending_tx, &return_type)?
        }
        AbiReturnType::Void => Ok(AbiMethodReturnValue::Void),
    };

    Ok(AbiMethodResult {
        transaction_id: transaction_id.to_owned(),
        transaction_info: pending_tx.clone(),
        return_value,
    })
}

fn get_return_value_with_abi_type(
    pending_tx: &PendingTransactionResponse,
    abi_type: &AbiType,
) -> Result<Result<AbiMethodReturnValue, AbiReturnDecodeError>, Error> {
    let logs = pending_tx.logs.as_deref().ok_or(Error::MissingReturnLog)?;
    let ret_line = logs.last().ok_or(Error::MissingReturnLog)?;

    // `Bytes` already holds the decoded log bytes — its `Deserialize` base64-decodes
    // the JSON string form and takes the msgpack `bin` form raw — so the last log is
    // the raw ARC-4 return line, not a base64 string to decode again.
    let decoded_ret_line = &ret_line.0;

    if !decoded_ret_line.starts_with(&ABI_RETURN_HASH) {
        return Err(Error::MissingReturnLog);
    }

    let abi_encoded = &decoded_ret_line[ABI_RETURN_HASH.len()..];
    Ok(match abi_type.decode(abi_encoded) {
        Ok(decoded) => Ok(AbiMethodReturnValue::Some(decoded)),
        Err(e) => Err(AbiReturnDecodeError(format!("{e:?}"))),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn created_app_id_at_finds_the_creating_index() {
        // A mixed group where a create sits at index 2 (not the awaited slot):
        // `created_app_id` reflects the awaited transaction, while
        // `created_app_ids`/`created_app_id_at` capture the create wherever it is.
        let outcome = ExecuteOutcome {
            confirmed_round: Some(42),
            transaction_ids: vec![],
            method_results: vec![],
            created_app_id: None,
            created_app_ids: vec![(2, AppId(777)), (4, AppId(888))],
        };

        assert_eq!(outcome.created_app_id_at(2), Some(AppId(777)));
        assert_eq!(outcome.created_app_id_at(4), Some(AppId(888)));
        // A non-creating index yields `None`.
        assert_eq!(outcome.created_app_id_at(0), None);
        assert_eq!(outcome.created_app_id_at(3), None);
    }
}
