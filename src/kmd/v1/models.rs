use crate::crypto::MultisigSignature;
use crate::crypto::{Ed25519PublicKey, MasterDerivationKey};
use crate::kmd::v1::{APIV1Wallet, APIV1WalletHandle};
use crate::serialization::serialize_bytes;
use crate::serialization::{deserialize_bytes, deserialize_bytes64, deserialize_mdk};
use data_encoding::BASE64;
use serde::Serialize;
use serde::{Deserialize, Deserializer};
use std::fmt::{Debug, Formatter};

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
pub struct GetWalletInfoRequest {
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
pub struct GetWalletInfoResponse {
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

impl Debug for ExportKeyResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExportKeyResponse")
            .field("private_key", &self.private_key.to_vec())
            .finish()
    }
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
    #[serde(default)]
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
