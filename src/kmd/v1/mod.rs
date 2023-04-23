use crate::Error;
use algonaut_core::{Address, MultisigSignature, ToMsgPack};
use algonaut_crypto::{Ed25519PublicKey, MasterDerivationKey};
use algonaut_kmd::{kmd::v1::Client, token::ApiToken, Headers};
use algonaut_model::kmd::v1::{
    CreateWalletResponse, DeleteKeyResponse, DeleteMultisigResponse, ExportKeyResponse,
    ExportMasterDerivationKeyResponse, ExportMultisigResponse, GenerateKeyResponse,
    GetWalletInfoResponse, ImportKeyResponse, ImportMultisigResponse, InitWalletHandleResponse,
    ListKeysResponse, ListMultisigResponse, ListWalletsResponse, ReleaseWalletHandleResponse,
    RenameWalletResponse, RenewWalletHandleResponse, SignMultisigTransactionResponse,
    SignTransactionResponse, VersionsResponse,
};
use algonaut_transaction::Transaction;

#[derive(Debug, Clone)]
pub struct Kmd {
    pub(crate) client: Client,
}

impl Kmd {
    /// Build a v1 client for the Algorand key management daemon.
    ///
    /// For third party providers / custom headers, use [with_headers](Self::with_headers).
    ///
    /// Returns an error if the url or token have an invalid format.
    pub fn new(url: &str, token: &str) -> Result<Kmd, Error> {
        Self::with_headers(
            url,
            vec![("X-KMD-API-Token", &ApiToken::parse(token)?.to_string())],
        )
    }

    /// Build a v1 client for the Algorand key management daemon.
    ///
    /// Use this initializer when interfacing with third party services, that require custom headers.
    ///
    /// Returns an error if the url or headers have an invalid format.
    pub fn with_headers(url: &str, headers: Headers) -> Result<Kmd, Error> {
        Ok(Kmd {
            client: Client::new(url, headers)?,
        })
    }

    /// Retrieves the current version
    pub async fn versions(&self) -> Result<VersionsResponse, Error> {
        Ok(self.client.versions().await?)
    }

    /// List all of the wallets that kmd is aware of
    pub async fn list_wallets(&self) -> Result<ListWalletsResponse, Error> {
        Ok(self.client.list_wallets().await?)
    }

    /// Creates a wallet
    pub async fn create_wallet(
        &self,
        wallet_name: &str,
        wallet_password: &str,
        wallet_driver_name: &str,
        master_derivation_key: MasterDerivationKey,
    ) -> Result<CreateWalletResponse, Error> {
        Ok(self
            .client
            .create_wallet(
                wallet_name,
                wallet_password,
                wallet_driver_name,
                master_derivation_key,
            )
            .await?)
    }

    /// Unlock the wallet and return a wallet token that can be used for subsequent operations
    ///
    /// These tokens expire periodically and must be renewed.
    /// You can see how much time remains until expiration with [get_wallet_info](Client::get_wallet_info)
    /// and renew it with [renew_wallet_handle](Client::renew_wallet_handle).
    /// When you're done, you can invalidate the token with [release_wallet_handle](Client::release_wallet_handle)
    pub async fn init_wallet_handle(
        &self,
        wallet_id: &str,
        wallet_password: &str,
    ) -> Result<InitWalletHandleResponse, Error> {
        Ok(self
            .client
            .init_wallet_handle(wallet_id, wallet_password)
            .await?)
    }

    /// Release a wallet handle token
    pub async fn release_wallet_handle(
        &self,
        wallet_handle: &str,
    ) -> Result<ReleaseWalletHandleResponse, Error> {
        Ok(self.client.release_wallet_handle(wallet_handle).await?)
    }

    /// Renew a wallet handle token
    pub async fn renew_wallet_handle(
        &self,
        wallet_handle: &str,
    ) -> Result<RenewWalletHandleResponse, Error> {
        Ok(self.client.renew_wallet_handle(wallet_handle).await?)
    }

    /// Rename a wallet
    pub async fn rename_wallet(
        &self,
        wallet_id: &str,
        wallet_password: &str,
        new_name: &str,
    ) -> Result<RenameWalletResponse, Error> {
        Ok(self
            .client
            .rename_wallet(wallet_id, wallet_password, new_name)
            .await?)
    }

    /// Get wallet info
    pub async fn get_wallet_info(
        &self,
        wallet_handle: &str,
    ) -> Result<GetWalletInfoResponse, Error> {
        Ok(self.client.get_wallet_info(wallet_handle).await?)
    }

    /// Export the master derivation key from a wallet
    pub async fn export_master_derivation_key(
        &self,
        wallet_handle: &str,
        wallet_password: &str,
    ) -> Result<ExportMasterDerivationKeyResponse, Error> {
        Ok(self
            .client
            .export_master_derivation_key(wallet_handle, wallet_password)
            .await?)
    }

    /// Import an externally generated key into the wallet
    pub async fn import_key(
        &self,
        wallet_handle: &str,
        private_key: [u8; 32],
    ) -> Result<ImportKeyResponse, Error> {
        Ok(self.client.import_key(wallet_handle, private_key).await?)
    }

    /// Export the Ed25519 seed associated with the passed address
    ///
    /// Note the first 32 bytes of the returned value is the seed, the second 32 bytes is the public key
    pub async fn export_key(
        &self,
        wallet_handle: &str,
        wallet_password: &str,
        address: &Address,
    ) -> Result<ExportKeyResponse, Error> {
        Ok(self
            .client
            .export_key(wallet_handle, wallet_password, address)
            .await?)
    }

    /// Generates a key and adds it to the wallet, returning the public key
    pub async fn generate_key(&self, wallet_handle: &str) -> Result<GenerateKeyResponse, Error> {
        Ok(self.client.generate_key(wallet_handle).await?)
    }

    /// Deletes the key from the wallet
    pub async fn delete_key(
        &self,
        wallet_handle: &str,
        wallet_password: &str,
        address: &str,
    ) -> Result<DeleteKeyResponse, Error> {
        Ok(self
            .client
            .delete_key(wallet_handle, wallet_password, address)
            .await?)
    }

    /// List all of the public keys in the wallet
    pub async fn list_keys(&self, wallet_handle: &str) -> Result<ListKeysResponse, Error> {
        Ok(self.client.list_keys(wallet_handle).await?)
    }

    /// Sign a transaction
    pub async fn sign_transaction(
        &self,
        wallet_handle: &str,
        wallet_password: &str,
        transaction: &Transaction,
    ) -> Result<SignTransactionResponse, Error> {
        Ok(self
            .client
            .sign_transaction(wallet_handle, wallet_password, transaction.to_msg_pack()?)
            .await?)
    }

    /// Lists all of the multisig accounts whose preimages this wallet stores
    pub async fn list_multisig(&self, wallet_handle: &str) -> Result<ListMultisigResponse, Error> {
        Ok(self.client.list_multisig(wallet_handle).await?)
    }

    /// Import a multisig account
    pub async fn import_multisig(
        &self,
        wallet_handle: &str,
        version: u8,
        threshold: u8,
        pks: &[Ed25519PublicKey],
    ) -> Result<ImportMultisigResponse, Error> {
        Ok(self
            .client
            .import_multisig(wallet_handle, version, threshold, pks)
            .await?)
    }

    /// Export multisig address metadata
    pub async fn export_multisig(
        &self,
        wallet_handle: &str,
        address: &str,
    ) -> Result<ExportMultisigResponse, Error> {
        Ok(self.client.export_multisig(wallet_handle, address).await?)
    }

    /// Delete a multisig from the wallet
    pub async fn delete_multisig(
        &self,
        wallet_handle: &str,
        wallet_password: &str,
        address: &str,
    ) -> Result<DeleteMultisigResponse, Error> {
        Ok(self
            .client
            .delete_multisig(wallet_handle, wallet_password, address)
            .await?)
    }

    /// Sign a multisig transaction.
    ///
    /// Start a multisig signature or add a signature to a partially completed multisig signature.
    pub async fn sign_multisig_transaction(
        &self,
        wallet_handle: &str,
        wallet_password: &str,
        transaction: &Transaction,
        public_key: Ed25519PublicKey,
        partial_multisig: Option<MultisigSignature>,
    ) -> Result<SignMultisigTransactionResponse, Error> {
        Ok(self
            .client
            .sign_multisig_transaction(
                wallet_handle,
                wallet_password,
                transaction.to_msg_pack()?,
                public_key,
                partial_multisig,
            )
            .await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_client_creation() {
        let kmd = Kmd::new(
            "http://example.com",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        assert!(kmd.ok().is_some());
    }

    #[test]
    #[should_panic(expected = "")]
    fn test_client_creation_with_empty_token() {
        Kmd::new("http://example.com", "").unwrap();
    }

    #[test]
    #[should_panic(expected = "")]
    fn test_client_builder_with_empty_url() {
        Kmd::new(
            "",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .unwrap();
    }
}
