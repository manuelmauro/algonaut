use algonaut_core::ToMsgPack;
use algonaut_crypto::HashDigest;
use serde::{Deserialize, Serialize, Serializer};
use sha2::Digest;

use crate::{error::TransactionError, Transaction};

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub struct TxGroup {
    #[serde(rename = "txlist", default)]
    tx_group_hashes: Vec<HashDigest>,
}

impl TxGroup {
    const MAX_TX_GROUP_SIZE: usize = 16;

    pub fn new(tx_group_hashes: Vec<HashDigest>) -> TxGroup {
        TxGroup { tx_group_hashes }
    }

    pub fn assign_group_id(txns: Vec<&mut Transaction>) -> Result<(), TransactionError> {
        let gid = TxGroup::compute_group_id(&txns)?;
        for tx in txns {
            tx.assign_group_id(gid);
        }
        Ok(())
    }

    fn compute_group_id(txns: &[&mut Transaction]) -> Result<HashDigest, TransactionError> {
        if txns.is_empty() {
            return Err(TransactionError::EmptyTransactionListError);
        }
        if txns.len() > Self::MAX_TX_GROUP_SIZE {
            return Err(TransactionError::MaxTransactionGroupSizeError {
                size: Self::MAX_TX_GROUP_SIZE,
            });
        }
        let mut ids: Vec<HashDigest> = vec![];
        for t in txns {
            ids.push(t.raw_id()?);
        }
        let group = TxGroup::new(ids);
        let hashed = sha2::Sha512_256::digest(&group.bytes_to_sign()?);
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
