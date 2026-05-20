use algonaut_core::ToMsgPack;
use algonaut_crypto::HashDigest;
use serde::{Deserialize, Serialize, Serializer};
use sha2::Digest;

use crate::{Transaction, error::TransactionError};

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub struct TxGroup {
    #[serde(rename = "txlist", default)]
    tx_group_hashes: Vec<HashDigest>,
}

impl TxGroup {
    const MAX_TX_GROUP_SIZE: usize = 16;

    /// Assign a group ID to the given transactions and return the grouped
    /// copies. Replaces the older `assign_group_id(&mut [&mut Transaction])`
    /// API, whose slice-of-mutable-references shape made it the odd one out
    /// in the public surface.
    ///
    /// The constructor returns `Vec<Transaction>` rather than `Self` because
    /// `TxGroup` (the msgpack-hashing helper) is not what callers want — they
    /// want the grouped transactions back. Per the
    /// `identifier-newtypes-at-client-boundary` ADR.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(mut txns: Vec<Transaction>) -> Result<Vec<Transaction>, TransactionError> {
        let mut refs: Vec<&mut Transaction> = txns.iter_mut().collect();
        let gid = TxGroup::compute_group_id(refs.as_mut_slice())?;
        for tx in txns.iter_mut() {
            tx.assign_group_id(gid);
        }
        Ok(txns)
    }

    /// In-place group-id assignment for callers that already hold their
    /// transactions through mutable references — currently only the atomic
    /// transaction composer, which stores transactions inside
    /// `TransactionWithSigner` records and would otherwise have to
    /// drain-and-refill on every `build_group`.
    ///
    /// `#[doc(hidden)]` because [`TxGroup::new`] is the user-facing API;
    /// this stays exposed (across-crate workspace visibility) for the
    /// composer's internal use only.
    #[doc(hidden)]
    pub fn assign_group_id(txns: &mut [&mut Transaction]) -> Result<(), TransactionError> {
        let gid = TxGroup::compute_group_id(txns)?;
        for tx in txns {
            tx.assign_group_id(gid);
        }
        Ok(())
    }

    /// Internal constructor that wraps a `Vec<HashDigest>` for msgpack
    /// hashing. Use [`TxGroup::new`] to actually group transactions.
    pub(crate) fn from_hashes(tx_group_hashes: Vec<HashDigest>) -> TxGroup {
        TxGroup { tx_group_hashes }
    }

    pub(crate) fn compute_group_id(
        txns: &mut [&mut Transaction],
    ) -> Result<HashDigest, TransactionError> {
        if txns.is_empty() {
            return Err(TransactionError::EmptyTransactionListError);
        }
        if txns.len() > Self::MAX_TX_GROUP_SIZE {
            return Err(TransactionError::MaxTransactionGroupSizeError {
                size: Self::MAX_TX_GROUP_SIZE,
            });
        }
        let mut ids: Vec<HashDigest> = vec![];
        for t in txns.iter() {
            ids.push(t.raw_id()?);
        }
        let group = TxGroup::from_hashes(ids);
        let hashed = sha2::Sha512_256::digest(group.bytes_to_sign()?);
        Ok(HashDigest(hashed.into()))
    }

    fn bytes_to_sign(&self) -> Result<Vec<u8>, TransactionError> {
        let encoded_tx = self.to_msg_pack()?;
        let mut prefix_encoded_tx = b"TG".to_vec();
        prefix_encoded_tx.extend_from_slice(&encoded_tx);
        Ok(prefix_encoded_tx)
    }
}

impl Serialize for TxGroup {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("TxGroup", 1)?;
        state.serialize_field("txlist", &self.tx_group_hashes)?;
        state.end()
    }
}
