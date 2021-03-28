use crate::error::AlgorandError;
use algonaut_core::MultisigSignature;
use algonaut_crypto::{Ed25519PublicKey, MasterDerivationKey};
use algonaut_transaction::Transaction;
use message::*;

/// API message structs for Algorand's kmd v1
pub mod message;

const KMD_TOKEN_HEADER: &str = "X-KMD-API-Token";

/// Client for interacting with the key management daemon
pub struct Client {
    pub(super) address: String,
    pub(super) token: String,
    pub(super) http_client: reqwest::blocking::Client,
}

impl Client {
    pub(super) fn new(address: &str, token: &str) -> Client {
        Client {
            address: address.to_string(),
            token: token.to_string(),
            http_client: reqwest::blocking::Client::new(),
        }
    }

    /// Retrieves the current version
    pub fn versions(&self) -> Result<VersionsResponse, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}versions", self.address))
            .header(KMD_TOKEN_HEADER, &self.token)
            .header("Accept", "application/json")
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// List all of the wallets that kmd is aware of
    pub fn list_wallets(&self) -> Result<ListWalletsResponse, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v1/wallets", self.address))
            .header(KMD_TOKEN_HEADER, &self.token)
            .header("Accept", "application/json")
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Creates a wallet
    pub fn create_wallet(
        &self,
        wallet_name: &str,
        wallet_password: &str,
        wallet_driver_name: &str,
        master_derivation_key: MasterDerivationKey,
    ) -> Result<CreateWalletResponse, AlgorandError> {
        let req = CreateWalletRequest {
            master_derivation_key,
            wallet_driver_name: wallet_driver_name.to_string(),
            wallet_name: wallet_name.to_string(),
            wallet_password: wallet_password.to_string(),
        };

        let response = self
            .http_client
            .post(&format!("{}v1/wallet", self.address))
            .header(KMD_TOKEN_HEADER, &self.token)
            .header("Accept", "application/json")
            .json(&req)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Unlock the wallet and return a wallet token that can be used for subsequent operations
    ///
    /// These tokens expire periodically and must be renewed.
    /// You can see how much time remains until expiration with [get_wallet_info](Client::get_wallet_info)
    /// and renew it with [renew_wallet_handle](Client::renew_wallet_handle).
    /// When you're done, you can invalidate the token with [release_wallet_handle](Client::release_wallet_handle)
    pub fn init_wallet_handle(
        &self,
        wallet_id: &str,
        wallet_password: &str,
    ) -> Result<InitWalletHandleResponse, AlgorandError> {
        let req = InitWalletHandleRequest {
            wallet_id: wallet_id.to_string(),
            wallet_password: wallet_password.to_string(),
        };
        let response = self
            .http_client
            .post(&format!("{}v1/wallet/init", self.address))
            .header(KMD_TOKEN_HEADER, &self.token)
            .header("Accept", "application/json")
            .json(&req)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Release a wallet handle token
    pub fn release_wallet_handle(
        &self,
        wallet_handle: &str,
    ) -> Result<ReleaseWalletHandleResponse, AlgorandError> {
        let req = ReleaseWalletHandleRequest {
            wallet_handle_token: wallet_handle.to_string(),
        };
        let response = self
            .http_client
            .post(&format!("{}v1/wallet/release", self.address))
            .header(KMD_TOKEN_HEADER, &self.token)
            .header("Accept", "application/json")
            .json(&req)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Renew a wallet handle token
    pub fn renew_wallet_handle(
        &self,
        wallet_handle: &str,
    ) -> Result<RenewWalletHandleResponse, AlgorandError> {
        let req = RenewWalletHandleRequest {
            wallet_handle_token: wallet_handle.to_string(),
        };
        let response = self
            .http_client
            .post(&format!("{}v1/wallet/renew", self.address))
            .header(KMD_TOKEN_HEADER, &self.token)
            .header("Accept", "application/json")
            .json(&req)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Rename a wallet
    pub fn rename_wallet(
        &self,
        wallet_id: &str,
        wallet_password: &str,
        new_name: &str,
    ) -> Result<RenameWalletResponse, AlgorandError> {
        let req = RenameWalletRequest {
            wallet_id: wallet_id.to_string(),
            wallet_password: wallet_password.to_string(),
            wallet_name: new_name.to_string(),
        };
        let response = self
            .http_client
            .post(&format!("{}v1/wallet/rename", self.address))
            .header(KMD_TOKEN_HEADER, &self.token)
            .header("Accept", "application/json")
            .json(&req)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Get wallet info
    pub fn get_wallet_info(
        &self,
        wallet_handle: &str,
    ) -> Result<GetWalletInfoResponse, AlgorandError> {
        let req = GetWalletInfoRequest {
            wallet_handle_token: wallet_handle.to_string(),
        };
        let response = self
            .http_client
            .post(&format!("{}v1/wallet/info", self.address))
            .header(KMD_TOKEN_HEADER, &self.token)
            .header("Accept", "application/json")
            .json(&req)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Export the master derivation key from a wallet
    pub fn export_master_derivation_key(
        &self,
        wallet_handle: &str,
        wallet_password: &str,
    ) -> Result<ExportMasterDerivationKeyResponse, AlgorandError> {
        let req = ExportMasterDerivationKeyRequest {
            wallet_handle_token: wallet_handle.to_string(),
            wallet_password: wallet_password.to_string(),
        };
        let response = self
            .http_client
            .post(&format!("{}v1/master-key/export", self.address))
            .header(KMD_TOKEN_HEADER, &self.token)
            .header("Accept", "application/json")
            .json(&req)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Import an externally generated key into the wallet
    pub fn import_key(
        &self,
        wallet_handle: &str,
        private_key: [u8; 32],
    ) -> Result<ImportKeyResponse, AlgorandError> {
        let req = ImportKeyRequest {
            wallet_handle_token: wallet_handle.to_string(),
            private_key,
        };
        let response = self
            .http_client
            .post(&format!("{}v1/key/import", self.address))
            .header(KMD_TOKEN_HEADER, &self.token)
            .header("Accept", "application/json")
            .json(&req)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Export the Ed25519 seed associated with the passed address
    ///
    /// Note the first 32 bytes of the returned value is the seed, the second 32 bytes is the public key
    pub fn export_key(
        &self,
        wallet_handle: &str,
        wallet_password: &str,
        address: &str,
    ) -> Result<ExportKeyResponse, AlgorandError> {
        let req = ExportKeyRequest {
            wallet_handle_token: wallet_handle.to_string(),
            address: address.to_string(),
            wallet_password: wallet_password.to_string(),
        };
        let response = self
            .http_client
            .post(&format!("{}v1/key/export", self.address))
            .header(KMD_TOKEN_HEADER, &self.token)
            .header("Accept", "application/json")
            .json(&req)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Generates a key and adds it to the wallet, returning the public key
    pub fn generate_key(&self, wallet_handle: &str) -> Result<GenerateKeyResponse, AlgorandError> {
        let req = GenerateKeyRequest {
            wallet_handle_token: wallet_handle.to_string(),
            display_mnemonic: false,
        };
        let response = self
            .http_client
            .post(&format!("{}v1/key", self.address))
            .header(KMD_TOKEN_HEADER, &self.token)
            .header("Accept", "application/json")
            .json(&req)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Deletes the key from the wallet
    pub fn delete_key(
        &self,
        wallet_handle: &str,
        wallet_password: &str,
        address: &str,
    ) -> Result<DeleteKeyResponse, AlgorandError> {
        let req = DeleteKeyRequest {
            wallet_handle_token: wallet_handle.to_string(),
            wallet_password: wallet_password.to_string(),
            address: address.to_string(),
        };
        let response = self
            .http_client
            .delete(&format!("{}v1/key", self.address))
            .header(KMD_TOKEN_HEADER, &self.token)
            .header("Accept", "application/json")
            .json(&req)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// List all of the public keys in the wallet
    pub fn list_keys(&self, wallet_handle: &str) -> Result<ListKeysResponse, AlgorandError> {
        let req = ListKeysRequest {
            wallet_handle_token: wallet_handle.to_string(),
        };
        let response = self
            .http_client
            .post(&format!("{}v1/key/list", self.address))
            .header(KMD_TOKEN_HEADER, &self.token)
            .header("Accept", "application/json")
            .json(&req)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Sign a transaction
    pub fn sign_transaction(
        &self,
        wallet_handle: &str,
        wallet_password: &str,
        transaction: &Transaction,
    ) -> Result<SignTransactionResponse, AlgorandError> {
        let transaction_bytes = rmp_serde::to_vec_named(transaction)?;
        let req = SignTransactionRequest {
            wallet_handle_token: wallet_handle.to_string(),
            transaction: transaction_bytes,
            wallet_password: wallet_password.to_string(),
        };
        let response = self
            .http_client
            .post(&format!("{}v1/transaction/sign", self.address))
            .header(KMD_TOKEN_HEADER, &self.token)
            .header("Accept", "application/json")
            .json(&req)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Lists all of the multisig accounts whose preimages this wallet stores
    pub fn list_multisig(
        &self,
        wallet_handle: &str,
    ) -> Result<ListMultisigResponse, AlgorandError> {
        let req = ListMultisigRequest {
            wallet_handle_token: wallet_handle.to_string(),
        };
        let response = self
            .http_client
            .post(&format!("{}v1/multisig/list", self.address))
            .header(KMD_TOKEN_HEADER, &self.token)
            .header("Accept", "application/json")
            .json(&req)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Import a multisig account
    pub fn import_multisig(
        &self,
        wallet_handle: &str,
        version: u8,
        threshold: u8,
        pks: &[Ed25519PublicKey],
    ) -> Result<ImportMultisigResponse, AlgorandError> {
        let req = ImportMultisigRequest {
            wallet_handle_token: wallet_handle.to_string(),
            multisig_version: version,
            threshold,
            pks: pks.to_vec(),
        };
        let response = self
            .http_client
            .post(&format!("{}v1/multisig/import", self.address))
            .header(KMD_TOKEN_HEADER, &self.token)
            .header("Accept", "application/json")
            .json(&req)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Export multisig address metadata
    pub fn export_multisig(
        &self,
        wallet_handle: &str,
        address: &str,
    ) -> Result<ExportMultisigResponse, AlgorandError> {
        let req = ExportMultisigRequest {
            wallet_handle_token: wallet_handle.to_string(),
            address: address.to_string(),
        };
        let response = self
            .http_client
            .post(&format!("{}v1/multisig/export", self.address))
            .header(KMD_TOKEN_HEADER, &self.token)
            .header("Accept", "application/json")
            .json(&req)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Delete a multisig from the wallet
    pub fn delete_multisig(
        &self,
        wallet_handle: &str,
        wallet_password: &str,
        address: &str,
    ) -> Result<DeleteMultisigResponse, AlgorandError> {
        let req = DeleteMultisigRequest {
            wallet_handle_token: wallet_handle.to_string(),
            wallet_password: wallet_password.to_string(),
            address: address.to_string(),
        };
        let response = self
            .http_client
            .delete(&format!("{}v1/multisig", self.address))
            .header(KMD_TOKEN_HEADER, &self.token)
            .header("Accept", "application/json")
            .json(&req)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Sign a multisig transaction.
    ///
    /// Start a multisig signature or add a signature to a partially completed multisig signature.
    pub fn sign_multisig_transaction(
        &self,
        wallet_handle: &str,
        wallet_password: &str,
        transaction: &Transaction,
        public_key: Ed25519PublicKey,
        partial_multisig: Option<MultisigSignature>,
    ) -> Result<SignMultisigTransactionResponse, AlgorandError> {
        let transaction_bytes = rmp_serde::to_vec_named(transaction)?;
        let req = SignMultisigTransactionRequest {
            wallet_handle_token: wallet_handle.to_string(),
            transaction: transaction_bytes,
            public_key,
            partial_multisig,
            wallet_password: wallet_password.to_string(),
        };
        let response = self
            .http_client
            .post(&format!("{}v1/multisig/sign", self.address))
            .header(KMD_TOKEN_HEADER, &self.token)
            .header("Accept", "application/json")
            .json(&req)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_wallet_request_successful_serialization() {
        let req = CreateWalletRequest {
            master_derivation_key: MasterDerivationKey([0; 32]),
            wallet_driver_name: "sqlite".into(),
            wallet_name: "testwallet".into(),
            wallet_password: "testpassword".into(),
        };

        let json = serde_json::to_string(&req);
        assert!(json.is_ok());
        println!("{:#?}", json.unwrap());
    }

    #[test]
    fn test_apiv1_wallet_successful_deserialization() {
        let wallet: Result<CreateWalletResponse, serde_json::Error> = serde_json::from_str(
            r#"
        {
            "wallet": {
              "driver_name": "sqlite",
              "driver_version": 1,
              "id": "07d6a46bbc3e64abe6062f8e08ba9c3b",
              "mnemonic_ux": false,
              "name": "name3",
              "supported_txs": [
                "pay",
                "keyreg"
              ]
            }
        }
        "#,
        );
        println!("{:#?}", wallet);
        assert!(wallet.is_ok());
    }
}
