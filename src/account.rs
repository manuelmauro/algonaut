use data_encoding::BASE32_NOPAD;
use rand::rngs::OsRng;
use rand::Rng;
use ring::signature::Ed25519KeyPair as KeyPairType;
use ring::signature::KeyPair;

use crate::auction::{Bid, SignedBid};
use crate::crypto::{Address, MultisigAddress, MultisigSignature, MultisigSubsig, Signature};
use crate::transaction::{SignedTransaction, Transaction};
use crate::{Ed25519PublicKey, Error};
use sha2::Digest;
use std::borrow::Borrow;

type ChecksumAlg = sha2::Sha512Trunc256;

pub struct Account {
    seed: [u8; 32],
    address: Address,
    key_pair: KeyPairType,
}

impl Account {
    pub fn generate() -> Account {
        let seed: [u8; 32] = OsRng.gen();
        Self::from_seed(seed)
    }

    /// Create account from human readable mnemonic of a 32 byte seed
    pub fn from_mnemonic(mnemonic: &str) -> Result<Account, Error> {
        let seed = crate::mnemonic::to_key(mnemonic)?;
        Ok(Self::from_seed(seed))
    }

    /// Create account from 32 byte seed
    pub fn from_seed(seed: [u8; 32]) -> Account {
        let key_pair = KeyPairType::from_seed_unchecked(&seed).unwrap();
        let mut pk = [0; 32];
        pk.copy_from_slice(key_pair.public_key().as_ref());
        let address = Address::new(pk);
        Account {
            seed,
            address,
            key_pair,
        }
    }

    /// Get the public key address of the account
    pub fn address(&self) -> Address {
        self.address
    }

    /// Get the human readable mnemonic of the 32 byte seed
    pub fn mnemonic(&self) -> String {
        crate::mnemonic::from_key(&self.seed).unwrap()
    }

    /// Get the 32 byte seed
    pub fn seed(&self) -> [u8; 32] {
        self.seed
    }

    fn sign(&self, bytes: &[u8]) -> Signature {
        let signature = self.key_pair.sign(&bytes);
        // ring returns a signature with padding at the end to make it 105 bytes, only 64 bytes are actually used
        let mut stripped_signature = [0; 64];
        stripped_signature.copy_from_slice(&signature.as_ref()[..64]);
        Signature(stripped_signature)
    }

    /// Sign a bid with the account's private key
    pub fn sign_bid(&self, bid: Bid) -> Result<SignedBid, Error> {
        let encoded_bid = rmp_serde::to_vec_named(&bid)?;
        let mut prefix_encoded_bid = b"aB".to_vec();
        prefix_encoded_bid.extend_from_slice(&encoded_bid);
        let signature = self.sign(&prefix_encoded_bid);
        Ok(SignedBid {
            bid,
            sig: signature,
        })
    }

    /// Sign a transaction with the account's private key
    pub fn sign_transaction(&self, transaction: &Transaction) -> Result<SignedTransaction, Error> {
        let encoded_tx = rmp_serde::to_vec_named(transaction)?;
        let mut prefix_encoded_tx = b"TX".to_vec();
        prefix_encoded_tx.extend_from_slice(&encoded_tx);
        let signature = self.sign(&prefix_encoded_tx);
        let id = BASE32_NOPAD.encode(&ChecksumAlg::digest(&signature.0));
        Ok(SignedTransaction {
            transaction: transaction.clone(),
            sig: Some(signature),
            multisig: None,
            transaction_id: id,
        })
    }

    /// Sign the transaction and populate the multisig field of the signed transaction with the given multisig address
    pub fn sign_multisig_transaction(
        &self,
        from: MultisigAddress,
        transaction: &Transaction,
    ) -> Result<SignedTransaction, Error> {
        if from.address() != transaction.sender {
            return Err(Error::Api(
                "Transaction sender does not match multisig identity".to_string(),
            ));
        }
        let my_public_key = Ed25519PublicKey(self.address.0);
        if !from.public_keys.contains(&my_public_key) {
            return Err(Error::Api(
                "Multisig identity does not contain this secret key".to_string(),
            ));
        }
        let signed_transaction = self.sign_transaction(transaction)?;
        let subsigs: Vec<MultisigSubsig> = from
            .public_keys
            .iter()
            .map(|key| {
                if *key == my_public_key {
                    MultisigSubsig {
                        key: *key,
                        sig: signed_transaction.clone().sig,
                    }
                } else {
                    MultisigSubsig {
                        key: *key,
                        sig: None,
                    }
                }
            })
            .collect();
        let multisig = MultisigSignature {
            version: from.version,
            threshold: from.threshold,
            subsigs,
        };
        Ok(SignedTransaction {
            multisig: Some(multisig),
            sig: None,
            transaction: transaction.clone(),
            transaction_id: signed_transaction.transaction_id,
        })
    }

    /// Appends the multisig signature from the given multisig address to the transaction
    pub fn append_multisig_transaction(
        &self,
        from: MultisigAddress,
        transaction: &SignedTransaction,
    ) -> Result<SignedTransaction, Error> {
        let from_transaction = self.sign_multisig_transaction(from, &transaction.transaction)?;
        Self::merge_multisig_transactions(&[&from_transaction, transaction])
    }

    /// Returns a signed transaction with the multisig signatures of the passed signed transactions merged
    pub fn merge_multisig_transactions<T: Borrow<SignedTransaction>>(
        transactions: &[T],
    ) -> Result<SignedTransaction, Error> {
        if transactions.len() < 2 {
            return Err(Error::Api("Can't merge only one transaction".to_string()));
        }
        let mut merged = transactions[0].borrow().clone();
        for transaction in transactions {
            let merged_msig = merged.multisig.as_mut().unwrap();
            let msig = transaction.borrow().multisig.as_ref().unwrap();
            if merged_msig.subsigs.len() != msig.subsigs.len() {
                return Err(Error::Api(
                    "Multisig signatures to merge must have the same number of subsignatures".to_string()
                ));
            }
            assert_eq!(merged_msig.subsigs.len(), msig.subsigs.len());
            for (merged_subsig, subsig) in merged_msig.subsigs.iter_mut().zip(&msig.subsigs) {
                if subsig.key != merged_subsig.key {
                    return Err(Error::Api(
                        "Transaction msig public keys do not match".to_string(),
                    ));
                }
                if merged_subsig.sig.is_none() {
                    merged_subsig.sig = subsig.sig
                } else if merged_subsig.sig != subsig.sig && subsig.sig.is_some() {
                    return Err(Error::Api(
                        "transaction msig has mismatched signatures".to_string(),
                    ));
                }
            }
        }
        Ok(merged)
    }
}
