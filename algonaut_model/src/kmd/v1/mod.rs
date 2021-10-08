use algonaut_core::MultisigSignature;
use algonaut_crypto::{
    deserialize_mdk, deserialize_public_keys, Ed25519PublicKey, MasterDerivationKey,
};
use algonaut_encoding::{deserialize_bytes, deserialize_bytes64, serialize_bytes};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

/// VersionsRequest is the request for `GET /versions`
#[derive(Serialize, Clone)]
pub struct VersionsRequest;

/// VersionsResponse is the response to `GET /versions`
#[derive(Debug, Deserialize, Clone)]
pub struct VersionsResponse {
    #[serde(default)]
    pub versions: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiV1Wallet {
    pub driver_name: String,
    pub driver_version: u32,
    pub id: String,
    pub mnemonic_ux: bool,
    pub name: String,
    pub supported_txs: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiV1WalletHandle {
    pub wallet: ApiV1Wallet,
    pub expires_seconds: i64,
}

/// ListWalletsRequest is the request for `GET /v1/wallets`
#[derive(Serialize, Clone)]
pub struct ListWalletsRequest;

/// ListWalletsResponse is the response to `GET /v1/wallets`
#[derive(Debug, Deserialize, Clone)]
pub struct ListWalletsResponse {
    #[serde(default)]
    pub wallets: Vec<ApiV1Wallet>,
}

#[derive(Serialize, Clone)]
pub struct CreateWalletRequest {
    pub master_derivation_key: MasterDerivationKey,
    pub wallet_driver_name: String,
    pub wallet_name: String,
    pub wallet_password: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateWalletResponse {
    pub wallet: ApiV1Wallet,
}

/// InitWalletHandleRequest is the request for `POST /v1/wallet/init`
#[derive(Serialize, Clone)]
pub struct InitWalletHandleRequest {
    pub wallet_id: String,
    pub wallet_password: String,
}

/// InitWalletHandleResponse is the response to `POST /v1/wallet/init`
#[derive(Debug, Deserialize, Clone)]
pub struct InitWalletHandleResponse {
    pub wallet_handle_token: String,
}

/// ReleaseWalletHandleRequest is the request for `POST /v1/wallet/release`
#[derive(Serialize, Clone)]
pub struct ReleaseWalletHandleRequest {
    pub wallet_handle_token: String,
}

/// ReleaseWalletHandleResponse is the response to `POST /v1/wallet/release`
#[derive(Debug, Deserialize, Clone)]
pub struct ReleaseWalletHandleResponse {}

/// RenewWalletHandleRequest is the request for `POST /v1/wallet/renew`
#[derive(Serialize, Clone)]
pub struct RenewWalletHandleRequest {
    pub wallet_handle_token: String,
}

/// RenewWalletHandleResponse is the response to `POST /v1/wallet/renew`
#[derive(Debug, Deserialize, Clone)]
pub struct RenewWalletHandleResponse {
    pub wallet_handle: ApiV1WalletHandle,
}

/// RenameWalletRequest is the request for `POST /v1/wallet/rename`
#[derive(Serialize, Clone)]
pub struct RenameWalletRequest {
    pub wallet_id: String,
    pub wallet_password: String,
    pub wallet_name: String,
}

/// RenameWalletResponse is the response to `POST /v1/wallet/rename`
#[derive(Debug, Deserialize, Clone)]
pub struct RenameWalletResponse {
    pub wallet: ApiV1Wallet,
}

/// GetWalletRequest is the request for `POST /v1/wallet/info`
#[derive(Serialize, Clone)]
pub struct GetWalletInfoRequest {
    pub wallet_handle_token: String,
}

/// GetWalletResponse is the response to `POST /v1/wallet/info`
#[derive(Debug, Deserialize, Clone)]
pub struct GetWalletInfoResponse {
    pub wallet_handle: ApiV1WalletHandle,
}

/// ExportMasterDerivationKeyRequest is the request for `POST /v1/master-key/export`
#[derive(Serialize, Clone)]
pub struct ExportMasterDerivationKeyRequest {
    pub wallet_handle_token: String,
    pub wallet_password: String,
}

/// ExportMasterDerivationKeyResponse is the response to `POST /v1/master-key/export`
#[derive(Debug, Deserialize, Clone)]
pub struct ExportMasterDerivationKeyResponse {
    #[serde(deserialize_with = "deserialize_mdk")]
    pub master_derivation_key: MasterDerivationKey,
}

/// ImportKeyRequest is the request for `POST /v1/key/import`
#[derive(Serialize, Clone)]
pub struct ImportKeyRequest {
    pub wallet_handle_token: String,
    #[serde(serialize_with = "serialize_bytes")]
    pub private_key: [u8; 32],
}

/// ImportKeyResponse is the response to `POST /v1/key/import`
#[derive(Debug, Deserialize, Clone)]
pub struct ImportKeyResponse {
    pub address: String,
}

/// ExportKeyRequest is the request for `POST /v1/key/export`
#[derive(Serialize, Clone)]
pub struct ExportKeyRequest {
    pub wallet_handle_token: String,
    pub address: String,
    pub wallet_password: String,
}

/// ExportKeyResponse is the response to `POST /v1/key/export`
#[derive(Deserialize, Clone)]
pub struct ExportKeyResponse {
    #[serde(deserialize_with = "deserialize_bytes64")]
    pub private_key: [u8; 64],
}

impl Debug for ExportKeyResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExportKeyResponse")
            .field("private_key", &self.private_key.to_vec())
            .finish()
    }
}

/// GenerateKeyRequest is the request for `POST /v1/key`
#[derive(Serialize, Clone)]
pub struct GenerateKeyRequest {
    pub wallet_handle_token: String,
    pub display_mnemonic: bool,
}

/// GenerateKeyResponse is the response to `POST /v1/key`
#[derive(Debug, Deserialize, Clone)]
pub struct GenerateKeyResponse {
    pub address: String,
}

/// DeleteKeyRequest is the request for `DELETE /v1/key`
#[derive(Serialize, Clone)]
pub struct DeleteKeyRequest {
    pub wallet_handle_token: String,
    pub address: String,
    pub wallet_password: String,
}

/// DeleteKeyResponse is the response to `DELETE /v1/key`
#[derive(Debug, Deserialize, Clone)]
pub struct DeleteKeyResponse {}

/// ListKeysRequest is the request for `POST /v1/key/list`
#[derive(Serialize, Clone)]
pub struct ListKeysRequest {
    pub wallet_handle_token: String,
}

/// ListKeysResponse is the response to `POST /v1/key/list`
#[derive(Debug, Deserialize, Clone)]
pub struct ListKeysResponse {
    #[serde(default)]
    pub addresses: Vec<String>,
}

/// SignTransactionRequest is the request for `POST /v1/transaction/sign`
#[derive(Serialize, Clone)]
pub struct SignTransactionRequest {
    pub wallet_handle_token: String,
    #[serde(serialize_with = "serialize_bytes")]
    pub transaction: Vec<u8>,
    pub wallet_password: String,
}

/// SignTransactionResponse is the response to `POST /v1/transaction/sign`
#[derive(Debug, Deserialize, Clone)]
pub struct SignTransactionResponse {
    #[serde(deserialize_with = "deserialize_bytes")]
    pub signed_transaction: Vec<u8>,
}

/// ListMultisigRequest is the request for `POST /v1/multisig/list`
#[derive(Serialize, Clone)]
pub struct ListMultisigRequest {
    pub wallet_handle_token: String,
}

/// ListMultisigResponse is the response to `POST /v1/multisig/list`
#[derive(Debug, Deserialize, Clone)]
pub struct ListMultisigResponse {
    #[serde(default)]
    pub addresses: Vec<String>,
}

/// ImportMultisigRequest is the request for `POST /v1/multisig/import`
#[derive(Serialize, Clone)]
pub struct ImportMultisigRequest {
    pub wallet_handle_token: String,
    pub multisig_version: u8,
    pub threshold: u8,
    pub pks: Vec<Ed25519PublicKey>,
}

/// ImportMultisigResponse is the response to `POST /v1/multisig/import`
#[derive(Debug, Deserialize, Clone)]
pub struct ImportMultisigResponse {
    pub address: String,
}

/// ExportMultisigRequest is the request for `POST /v1/multisig/export`
#[derive(Serialize, Clone)]
pub struct ExportMultisigRequest {
    pub wallet_handle_token: String,
    pub address: String,
}

/// ExportMultisigResponse is the response to `POST /v1/multisig/export`
#[derive(Debug, Deserialize, Clone)]
pub struct ExportMultisigResponse {
    pub multisig_version: u8,
    pub threshold: u8,
    #[serde(deserialize_with = "deserialize_public_keys")]
    pub pks: Vec<Ed25519PublicKey>,
}

/// DeleteMultisigRequest is the request for `DELETE /v1/multisig`
#[derive(Serialize, Clone)]
pub struct DeleteMultisigRequest {
    pub wallet_handle_token: String,
    pub address: String,
    pub wallet_password: String,
}

/// DeleteMultisigResponse is the response to POST /v1/multisig/delete`
#[derive(Debug, Deserialize, Clone)]
pub struct DeleteMultisigResponse {}

/// SignMultisigTransactionRequest is the request for `POST /v1/multisig/sign`
#[derive(Serialize, Clone)]
pub struct SignMultisigTransactionRequest {
    pub wallet_handle_token: String,
    #[serde(serialize_with = "serialize_bytes")]
    pub transaction: Vec<u8>,
    pub public_key: Ed25519PublicKey,
    pub partial_multisig: Option<MultisigSignature>,
    pub wallet_password: String,
}

/// SignMultisigTransactionResponse is the response to `POST /v1/multisig/sign`
#[derive(Debug, Deserialize, Clone)]
pub struct SignMultisigTransactionResponse {
    #[serde(deserialize_with = "deserialize_bytes")]
    pub multisig: Vec<u8>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiV1ResponseEnvelope {
    pub error: bool,
    pub message: String,
}
