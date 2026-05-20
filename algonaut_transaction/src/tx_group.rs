use algonaut_core::ToMsgPack;
use algonaut_crypto::HashDigest;
use serde::{Deserialize, Serialize, Serializer};
use sha2::Digest;

use crate::{Transaction, error::TransactionError};

/// A batch of transactions sharing a group ID.
///
/// The only construction path is [`TryFrom<Vec<Transaction>>`], which
/// validates the group size, computes the group ID, and stamps it onto
/// every transaction before returning the populated batch. Once
/// constructed, the inner transactions are accessible via
/// [`TxGroup::transactions`], [`TxGroup::into_transactions`], or
/// the [`IntoIterator`] impl.
///
/// The previous public surface — `TxGroup::new(Vec<HashDigest>)`,
/// `TxGroup::assign_group_id(&mut [&mut Transaction])`, and the brief
/// rename `TxGroup::assign` — leaked an internal hashing form and forced
/// callers into either slice-of-mut-refs ergonomics or a static method
/// that pretended to be a constructor. Both are gone; the msgpack
/// hashing helper now lives as a private [`TxGroupDigests`] type.
#[derive(Debug, Clone)]
pub struct TxGroup {
    txns: Vec<Transaction>,
}

impl TxGroup {
    const MAX_TX_GROUP_SIZE: usize = 16;

    /// Borrow the grouped transactions.
    pub fn transactions(&self) -> &[Transaction] {
        &self.txns
    }

    /// Take the grouped transactions, consuming the batch.
    pub fn into_transactions(self) -> Vec<Transaction> {
        self.txns
    }
}

impl TryFrom<Vec<Transaction>> for TxGroup {
    type Error = TransactionError;

    fn try_from(mut txns: Vec<Transaction>) -> Result<Self, Self::Error> {
        let mut refs: Vec<&mut Transaction> = txns.iter_mut().collect();
        let gid = compute_group_id(refs.as_mut_slice())?;
        for tx in txns.iter_mut() {
            tx.assign_group_id(gid);
        }
        Ok(Self { txns })
    }
}

impl IntoIterator for TxGroup {
    type Item = Transaction;
    type IntoIter = std::vec::IntoIter<Transaction>;

    fn into_iter(self) -> Self::IntoIter {
        self.txns.into_iter()
    }
}

/// Compute the group ID for a slice of transactions, leaving them
/// untouched. Used by the in-place [`assign_in_place`] helper and by
/// [`TxGroup::try_from`].
pub(crate) fn compute_group_id(
    txns: &mut [&mut Transaction],
) -> Result<HashDigest, TransactionError> {
    if txns.is_empty() {
        return Err(TransactionError::EmptyTransactionListError);
    }
    if txns.len() > TxGroup::MAX_TX_GROUP_SIZE {
        return Err(TransactionError::MaxTransactionGroupSizeError {
            size: TxGroup::MAX_TX_GROUP_SIZE,
        });
    }
    let mut ids: Vec<HashDigest> = vec![];
    for t in txns.iter() {
        ids.push(t.raw_id()?);
    }
    let digests = TxGroupDigests {
        tx_group_hashes: ids,
    };
    let hashed = sha2::Sha512_256::digest(digests.bytes_to_sign()?);
    Ok(HashDigest(hashed.into()))
}

/// In-place group-id assignment for callers that already hold their
/// transactions through mutable references — the atomic transaction
/// composer, which stores transactions inside `TransactionWithSigner`
/// records and cannot move them out for [`TxGroup::try_from`].
///
/// `#[doc(hidden)]` because [`TxGroup::try_from`] is the user-facing
/// API; this stays exposed (workspace visibility) for the composer's
/// internal use only.
#[doc(hidden)]
pub fn assign_in_place(txns: &mut [&mut Transaction]) -> Result<(), TransactionError> {
    let gid = compute_group_id(txns)?;
    for tx in txns {
        tx.assign_group_id(gid);
    }
    Ok(())
}

/// The msgpack-hashing form of a transaction group: a list of transaction
/// raw-IDs serialised as `txlist` and prefixed with `"TG"` before SHA-512/256
/// hashing. Held internally so the public [`TxGroup`] type isn't
/// contaminated by a representation no caller cares about.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
struct TxGroupDigests {
    #[serde(rename = "txlist", default)]
    tx_group_hashes: Vec<HashDigest>,
}

impl ToMsgPack for TxGroupDigests {}

impl TxGroupDigests {
    fn bytes_to_sign(&self) -> Result<Vec<u8>, TransactionError> {
        let encoded_tx = self.to_msg_pack()?;
        let mut prefix_encoded_tx = b"TG".to_vec();
        prefix_encoded_tx.extend_from_slice(&encoded_tx);
        Ok(prefix_encoded_tx)
    }
}

impl Serialize for TxGroupDigests {
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
