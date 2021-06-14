use crate::auction::{Bid, SignedBid};
use crate::error::{AlgorandError, ApiError};
use crate::transaction::{SignedTransaction, Transaction, TransactionSignature};
use algonaut_core::{
    Address, CompiledTeal, MultisigAddress, MultisigSignature, MultisigSubsig, Signature, ToMsgPack,
};
use algonaut_crypto::mnemonic;
use rand::rngs::OsRng;
use rand::Rng;
use ring::signature::Ed25519KeyPair as KeyPairType;
use ring::signature::KeyPair;

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
    pub fn from_mnemonic(mnemonic: &str) -> Result<Account, AlgorandError> {
        let seed = mnemonic::to_key(mnemonic)?;
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
        mnemonic::from_key(&self.seed).unwrap()
    }

    /// Get the 32 byte seed
    pub fn seed(&self) -> [u8; 32] {
        self.seed
    }

    fn generate_sig(&self, bytes: &[u8]) -> Signature {
        let signature = self.key_pair.sign(&bytes);
        // ring returns a signature with padding at the end to make it 105 bytes, only 64 bytes are actually used
        let mut stripped_signature = [0; 64];
        stripped_signature.copy_from_slice(&signature.as_ref()[..64]);
        Signature(stripped_signature)
    }

    pub fn generate_program_sig(&self, program: &CompiledTeal) -> Signature {
        self.generate_sig(&["Program".as_bytes(), &program.bytes].concat())
    }

    fn generate_transaction_sig(
        &self,
        transaction: &Transaction,
    ) -> Result<Signature, AlgorandError> {
        Ok(self.generate_sig(&transaction.bytes_to_sign()?))
    }

    /// Sign a bid with the account's private key
    pub fn sign_bid(&self, bid: Bid) -> Result<SignedBid, AlgorandError> {
        let encoded_bid = bid.to_msg_pack()?;
        let mut prefix_encoded_bid = b"aB".to_vec();
        prefix_encoded_bid.extend_from_slice(&encoded_bid);
        let signature = self.generate_sig(&prefix_encoded_bid);
        Ok(SignedBid {
            bid,
            sig: signature,
        })
    }

    /// Sign transaction and generate a single signature SignedTransaction
    pub fn sign_transaction(
        &self,
        transaction: &Transaction,
    ) -> Result<SignedTransaction, AlgorandError> {
        Ok(SignedTransaction {
            transaction: transaction.clone(),
            transaction_id: transaction.id()?,
            sig: TransactionSignature::Single(self.generate_transaction_sig(&transaction)?),
        })
    }

    /// Sign transaction and generate a multi signature SignedTransaction
    pub fn sign_multisig_transaction(
        &self,
        from: MultisigAddress,
        transaction: &Transaction,
    ) -> Result<SignedTransaction, AlgorandError> {
        Ok(SignedTransaction {
            transaction: transaction.clone(),
            transaction_id: transaction.id()?,
            sig: TransactionSignature::Multi(self.init_transaction_msig(transaction, from)?),
        })
    }

    /// Creates transaction multi signature corresponding to multisign addresses, inserting own signature
    pub fn init_transaction_msig(
        &self,
        transaction: &Transaction,
        from: MultisigAddress,
    ) -> Result<MultisigSignature, AlgorandError> {
        if from.address() != transaction.sender {
            return Err(ApiError::InvalidSenderInMultisig.into());
        }
        if !from.contains(&self.address) {
            return Err(ApiError::InvalidSecretKeyInMultisig.into());
        }

        Ok(self.init_msig(from, self.generate_transaction_sig(&transaction)?))
    }

    /// Creates logic multi signature corresponding to multisign addresses, inserting own signature
    pub fn init_logic_msig(
        &self,
        program: &CompiledTeal,
        ma: MultisigAddress,
    ) -> Result<MultisigSignature, AlgorandError> {
        if !ma.contains(&self.address) {
            return Err(ApiError::InvalidSecretKeyInMultisig.into());
        }

        Ok(self.init_msig(ma, self.generate_program_sig(program)))
    }

    pub fn append_to_logic_msig(
        &self,
        program: &CompiledTeal,
        msig: MultisigSignature,
    ) -> Result<MultisigSignature, AlgorandError> {
        self.append_sig_to_msig(self.generate_program_sig(program), msig)
    }

    pub fn append_to_transaction_msig(
        &self,
        transaction: &Transaction,
        msig: MultisigSignature,
    ) -> Result<MultisigSignature, AlgorandError> {
        self.append_sig_to_msig(self.generate_transaction_sig(transaction)?, msig)
    }

    /// Creates multi signature corresponding to multisign addresses, inserting own signature
    fn init_msig(&self, ma: MultisigAddress, sig: Signature) -> MultisigSignature {
        let my_public_key = self.address.as_public_key();
        let subsigs: Vec<MultisigSubsig> = ma
            .public_keys
            .into_iter()
            .map(|key| {
                if key == my_public_key {
                    MultisigSubsig {
                        key,
                        sig: Some(sig),
                    }
                } else {
                    MultisigSubsig { key, sig: None }
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
    ) -> Result<MultisigSignature, AlgorandError> {
        let my_public_key = self.address.as_public_key();
        if !msig
            .subsigs
            .iter()
            .any(|s: &MultisigSubsig| s.key == my_public_key)
        {
            return Err(ApiError::InvalidSecretKeyInMultisig.into());
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
