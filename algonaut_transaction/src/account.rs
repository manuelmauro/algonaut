use std::convert::TryInto;

use crate::auction::{Bid, SignedBid};
use crate::error::TransactionError;
use crate::transaction::{SignedTransaction, Transaction, TransactionSignature};
use algonaut_core::{
    Address, CompiledTeal, MultisigAddress, MultisigSignature, MultisigSubsig, ToMsgPack,
};
use algonaut_crypto::{mnemonic, Signature};
use rand::rngs::OsRng;
use rand::Rng;
use ring::signature::{Ed25519KeyPair, KeyPair};

#[derive(Debug)]
pub struct Account {
    seed: [u8; 32],
    address: Address,
    key_pair: Ed25519KeyPair,
}

impl Clone for Account {
    fn clone(&self) -> Self {
        Self::from_seed(self.seed)
    }
}

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        // TODO verify that this is correct - do we always get the same key pair for a seed? or is it otherwise still correct?
        self.seed == other.seed && self.address == other.address
    }
}
impl Eq for Account {}

impl Account {
    pub fn generate() -> Account {
        let seed: [u8; 32] = OsRng.gen();
        Self::from_seed(seed)
    }

    /// Create account from human readable mnemonic of a 32 byte seed
    pub fn from_mnemonic(mnemonic: &str) -> Result<Account, TransactionError> {
        let seed = mnemonic::to_key(mnemonic)?;
        Ok(Self::from_seed(seed))
    }

    /// Create account from 32-byte seed
    pub fn from_seed(seed: [u8; 32]) -> Account {
        let key_pair = Ed25519KeyPair::from_seed_unchecked(&seed).unwrap();
        let public_key = key_pair.public_key().as_ref();
        let public_key_byte_array = key_pair
            .public_key()
            .as_ref()
            .try_into()
            .unwrap_or_else(|_| panic!("Invalid public key length: {}", public_key.len()));
        let address = Address::new(public_key_byte_array);
        Account {
            seed,
            address,
            key_pair,
        }
    }

    /// Create an account from 64-byte Ed25519 key pair
    ///
    /// Note the first 32 bytes of the Ed25519 key is the seed, the second 32 bytes is the public key
    pub fn from_key_pair(key_pair: [u8; 64]) -> Result<Account, TransactionError> {
        let seed = key_pair[0..32].try_into().unwrap();

        Ok(Account::from_seed(seed))
    }

    #[cfg(test)]
    pub(crate) fn raw_public_key(&self) -> &[u8] {
        self.key_pair.public_key().as_ref()
    }

    /// Get the public key address of the account
    pub fn address(&self) -> Address {
        self.address
    }

    /// Get the human readable mnemonic of the 32 byte seed
    pub fn mnemonic(&self) -> String {
        mnemonic::from_key(&self.seed).unwrap()
    }

    /// Get the 32 byte seed
    pub fn seed(&self) -> [u8; 32] {
        self.seed
    }

    /// Sign the given bytes, and wrap in Signature.
    fn generate_raw_sig(&self, bytes: &[u8]) -> Signature {
        let signature = self.key_pair.sign(bytes);
        // ring returns a signature with padding at the end to make it 105 bytes, only 64 bytes are actually used
        let stripped_signature: [u8; 64] = signature.as_ref()[..64]
            .try_into()
            // unwrap: we passed ..64, try_into() always succeeds
            .unwrap();
        Signature(stripped_signature)
    }

    /// Sign the given bytes, and wrap in signature. The message is prepended with an identifier for domain separation.
    pub fn generate_sig(&self, bytes: &[u8]) -> Signature {
        let mut bytes_sign_prefix = b"MX".to_vec();
        bytes_sign_prefix.extend_from_slice(bytes);
        self.generate_raw_sig(&bytes_sign_prefix)
    }

    pub fn generate_program_sig(&self, program: &CompiledTeal) -> Signature {
        self.generate_raw_sig(&program.bytes_to_sign())
    }

    fn generate_transaction_sig(
        &self,
        transaction: &Transaction,
    ) -> Result<Signature, TransactionError> {
        Ok(self.generate_raw_sig(&transaction.bytes_to_sign()?))
    }

    /// Sign a bid with the account's private key
    pub fn sign_bid(&self, bid: Bid) -> Result<SignedBid, TransactionError> {
        let encoded_bid = bid.to_msg_pack()?;
        let mut prefix_encoded_bid = b"aB".to_vec();
        prefix_encoded_bid.extend_from_slice(&encoded_bid);
        let signature = self.generate_raw_sig(&prefix_encoded_bid);
        Ok(SignedBid {
            bid,
            sig: signature,
        })
    }

    /// Sign transaction and generate a single signature SignedTransaction
    pub fn sign_transaction(
        &self,
        transaction: Transaction,
    ) -> Result<SignedTransaction, TransactionError> {
        let transaction_id = transaction.id()?;
        let sig = TransactionSignature::Single(self.generate_transaction_sig(&transaction)?);
        let auth_address = auth_address(&transaction, &self.address);
        Ok(SignedTransaction {
            transaction,
            transaction_id,
            sig,
            auth_address,
        })
    }

    /// Creates transaction multi signature corresponding to multisign addresses, inserting own signature
    pub fn init_transaction_msig(
        &self,
        transaction: &Transaction,
        from: &MultisigAddress,
    ) -> Result<MultisigSignature, TransactionError> {
        if from.address() != transaction.sender() {
            return Err(TransactionError::InvalidSenderInMultisig);
        }
        if !from.contains(&self.address) {
            return Err(TransactionError::InvalidSecretKeyInMultisig);
        }

        Ok(self.init_msig(from, self.generate_transaction_sig(transaction)?))
    }

    /// Creates logic multi signature corresponding to multisign addresses, inserting own signature
    pub fn init_logic_msig(
        &self,
        program: &CompiledTeal,
        ma: &MultisigAddress,
    ) -> Result<MultisigSignature, TransactionError> {
        if !ma.contains(&self.address) {
            return Err(TransactionError::InvalidSecretKeyInMultisig);
        }

        Ok(self.init_msig(ma, self.generate_program_sig(program)))
    }

    pub fn append_to_logic_msig(
        &self,
        program: &CompiledTeal,
        msig: MultisigSignature,
    ) -> Result<MultisigSignature, TransactionError> {
        self.append_sig_to_msig(self.generate_program_sig(program), msig)
    }

    pub fn append_to_transaction_msig(
        &self,
        transaction: &Transaction,
        msig: MultisigSignature,
    ) -> Result<MultisigSignature, TransactionError> {
        self.append_sig_to_msig(self.generate_transaction_sig(transaction)?, msig)
    }

    /// Creates multi signature corresponding to multisign addresses, inserting own signature
    fn init_msig(&self, ma: &MultisigAddress, sig: Signature) -> MultisigSignature {
        let my_public_key = self.address.as_public_key();
        let subsigs: Vec<MultisigSubsig> = ma
            .public_keys
            .iter()
            .map(|key| {
                if *key == my_public_key {
                    MultisigSubsig {
                        key: *key,
                        sig: Some(sig),
                    }
                } else {
                    MultisigSubsig {
                        key: *key,
                        sig: None,
                    }
                }
            })
            .collect();

        MultisigSignature {
            version: ma.version,
            threshold: ma.threshold,
            subsigs,
        }
    }

    /// Inserts signature in multi signature
    /// Private: Assumes that my_sig was generated with this account
    fn append_sig_to_msig(
        &self,
        my_sig: Signature,
        msig: MultisigSignature,
    ) -> Result<MultisigSignature, TransactionError> {
        let my_public_key = self.address.as_public_key();
        if !msig
            .subsigs
            .iter()
            .any(|s: &MultisigSubsig| s.key == my_public_key)
        {
            return Err(TransactionError::InvalidSecretKeyInMultisig);
        }

        let subsigs: Vec<MultisigSubsig> = msig
            .subsigs
            .iter()
            .map(|subsig| {
                if subsig.key == my_public_key {
                    MultisigSubsig {
                        key: subsig.key,
                        sig: Some(my_sig),
                    }
                } else {
                    subsig.clone()
                }
            })
            .collect();
        Ok(MultisigSignature { subsigs, ..msig })
    }
}

/// Returns auth address to be set in signed transaction
pub(crate) fn auth_address(tx: &Transaction, signing_account_address: &Address) -> Option<Address> {
    (&tx.sender() != signing_account_address)
        .then_some(signing_account_address)
        .copied()
}

#[cfg(test)]
mod tests {
    use crate::account::Account;
    use algonaut_core::Address;
    use algonaut_crypto::{mnemonic, Signature};
    use data_encoding::BASE64;
    use rand::Rng;
    use std::convert::TryInto;

    #[test]
    fn test_public_key_is_correct() {
        let mnemonic = "actress tongue harbor tray suspect odor load topple vocal avoid ignore apple lunch unknown tissue museum once switch captain place lemon sail outdoor absent creek";
        let address: Address = "DPLD3RTSWC5STVBPZL5DIIVE2OC4BSAWTOYBLFN2X6EFLT2ZNF4SMX64UA"
            .parse()
            .unwrap();

        let account = Account::from_mnemonic(mnemonic).unwrap();
        let public_key_slice = account.raw_public_key();
        let public_key_bytes: [u8; 32] = public_key_slice.try_into().unwrap();
        assert_eq!(Address(public_key_bytes), address);
    }

    #[test]
    fn test_key_gen() {
        for _ in 0..100 {
            let account = Account::generate();
            let public_key_slice = account.raw_public_key();
            let public_key_bytes: [u8; 32] = public_key_slice.try_into().unwrap();
            assert_eq!(Address(public_key_bytes), account.address());
        }
    }

    #[test]
    fn test_to_mnemonic() {
        let mnemonic = "actress tongue harbor tray suspect odor load topple vocal avoid ignore apple lunch unknown tissue museum once switch captain place lemon sail outdoor absent creek";
        let account = Account::from_mnemonic(mnemonic).unwrap();
        assert_eq!(account.mnemonic(), mnemonic);
    }

    #[test]
    fn test_sign_bytes() {
        let mut b = rand::thread_rng().gen::<[u8; 15]>();
        let account = Account::generate();
        let signature = account.generate_sig(&b);

        assert!(account.address().verify_bytes(&b, &signature));
        b[0] = b[0].wrapping_add(1);
        assert!(!account.address().verify_bytes(&b, &signature));
    }

    #[test]
    fn test_verify_bytes() {
        let mut message = BASE64.decode("rTs7+dUj".as_bytes()).unwrap();
        let signature = Signature(BASE64.decode("COEBmoD+ysVECoyVOAsvMAjFxvKeQVkYld+RSHMnEiHsypqrfj2EdYqhrm4t7dK3ZOeSQh3aXiZK/zqQDTPBBw==".as_bytes()).unwrap().try_into().unwrap());
        let address: Address = "DPLD3RTSWC5STVBPZL5DIIVE2OC4BSAWTOYBLFN2X6EFLT2ZNF4SMX64UA"
            .parse()
            .unwrap();

        assert!(address.verify_bytes(&message, &signature));
        message[0] = message[0].wrapping_add(1);
        assert!(!address.verify_bytes(&message, &signature));
    }

    #[test]
    #[ignore]
    fn test_teal_sign() {
        // TODO TestAccount.testTealSign()
        // TODO implement TEAL signing
    }

    #[test]
    fn test_to_seed() {
        let mnemonic = "actress tongue harbor tray suspect odor load topple vocal avoid ignore apple lunch unknown tissue museum once switch captain place lemon sail outdoor absent creek";
        let seed = mnemonic::to_key(mnemonic).unwrap();
        let account = Account::from_seed(seed);
        assert_eq!(mnemonic::to_key(&account.mnemonic()).unwrap(), seed);
    }
}
