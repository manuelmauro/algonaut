use serde::Deserialize;

use crate::crypto::MultisigSignature;
use crate::kmd::requests::*;
use crate::kmd::responses::*;
use crate::transaction::Transaction;
use crate::{Ed25519PublicKey, Error, MasterDerivationKey};

const KMD_TOKEN_HEADER: &str = "X-KMD-API-Token";

/// Client for interacting with the key management daemon
pub struct KmdClient {
    address: String,
    token: String,
    http_client: reqwest::Client,
}

impl KmdClient {
    pub fn new(address: &str, token: &str) -> KmdClient {
        KmdClient {
            address: address.to_string(),
            token: token.to_string(),
            http_client: reqwest::Client::new(),
        }
    }

    /// Retrieves the current version
    pub fn versions(&self) -> Result<VersionsResponse, Error> {
        self.do_v1_request(VersionsRequest)
    }

    /// List all of the wallets that kmd is aware of
    pub fn list_wallets(&self) -> Result<ListWalletsResponse, Error> {
        self.do_v1_request(ListWalletsRequest)
    }

    /// Creates a wallet
    pub fn create_wallet(
        &self,
        wallet_name: &str,
        wallet_password: &str,
        wallet_driver_name: &str,
        master_derivation_key: MasterDerivationKey,
    ) -> Result<CreateWalletResponse, Error> {
        let req = CreateWalletRequest {
            master_derivation_key,
            wallet_driver_name: wallet_driver_name.to_string(),
            wallet_name: wallet_name.to_string(),
            wallet_password: wallet_password.to_string(),
        };
        self.do_v1_request(req)
    }

    /// Unlock the wallet and return a wallet token that can be used for subsequent operations
    ///
    /// These tokens expire periodically and must be renewed.
    /// You can see how much time remains until expiration with [get_wallet](KmdClient::get_wallet) and renew it with [renew_wallet_handle](KmdClient::renew_wallet_handle).
    /// When you're done, you can invalidate the token with [release_wallet_handle](KmdClient::release_wallet_handle)
    pub fn init_wallet_handle(
        &self,
        wallet_id: &str,
        wallet_password: &str,
    ) -> Result<InitWalletHandleResponse, Error> {
        let req = InitWalletHandleRequest {
            wallet_id: wallet_id.to_string(),
            wallet_password: wallet_password.to_string(),
        };
        self.do_v1_request(req)
    }

    /// Release a wallet handle token
    pub fn release_wallet_handle(
        &self,
        wallet_handle: &str,
    ) -> Result<ReleaseWalletHandleResponse, Error> {
        let req = ReleaseWalletHandleRequest {
            wallet_handle_token: wallet_handle.to_string(),
        };
        self.do_v1_request(req)
    }

    /// Renew a wallet handle token
    pub fn renew_wallet_handle(
        &self,
        wallet_handle: &str,
    ) -> Result<RenewWalletHandleResponse, Error> {
        let req = RenewWalletHandleRequest {
            wallet_handle_token: wallet_handle.to_string(),
        };
        self.do_v1_request(req)
    }

    /// Rename a wallet
    pub fn rename_wallet(
        &self,
        wallet_id: &str,
        wallet_password: &str,
        new_name: &str,
    ) -> Result<RenameWalletResponse, Error> {
        let req = RenameWalletRequest {
            wallet_id: wallet_id.to_string(),
            wallet_password: wallet_password.to_string(),
            wallet_name: new_name.to_string(),
        };
        self.do_v1_request(req)
    }

    /// Get wallet info
    pub fn get_wallet(&self, wallet_handle: &str) -> Result<GetWalletResponse, Error> {
        let req = GetWalletRequest {
            wallet_handle_token: wallet_handle.to_string(),
        };
        self.do_v1_request(req)
    }

    /// Export the master derivation key from a wallet
    pub fn export_master_derivation_key(
        &self,
        wallet_handle: &str,
        wallet_password: &str,
    ) -> Result<ExportMasterDerivationKeyResponse, Error> {
        let req = ExportMasterDerivationKeyRequest {
            wallet_handle_token: wallet_handle.to_string(),
            wallet_password: wallet_password.to_string(),
        };
        self.do_v1_request(req)
    }

    /// Import an externally generated key into the wallet
    pub fn import_key(
        &self,
        wallet_handle: &str,
        private_key: [u8; 32],
    ) -> Result<ImportKeyResponse, Error> {
        let req = ImportKeyRequest {
            wallet_handle_token: wallet_handle.to_string(),
            private_key,
        };
        self.do_v1_request(req)
    }

    /// Export the Ed25519 seed associated with the passed address
    ///
    /// Note the first 32 bytes of the returned value is the seed, the second 32 bytes is the public key
    pub fn export_key(
        &self,
        wallet_handle: &str,
        wallet_password: &str,
        address: &str,
    ) -> Result<ExportKeyResponse, Error> {
        let req = ExportKeyRequest {
            wallet_handle_token: wallet_handle.to_string(),
            address: address.to_string(),
            wallet_password: wallet_password.to_string(),
        };
        self.do_v1_request(req)
    }

    /// Generates a key and adds it to the wallet, returning the public key
    pub fn generate_key(&self, wallet_handle: &str) -> Result<GenerateKeyResponse, Error> {
        let req = GenerateKeyRequest {
            wallet_handle_token: wallet_handle.to_string(),
            display_mnemonic: false,
        };
        self.do_v1_request(req)
    }

    /// Deletes the key from the wallet
    pub fn delete_key(
        &self,
        wallet_handle: &str,
        wallet_password: &str,
        address: &str,
    ) -> Result<DeleteKeyResponse, Error> {
        let req = DeleteKeyRequest {
            wallet_handle_token: wallet_handle.to_string(),
            wallet_password: wallet_password.to_string(),
            address: address.to_string(),
        };
        self.do_v1_request(req)
    }

    /// List all of the public keys in the wallet
    pub fn list_keys(&self, wallet_handle: &str) -> Result<ListKeysResponse, Error> {
        let req = ListKeysRequest {
            wallet_handle_token: wallet_handle.to_string(),
        };
        self.do_v1_request(req)
    }

    /// Sign a transaction
    pub fn sign_transaction(
        &self,
        wallet_handle: &str,
        wallet_password: &str,
        transaction: &Transaction,
    ) -> Result<SignTransactionResponse, Error> {
        let transaction_bytes = rmp_serde::to_vec_named(transaction)?;
        let req = SignTransactionRequest {
            wallet_handle_token: wallet_handle.to_string(),
            transaction: transaction_bytes,
            wallet_password: wallet_password.to_string(),
        };
        self.do_v1_request(req)
    }

    /// Lists all of the multisig accounts whose preimages this wallet stores
    pub fn list_multisig(&self, wallet_handle: &str) -> Result<ListMultisigResponse, Error> {
        let req = ListMultisigRequest {
            wallet_handle_token: wallet_handle.to_string(),
        };
        self.do_v1_request(req)
    }

    /// Import a multisig account
    pub fn import_multisig(
        &self,
        wallet_handle: &str,
        version: u8,
        threshold: u8,
        pks: &[Ed25519PublicKey],
    ) -> Result<ImportMultisigResponse, Error> {
        let req = ImportMultisigRequest {
            wallet_handle_token: wallet_handle.to_string(),
            multisig_version: version,
            threshold,
            pks: pks.to_vec(),
        };
        self.do_v1_request(req)
    }

    /// Export multisig address metadata
    pub fn export_multisig(
        &self,
        wallet_handle: &str,
        address: &str,
    ) -> Result<ExportMultisigResponse, Error> {
        let req = ExportMultisigRequest {
            wallet_handle_token: wallet_handle.to_string(),
            address: address.to_string(),
        };
        self.do_v1_request(req)
    }

    /// Delete a multisig from the wallet
    pub fn delete_multisig(
        &self,
        wallet_handle: &str,
        wallet_password: &str,
        address: &str,
    ) -> Result<DeleteMultisigResponse, Error> {
        let req = DeleteMultisigRequest {
            wallet_handle_token: wallet_handle.to_string(),
            wallet_password: wallet_password.to_string(),
            address: address.to_string(),
        };
        self.do_v1_request(req)
    }

    /// Start a multisig signature or add a signature to a partially completed multisig signature
    pub fn sign_multisig_transaction(
        &self,
        wallet_handle: &str,
        wallet_password: &str,
        transaction: &Transaction,
        public_key: Ed25519PublicKey,
        partial_multisig: Option<MultisigSignature>,
    ) -> Result<SignMultisigTransactionResponse, Error> {
        let transaction_bytes = rmp_serde::to_vec_named(transaction)?;
        let req = SignMultisigTransactionRequest {
            wallet_handle_token: wallet_handle.to_string(),
            transaction: transaction_bytes,
            public_key,
            partial_multisig,
            wallet_password: wallet_password.to_string(),
        };
        self.do_v1_request(req)
    }

    fn do_v1_request<R>(&self, req: R) -> Result<R::Response, Error>
    where
        R: APIV1Request,
    {
        let response = self
            .http_client
            .request(R::METHOD, &format!("{}/{}", self.address, R::PATH))
            .header(KMD_TOKEN_HEADER, &self.token)
            .header("Accept", "application/json")
            .json(&req)
            .send()?
            .text()?;
        if let Ok(envelope) = serde_json::from_str::<APIV1ResponseEnvelope>(&response) {
            if envelope.error {
                return Err(Error::Api(envelope.message));
            }
        }
        Ok(serde_json::from_str(&response)?)
    }
}

pub mod requests {
    use reqwest::Method;
    use serde::de::DeserializeOwned;
    use serde::Serialize;

    use crate::crypto::MultisigSignature;
    use crate::kmd::responses::*;
    use crate::util::serialize_bytes;
    use crate::{Ed25519PublicKey, MasterDerivationKey};

    pub trait APIV1Request: Serialize {
        type Response: DeserializeOwned;
        const PATH: &'static str;
        const METHOD: Method;
    }

    /// VersionsRequest is the request for `GET /versions`
    #[derive(Serialize)]
    pub struct VersionsRequest;

    /// ListWalletsRequest is the request for `GET /v1/wallets`
    #[derive(Serialize)]
    pub struct ListWalletsRequest;

    #[derive(Serialize)]
    pub struct CreateWalletRequest {
        pub master_derivation_key: MasterDerivationKey,
        pub wallet_driver_name: String,
        pub wallet_name: String,
        pub wallet_password: String,
    }

    /// InitWalletHandleRequest is the request for `POST /v1/wallet/init`
    #[derive(Serialize)]
    pub struct InitWalletHandleRequest {
        pub wallet_id: String,
        pub wallet_password: String,
    }

    /// ReleaseWalletHandleRequest is the request for `POST /v1/wallet/release`
    #[derive(Serialize)]
    pub struct ReleaseWalletHandleRequest {
        pub wallet_handle_token: String,
    }

    /// RenewWalletHandleRequest is the request for `POST /v1/wallet/renew`
    #[derive(Serialize)]
    pub struct RenewWalletHandleRequest {
        pub wallet_handle_token: String,
    }

    /// RenameWalletRequest is the request for `POST /v1/wallet/rename`
    #[derive(Serialize)]
    pub struct RenameWalletRequest {
        pub wallet_id: String,
        pub wallet_password: String,
        pub wallet_name: String,
    }

    /// GetWalletRequest is the request for `POST /v1/wallet/info`
    #[derive(Serialize)]
    pub struct GetWalletRequest {
        pub wallet_handle_token: String,
    }

    /// ExportMasterDerivationKeyRequest is the request for `POST /v1/master-key/export`
    #[derive(Serialize)]
    pub struct ExportMasterDerivationKeyRequest {
        pub wallet_handle_token: String,
        pub wallet_password: String,
    }

    /// ImportKeyRequest is the request for `POST /v1/key/import`
    #[derive(Serialize)]
    pub struct ImportKeyRequest {
        pub wallet_handle_token: String,
        #[serde(serialize_with = "serialize_bytes")]
        pub private_key: [u8; 32],
    }

    /// ExportKeyRequest is the request for `POST /v1/key/export`
    #[derive(Serialize)]
    pub struct ExportKeyRequest {
        pub wallet_handle_token: String,
        pub address: String,
        pub wallet_password: String,
    }

    /// GenerateKeyRequest is the request for `POST /v1/key`
    #[derive(Serialize)]
    pub struct GenerateKeyRequest {
        pub wallet_handle_token: String,
        pub display_mnemonic: bool,
    }

    /// DeleteKeyRequest is the request for `DELETE /v1/key`
    #[derive(Serialize)]
    pub struct DeleteKeyRequest {
        pub wallet_handle_token: String,
        pub address: String,
        pub wallet_password: String,
    }

    /// ListKeysRequest is the request for `POST /v1/key/list`
    #[derive(Serialize)]
    pub struct ListKeysRequest {
        pub wallet_handle_token: String,
    }

    /// SignTransactionRequest is the request for `POST /v1/transaction/sign`
    #[derive(Serialize)]
    pub struct SignTransactionRequest {
        pub wallet_handle_token: String,
        #[serde(serialize_with = "serialize_bytes")]
        pub transaction: Vec<u8>,
        pub wallet_password: String,
    }

    /// ListMultisigRequest is the request for `POST /v1/multisig/list`
    #[derive(Serialize)]
    pub struct ListMultisigRequest {
        pub wallet_handle_token: String,
    }

    /// ImportMultisigRequest is the request for `POST /v1/multisig/import`
    #[derive(Serialize)]
    pub struct ImportMultisigRequest {
        pub wallet_handle_token: String,
        pub multisig_version: u8,
        pub threshold: u8,
        pub pks: Vec<Ed25519PublicKey>,
    }

    /// ExportMultisigRequest is the request for `POST /v1/multisig/export`
    #[derive(Serialize)]
    pub struct ExportMultisigRequest {
        pub wallet_handle_token: String,
        pub address: String,
    }

    /// DeleteMultisigRequest is the request for `DELETE /v1/multisig`
    #[derive(Serialize)]
    pub struct DeleteMultisigRequest {
        pub wallet_handle_token: String,
        pub address: String,
        pub wallet_password: String,
    }

    /// SignMultisigTransactionRequest is the request for `POST /v1/multisig/sign`
    #[derive(Serialize)]
    pub struct SignMultisigTransactionRequest {
        pub wallet_handle_token: String,
        #[serde(serialize_with = "serialize_bytes")]
        pub transaction: Vec<u8>,
        pub public_key: Ed25519PublicKey,
        pub partial_multisig: Option<MultisigSignature>,
        pub wallet_password: String,
    }

    impl APIV1Request for VersionsRequest {
        type Response = VersionsResponse;
        const PATH: &'static str = "versions";
        const METHOD: Method = Method::GET;
    }

    impl APIV1Request for ListWalletsRequest {
        type Response = ListWalletsResponse;
        const PATH: &'static str = "v1/wallets";
        const METHOD: Method = Method::GET;
    }

    impl APIV1Request for CreateWalletRequest {
        type Response = CreateWalletResponse;
        const PATH: &'static str = "v1/wallet";
        const METHOD: Method = Method::POST;
    }

    impl APIV1Request for InitWalletHandleRequest {
        type Response = InitWalletHandleResponse;
        const PATH: &'static str = "v1/wallet/init";
        const METHOD: Method = Method::POST;
    }

    impl APIV1Request for ReleaseWalletHandleRequest {
        type Response = ReleaseWalletHandleResponse;
        const PATH: &'static str = "v1/wallet/release";
        const METHOD: Method = Method::POST;
    }

    impl APIV1Request for RenewWalletHandleRequest {
        type Response = RenewWalletHandleResponse;
        const PATH: &'static str = "v1/wallet/renew";
        const METHOD: Method = Method::POST;
    }

    impl APIV1Request for RenameWalletRequest {
        type Response = RenameWalletResponse;
        const PATH: &'static str = "v1/wallet/rename";
        const METHOD: Method = Method::POST;
    }

    impl APIV1Request for GetWalletRequest {
        type Response = GetWalletResponse;
        const PATH: &'static str = "v1/wallet/info";
        const METHOD: Method = Method::POST;
    }

    impl APIV1Request for ExportMasterDerivationKeyRequest {
        type Response = ExportMasterDerivationKeyResponse;
        const PATH: &'static str = "v1/master-key/export";
        const METHOD: Method = Method::POST;
    }

    impl APIV1Request for ImportKeyRequest {
        type Response = ImportKeyResponse;
        const PATH: &'static str = "v1/key/import";
        const METHOD: Method = Method::POST;
    }

    impl APIV1Request for ExportKeyRequest {
        type Response = ExportKeyResponse;
        const PATH: &'static str = "v1/key/export";
        const METHOD: Method = Method::POST;
    }

    impl APIV1Request for GenerateKeyRequest {
        type Response = GenerateKeyResponse;
        const PATH: &'static str = "v1/key";
        const METHOD: Method = Method::POST;
    }

    impl APIV1Request for DeleteKeyRequest {
        type Response = DeleteKeyResponse;
        const PATH: &'static str = "v1/key";
        const METHOD: Method = Method::DELETE;
    }

    impl APIV1Request for ListKeysRequest {
        type Response = ListKeysResponse;
        const PATH: &'static str = "v1/key/list";
        const METHOD: Method = Method::POST;
    }

    impl APIV1Request for SignTransactionRequest {
        type Response = SignTransactionResponse;
        const PATH: &'static str = "v1/transaction/sign";
        const METHOD: Method = Method::POST;
    }

    impl APIV1Request for ListMultisigRequest {
        type Response = ListMultisigResponse;
        const PATH: &'static str = "v1/multisig/list";
        const METHOD: Method = Method::POST;
    }

    impl APIV1Request for ImportMultisigRequest {
        type Response = ImportMultisigResponse;
        const PATH: &'static str = "v1/multisig/import";
        const METHOD: Method = Method::POST;
    }

    impl APIV1Request for ExportMultisigRequest {
        type Response = ExportMultisigResponse;
        const PATH: &'static str = "v1/multisig/export";
        const METHOD: Method = Method::POST;
    }

    impl APIV1Request for DeleteMultisigRequest {
        type Response = DeleteMultisigResponse;
        const PATH: &'static str = "v1/multisig";
        const METHOD: Method = Method::DELETE;
    }

    impl APIV1Request for SignMultisigTransactionRequest {
        type Response = SignMultisigTransactionResponse;
        const PATH: &'static str = "v1/multisig/sign";
        const METHOD: Method = Method::POST;
    }
}

pub mod responses {
    use data_encoding::BASE64;
    use serde::{Deserialize, Deserializer};

    use crate::util::{deserialize_bytes, deserialize_bytes64, deserialize_mdk};

    use crate::{Ed25519PublicKey, MasterDerivationKey};

    use crate::kmd::{APIV1Wallet, APIV1WalletHandle};

    #[derive(Debug, Deserialize)]
    pub struct APIV1ResponseEnvelope {
        pub error: bool,
        pub message: String,
    }

    /// VersionsResponse is the response to `GET /versions`
    #[derive(Debug, Deserialize)]
    pub struct VersionsResponse {
        #[serde(default)]
        pub versions: Vec<String>,
    }

    /// ListWalletsResponse is the response to `GET /v1/wallets`
    #[derive(Debug, Deserialize)]
    pub struct ListWalletsResponse {
        #[serde(default)]
        pub wallets: Vec<APIV1Wallet>,
    }

    #[derive(Debug, Deserialize)]
    pub struct CreateWalletResponse {
        pub wallet: APIV1Wallet,
    }

    /// InitWalletHandleResponse is the response to `POST /v1/wallet/init`
    #[derive(Debug, Deserialize)]
    pub struct InitWalletHandleResponse {
        pub wallet_handle_token: String,
    }

    /// ReleaseWalletHandleResponse is the response to `POST /v1/wallet/release`
    #[derive(Debug, Deserialize)]
    pub struct ReleaseWalletHandleResponse {}

    /// RenewWalletHandleResponse is the response to `POST /v1/wallet/renew`
    #[derive(Debug, Deserialize)]
    pub struct RenewWalletHandleResponse {
        pub wallet_handle: APIV1WalletHandle,
    }

    /// RenameWalletResponse is the response to `POST /v1/wallet/rename`
    #[derive(Debug, Deserialize)]
    pub struct RenameWalletResponse {
        pub wallet: APIV1Wallet,
    }

    /// GetWalletResponse is the response to `POST /v1/wallet/info`
    #[derive(Debug, Deserialize)]
    pub struct GetWalletResponse {
        pub wallet_handle: APIV1WalletHandle,
    }

    /// ExportMasterDerivationKeyResponse is the response to `POST /v1/master-key/export`
    #[derive(Debug, Deserialize)]
    pub struct ExportMasterDerivationKeyResponse {
        #[serde(deserialize_with = "deserialize_mdk")]
        pub master_derivation_key: MasterDerivationKey,
    }

    /// ImportKeyResponse is the response to `POST /v1/key/import`
    #[derive(Debug, Deserialize)]
    pub struct ImportKeyResponse {
        pub address: String,
    }

    /// ExportKeyResponse is the response to `POST /v1/key/export`
    #[derive(Deserialize)]
    pub struct ExportKeyResponse {
        #[serde(deserialize_with = "deserialize_bytes64")]
        pub private_key: [u8; 64],
    }

    /// GenerateKeyResponse is the response to `POST /v1/key`
    #[derive(Debug, Deserialize)]
    pub struct GenerateKeyResponse {
        pub address: String,
    }

    /// DeleteKeyResponse is the response to `DELETE /v1/key`
    #[derive(Debug, Deserialize)]
    pub struct DeleteKeyResponse {}

    /// ListKeysResponse is the response to `POST /v1/key/list`
    #[derive(Debug, Deserialize)]
    pub struct ListKeysResponse {
        pub addresses: Vec<String>,
    }

    /// SignTransactionResponse is the response to `POST /v1/transaction/sign`
    #[derive(Debug, Deserialize)]
    pub struct SignTransactionResponse {
        #[serde(deserialize_with = "deserialize_bytes")]
        pub signed_transaction: Vec<u8>,
    }

    /// ListMultisigResponse is the response to `POST /v1/multisig/list`
    #[derive(Debug, Deserialize)]
    pub struct ListMultisigResponse {
        #[serde(default)]
        pub addresses: Vec<String>,
    }

    /// ImportMultisigResponse is the response to `POST /v1/multisig/import`
    #[derive(Debug, Deserialize)]
    pub struct ImportMultisigResponse {
        pub address: String,
    }

    /// ExportMultisigResponse is the response to `POST /v1/multisig/export`
    #[derive(Debug, Deserialize)]
    pub struct ExportMultisigResponse {
        pub multisig_version: u8,
        pub threshold: u8,
        #[serde(deserialize_with = "deserialize_public_keys")]
        pub pks: Vec<Ed25519PublicKey>,
    }

    /// DeleteMultisigResponse is the response to POST /v1/multisig/delete`
    #[derive(Debug, Deserialize)]
    pub struct DeleteMultisigResponse {}

    /// SignMultisigTransactionResponse is the response to `POST /v1/multisig/sign`
    #[derive(Debug, Deserialize)]
    pub struct SignMultisigTransactionResponse {
        #[serde(deserialize_with = "deserialize_bytes")]
        pub multisig: Vec<u8>,
    }

    fn deserialize_public_keys<'de, D>(deserializer: D) -> Result<Vec<Ed25519PublicKey>, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        <Vec<String>>::deserialize(deserializer)?
            .iter()
            .map(|string| {
                let mut decoded = [0; 32];
                let bytes = BASE64.decode(string.as_bytes()).map_err(D::Error::custom)?;
                decoded.copy_from_slice(&bytes);
                Ok(Ed25519PublicKey(decoded))
            })
            .collect()
    }
}

#[derive(Debug, Deserialize)]
pub struct APIV1Wallet {
    pub id: String,
    pub name: String,
    pub driver_name: String,
    pub driver_version: u32,
    pub mnemonic_ux: bool,
    pub supported_txs: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct APIV1WalletHandle {
    pub wallet: APIV1Wallet,
    pub expires_seconds: i64,
}
