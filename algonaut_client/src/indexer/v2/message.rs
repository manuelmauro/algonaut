use algonaut_core::{MicroAlgos, Round};
use algonaut_crypto::{deserialize_hash, HashDigest};
use algonaut_encoding::deserialize_bytes;
use serde::{Deserialize, Serialize};

///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QueryAccount {
    /// Application ID.
    #[serde(rename = "application-id")]
    pub application_id: Option<u64>,

    /// Asset ID.
    #[serde(rename = "asset-id")]
    pub asset_id: Option<u64>,

    /// Include accounts configured to use this spending key.
    #[serde(rename = "auth-addr")]
    pub auth_addr: Option<String>,

    /// Results should have an amount greater than this value. MicroAlgos are the default currency
    /// unless an asset-id is provided, in which case the asset will be used.
    #[serde(rename = "currency-greater-than")]
    pub currency_greater_than: Option<u64>,

    /// Results should have an amount less than this value. MicroAlgos are the default currency
    /// unless an asset-id is provided, in which case the asset will be used.
    #[serde(rename = "currency-less-than")]
    pub currency_less_than: Option<u64>,

    /// Maximum number of results to return.
    pub limit: Option<u64>,

    /// The next page of results. Use the next token provided by the previous results.
    pub next: Option<String>,

    /// Include results for the specified round. For performance reasons, this parameter may be
    /// disabled on some configurations.
    pub round: Option<Round>,
}

///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AccountResponse {
    /// Accounts.
    pub accounts: Vec<Account>,

    /// Round at which the results were computed..
    #[serde(rename = "current-round")]
    pub current_round: u64,

    /// Used for pagination, when making another request provide this token with the next
    /// parameter.
    #[serde(rename = "next-token")]
    pub next_token: Option<String>,
}

///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QueryRound {
    pub round: Round,
}

///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AccountIdResponse {
    /// Account.
    pub account: Account,

    /// Round at which the results were computed.
    #[serde(rename = "current-round")]
    pub current_round: u64,
}

/// Query account transactions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QueryAccountTransaction {
    /// Include results after the given time. Must be an RFC 3339 formatted string.
    #[serde(rename = "after-time", skip_serializing_if = "Option::is_none")]
    pub after_time: Option<String>,

    /// Asset ID
    #[serde(rename = "asset-id", skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<u64>,

    /// Include results before the given time. Must be an RFC 3339 formatted string.
    #[serde(rename = "before-time", skip_serializing_if = "Option::is_none")]
    pub before_time: Option<String>,

    /// Results should have an amount greater than this value. MicroAlgos are the default currency
    /// unless an asset-id is provided, in which case the asset will be used.
    #[serde(
        rename = "currency-greater-than",
        skip_serializing_if = "Option::is_none"
    )]
    pub currency_greater_than: Option<u64>,

    /// Results should have an amount less than this value. MicroAlgos are the default currency
    /// unless an asset-id is provided, in which case the asset will be used.
    #[serde(rename = "currency-less-than", skip_serializing_if = "Option::is_none")]
    pub currency_less_than: Option<u64>,

    /// Maximum number of results to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,

    /// Include results at or before the specified max-round.
    #[serde(rename = "max-round", skip_serializing_if = "Option::is_none")]
    pub max_round: Option<Round>,

    /// Include results at or after the specified min-round.
    #[serde(rename = "min-round", skip_serializing_if = "Option::is_none")]
    pub min_round: Option<Round>,

    /// The next page of results. Use the next token provided by the previous results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,

    /// Specifies a prefix which must be contained in the note field.
    #[serde(rename = "note-prefix", skip_serializing_if = "Option::is_none")]
    pub note_prefix: Option<String>,

    /// Include results which include the rekey-to field.
    #[serde(rename = "rekey-to", skip_serializing_if = "Option::is_none")]
    pub rekey_to: Option<bool>,

    /// Include results for the specified round.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub round: Option<u64>,

    /// SigType filters just results using the specified type of signature:
    /// * sig - Standard
    /// * msig - MultiSig
    /// * lsig - LogicSig
    #[serde(rename = "sig-type", skip_serializing_if = "Option::is_none")]
    pub sig_type: Option<SignatureType>,

    /// Filters results according to the type of transactions.
    #[serde(rename = "tx-type", skip_serializing_if = "Option::is_none")]
    pub tx_type: Option<TransactionType>,

    /// Lookup the specific transaction by ID.
    #[serde(rename = "tx-type", skip_serializing_if = "Option::is_none")]
    pub txid: Option<String>,
}

/// Resonse to account transactions' endpoint.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AccountTransactionResponse {
    /// Round at which the results were computed.
    #[serde(rename = "current-round")]
    pub current_round: u64,

    /// Used for pagination, when making another request provide this token with the next parameter.
    #[serde(rename = "next-token", skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,

    /// Transaction list.
    #[serde(rename = "transactions")]
    pub transactions: Vec<Transaction>,
}

/// Query applications.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QueryApplications {
    /// Application ID.
    #[serde(rename = "application-id", skip_serializing_if = "Option::is_none")]
    pub application_id: Option<u64>,

    /// Maximum number of results to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,

    /// The next page of results. Use the next token provided by the previous results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
}

/// Response for applications/ endpoint.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ApplicationResponse {
    #[serde(rename = "applications")]
    pub applications: Vec<Application>,

    /// Round at which the results were computed.
    #[serde(rename = "current-round")]
    pub current_round: i32,

    /// Used for pagination, when making another request provide this token with the next parameter.
    #[serde(rename = "next-token", skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

/// Response for applications/id endpoint.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ApplicationInfoResponse {
    #[serde(rename = "application", skip_serializing_if = "Option::is_none")]
    pub application: Option<Box<Application>>,

    /// Round at which the results were computed.
    #[serde(rename = "current-round")]
    pub current_round: i32,
}

/// Query assets.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QueryAssets {
    /// Asset ID.
    #[serde(rename = "asset-id", skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<u64>,

    /// Filter just assets with the given creator address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator: Option<String>,

    /// Maximum number of results to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,

    /// Filter just assets with the given name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The next page of results. Use the next token provided by the previous results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,

    /// Filter just assets with the given unit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
}

/// Assets response.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssetResponse {
    #[serde(rename = "assets")]
    pub assets: Vec<Asset>,

    /// Round at which the results were computed.
    #[serde(rename = "current-round")]
    pub current_round: i32,

    /// Used for pagination, when making another request provide this token with the next parameter.
    #[serde(rename = "next-token", skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

/// Assets info response.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssetsInfoResponse {
    #[serde(rename = "asset")]
    pub asset: Box<Asset>,

    /// Round at which the results were computed.
    #[serde(rename = "current-round")]
    pub current_round: i32,
}

/// Query assets.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QueryBalances {
    /// Results should have an amount greater than this value. MicroAlgos are the default currency
    /// unless an asset-id is provided, in which case the asset will be used.
    #[serde(
        rename = "currency-greater-than",
        skip_serializing_if = "Option::is_none"
    )]
    pub currency_greater_than: Option<u64>,

    /// Results should have an amount less than this value. MicroAlgos are the default currency
    /// unless an asset-id is provided, in which case the asset will be used.
    #[serde(rename = "currency-less-than", skip_serializing_if = "Option::is_none")]
    pub currency_less_than: Option<u64>,

    /// Maximum number of results to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,

    /// The next page of results. Use the next token provided by the previous results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,

    /// Include results for the specified round.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub round: Option<u64>,
}

/// Balances response.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BalancesResponse {
    #[serde(rename = "balances")]
    pub balances: Vec<MiniAssetHolding>,

    /// Round at which the results were computed.
    #[serde(rename = "current-round")]
    pub current_round: i32,

    /// Used for pagination, when making another request provide this token with the next parameter.
    #[serde(rename = "next-token", skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

/// Query assets transactions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QueryAssetTransaction {
    /// Only include transactions with this address in one of the transaction fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,

    /// Combine with the address parameter to define what type of address to search for.
    #[serde(rename = "address-role", skip_serializing_if = "Option::is_none")]
    pub address_role: Option<Role>,

    /// Include results after the given time. Must be an RFC 3339 formatted string.
    #[serde(rename = "after-time", skip_serializing_if = "Option::is_none")]
    pub after_time: Option<String>,

    /// Include results before the given time. Must be an RFC 3339 formatted string.
    #[serde(rename = "before-time", skip_serializing_if = "Option::is_none")]
    pub before_time: Option<String>,

    /// Results should have an amount greater than this value. MicroAlgos are the default currency
    /// unless an asset-id is provided, in which case the asset will be used.
    #[serde(
        rename = "currency-greater-than",
        skip_serializing_if = "Option::is_none"
    )]
    pub currency_greater_than: Option<u64>,

    /// Results should have an amount less than this value. MicroAlgos are the default currency
    /// unless an asset-id is provided, in which case the asset will be used.
    #[serde(rename = "currency-less-than", skip_serializing_if = "Option::is_none")]
    pub currency_less_than: Option<u64>,

    /// Combine with address and address-role parameters to define what type of address to search
    /// for. The close to fields are normally treated as a receiver, if you would like to exclude
    /// them set this parameter to true.
    #[serde(rename = "exclude-close-to", skip_serializing_if = "Option::is_none")]
    pub exclude_close_to: Option<bool>,

    /// Maximum number of results to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,

    /// Include results at or before the specified max-round.
    #[serde(rename = "max-round", skip_serializing_if = "Option::is_none")]
    pub max_round: Option<Round>,

    /// Include results at or after the specified min-round.
    #[serde(rename = "min-round", skip_serializing_if = "Option::is_none")]
    pub min_round: Option<Round>,

    /// The next page of results. Use the next token provided by the previous results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,

    /// Specifies a prefix which must be contained in the note field.
    #[serde(rename = "note-prefix", skip_serializing_if = "Option::is_none")]
    pub note_prefix: Option<String>,

    /// Include results which include the rekey-to field.
    #[serde(rename = "rekey-to", skip_serializing_if = "Option::is_none")]
    pub rekey_to: Option<bool>,

    /// Include results for the specified round.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub round: Option<u64>,

    /// SigType filters just results using the specified type of signature:
    /// * sig - Standard
    /// * msig - MultiSig
    /// * lsig - LogicSig
    #[serde(rename = "sig-type", skip_serializing_if = "Option::is_none")]
    pub sig_type: Option<SignatureType>,

    /// Filters results according to the type of transactions.
    #[serde(rename = "tx-type", skip_serializing_if = "Option::is_none")]
    pub tx_type: Option<TransactionType>,

    /// Lookup the specific transaction by ID.
    #[serde(rename = "tx-type", skip_serializing_if = "Option::is_none")]
    pub txid: Option<String>,
}

/// Resonse to asset transactions' endpoint.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssetTransactionResponse {
    /// Round at which the results were computed.
    #[serde(rename = "current-round")]
    pub current_round: u64,

    /// Used for pagination, when making another request provide this token with the next parameter.
    #[serde(rename = "next-token", skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,

    /// Transaction list.
    #[serde(rename = "transactions")]
    pub transactions: Vec<Transaction>,
}

/// Query transactions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QueryTransaction {
    /// Only include transactions with this address in one of the transaction fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,

    /// Combine with the address parameter to define what type of address to search for.
    #[serde(rename = "address-role", skip_serializing_if = "Option::is_none")]
    pub address_role: Option<Role>,

    /// Include results after the given time. Must be an RFC 3339 formatted string.
    #[serde(rename = "after-time", skip_serializing_if = "Option::is_none")]
    pub after_time: Option<String>,

    /// Application ID.
    #[serde(rename = "application-id", skip_serializing_if = "Option::is_none")]
    pub application_id: Option<u64>,

    ///Asset ID of the holding.
    #[serde(rename = "asset-id", skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<u64>,

    /// Include results before the given time. Must be an RFC 3339 formatted string.
    #[serde(rename = "before-time", skip_serializing_if = "Option::is_none")]
    pub before_time: Option<String>,

    /// Results should have an amount greater than this value. MicroAlgos are the default currency
    /// unless an asset-id is provided, in which case the asset will be used.
    #[serde(
        rename = "currency-greater-than",
        skip_serializing_if = "Option::is_none"
    )]
    pub currency_greater_than: Option<u64>,

    /// Results should have an amount less than this value. MicroAlgos are the default currency
    /// unless an asset-id is provided, in which case the asset will be used.
    #[serde(rename = "currency-less-than", skip_serializing_if = "Option::is_none")]
    pub currency_less_than: Option<u64>,

    /// Combine with address and address-role parameters to define what type of address to search
    /// for. The close to fields are normally treated as a receiver, if you would like to exclude
    /// them set this parameter to true.
    #[serde(rename = "exclude-close-to", skip_serializing_if = "Option::is_none")]
    pub exclude_close_to: Option<bool>,

    /// Maximum number of results to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,

    /// Include results at or before the specified max-round.
    #[serde(rename = "max-round", skip_serializing_if = "Option::is_none")]
    pub max_round: Option<Round>,

    /// Include results at or after the specified min-round.
    #[serde(rename = "min-round", skip_serializing_if = "Option::is_none")]
    pub min_round: Option<Round>,

    /// The next page of results. Use the next token provided by the previous results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,

    /// Specifies a prefix which must be contained in the note field.
    #[serde(rename = "note-prefix", skip_serializing_if = "Option::is_none")]
    pub note_prefix: Option<String>,

    /// Include results which include the rekey-to field.
    #[serde(rename = "rekey-to", skip_serializing_if = "Option::is_none")]
    pub rekey_to: Option<bool>,

    /// Include results for the specified round.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub round: Option<u64>,

    /// SigType filters just results using the specified type of signature:
    /// * sig - Standard
    /// * msig - MultiSig
    /// * lsig - LogicSig
    #[serde(rename = "sig-type", skip_serializing_if = "Option::is_none")]
    pub sig_type: Option<SignatureType>,

    /// Filters results according to the type of transactions.
    #[serde(rename = "tx-type", skip_serializing_if = "Option::is_none")]
    pub tx_type: Option<TransactionType>,

    /// Lookup the specific transaction by ID.
    #[serde(rename = "tx-type", skip_serializing_if = "Option::is_none")]
    pub txid: Option<String>,
}

/// Response to transactions/ endpoint.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransactionResponse {
    /// Round at which the results were computed.
    #[serde(rename = "current-round")]
    pub current_round: i32,

    /// Used for pagination, when making another request provide this token with the next parameter.
    #[serde(rename = "next-token")]
    pub next_token: Option<String>,

    #[serde(rename = "transactions")]
    pub transactions: Vec<Transaction>,
}

/// Response to transaction/id endpoint.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransactionInfoResponse {
    /// Round at which the results were computed.
    #[serde(rename = "current-round")]
    pub current_round: i32,

    #[serde(rename = "transaction")]
    pub transaction: Transaction,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Account {
    /// The account public key.
    pub address: String,

    /// `algo` total number of MicroAlgos in the account.
    pub amount: u64,

    /// specifies the amount of MicroAlgos in the account, without the pending rewards.
    #[serde(rename = "amount-without-pending-rewards")]
    pub amount_without_pending_rewards: u64,

    /// `appl` applications local data stored in this account.
    ///
    /// Note the raw object uses map(int) -> AppLocalState for this type.
    #[serde(rename = "apps-local-state")]
    pub apps_local_state: Option<Vec<ApplicationLocalState>>,

    /// `tsch` stores the sum of all of the local schemas and global schemas in this account.
    ///
    /// Note: the raw account uses StateSchema for this type.
    #[serde(rename = "apps-total-schema")]
    pub apps_total_schema: Option<ApplicationStateSchema>,

    /// `asset` assets held by this account.
    /// Note the raw object uses map(int) -> AssetHolding for this type.
    pub assets: Option<Vec<AssetHolding>>,

    /// `spend` the address against which signing should be checked. If empty, the address of the
    /// current account is used. This field can be updated in any transaction by setting the
    /// RekeyTo field.
    #[serde(rename = "auth-addr")]
    pub auth_addr: Option<String>,

    /// Round during which this account was most recently closed.
    #[serde(rename = "closed-at-round")]
    pub closed_at_round: Option<u64>,

    /// `appp` parameters of applications created by this account including app global data.
    ///
    /// Note: the raw account uses map(int) -> AppParams for this type.
    #[serde(rename = "created-apps")]
    pub created_apps: Option<Vec<Application>>,

    /// `apar` parameters of assets created by this account.
    ///
    /// Note: the raw account uses map(int) -> Asset for this type.
    #[serde(rename = "created-assets")]
    pub created_assets: Option<Vec<Asset>>,

    /// Round during which this account first appeared in a transaction.
    #[serde(rename = "created-at-round")]
    pub created_at_round: Option<Round>,

    /// Whether or not this account is currently closed.
    pub deleted: Option<bool>,

    /// Participation.
    pub participation: Option<AccountParticipation>,

    /// Amount of MicroAlgos of pending rewards in this account.
    #[serde(rename = "pending-rewards")]
    pub pending_rewards: MicroAlgos,

    /// `ebase` used as part of the rewards computation. Only applicable to accounts which
    /// are participating.
    #[serde(rename = "reward-base")]
    pub reward_base: Option<u64>,

    /// `ern` total rewards of MicroAlgos the account has received, including pending rewards.
    pub rewards: MicroAlgos,

    /// The round for which this information is relevant.
    pub round: Round,

    /// Indicates what type of signature is used by this account, must be one of:
    /// * sig
    /// * msig
    /// * lsig
    #[serde(rename = "sig-type")]
    pub sig_type: Option<SignatureType>,

    /// `onl` delegation status of the account's MicroAlgos
    /// * Offline - indicates that the associated account is delegated.
    /// * Online - indicates that the associated account used as part of the delegation pool.
    /// * NotParticipating - indicates that the associated account is neither a delegator nor a delegate.
    pub status: String,
}

/// Signature types.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SignatureType {
    #[serde(rename = "sig")]
    Sig,
    #[serde(rename = "msig")]
    MultiSig,
    #[serde(rename = "lsig")]
    LSig,
}

/// AccountParticipation describes the parameters used by this account in consensus protocol.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AccountParticipation {
    /// `sel` Selection public key (if any) currently registered for this round.
    /// Pattern : "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==\|[A-Za-z0-9+/]{3}=)?$"
    #[serde(
        rename = "selection-participation-key",
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "deserialize_bytes"
    )]
    pub selection_participation_key: Vec<u8>,

    /// `voteFst` First round for which this participation is valid.
    #[serde(rename = "vote-first-valid")]
    pub vote_first_valid: u64,

    /// `voteKD` Number of subkeys in each batch of participation keys.
    #[serde(rename = "vote-key-dilution")]
    pub vote_key_dilution: u64,

    /// `voteLst` Last round for which this participation is valid.
    #[serde(rename = "vote-last-valid")]
    pub vote_last_valid: u64,

    /// `vote` root participation public key (if any) currently registered for this round.
    /// Pattern : "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==\|[A-Za-z0-9+/]{3}=)?$"
    #[serde(
        rename = "vote-participation-key",
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "deserialize_bytes"
    )]
    pub vote_participation_key: Vec<u8>,
}

/// Application state delta.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AccountStateDelta {
    /// Address
    pub address: String,

    /// Delta
    pub delta: StateDelta,
}

/// Application index and its parameters
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Application {
    /// Round when this application was created.
    #[serde(rename = "created-at-round")]
    pub created_at_round: Option<bool>,

    /// Whether or not this application is currently deleted.
    #[serde(rename = "deleted")]
    pub deleted: Option<bool>,

    /// Round when this application was deleted.
    #[serde(rename = "deleted-at-round")]
    pub deleted_at_round: Option<Round>,

    /// `appidx` application index.
    pub id: u64,

    /// `appparams` application parameters.
    pub params: ApplicationParams,
}

/// Stores local state associated with an application.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ApplicationLocalState {
    /// Round when account closed out of the application.
    #[serde(rename = "closed-out-at-round")]
    pub closed_out_at_round: Option<Round>,

    /// Whether or not the application local state is currently deleted from its account.
    pub deleted: Option<bool>,

    /// The application which this local state is for.
    pub id: u64,

    /// `tkv` storage.
    #[serde(rename = "key-value")]
    pub key_value: TealKeyValueStore,

    /// Round when the account opted into the application.
    #[serde(rename = "opted-in-at-round")]
    pub opted_in_at_round: Option<Round>,

    /// `hsch` schema.
    #[serde(rename = "key-value")]
    pub schema: ApplicationStateSchema,
}

/// Stores the global information associated with an application.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ApplicationParams {
    /// `approv` approval program.
    /// Pattern : "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==\|[A-Za-z0-9+/]{3}=)?$"
    #[serde(
        rename = "approval-program",
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "deserialize_bytes"
    )]
    pub approval_program: Vec<u8>,

    /// `clearp` approval program.
    /// Pattern : "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==\|[A-Za-z0-9+/]{3}=)?$"
    #[serde(
        rename = "clear-state-program",
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "deserialize_bytes"
    )]
    pub clear_state_program: Vec<u8>,

    /// The address that created this application. This is the address where the parameters and
    /// global state for this application can be found.
    pub creator: String,

    /// `gs` global schema
    #[serde(rename = "global-state")]
    pub global_state: TealKeyValueStore,

    /// `lsch` global schema
    #[serde(rename = "global-state-schema")]
    pub global_state_schema: ApplicationStateSchema,

    /// `lsch` local schema
    #[serde(rename = "local-state-schema")]
    pub local_state_schema: ApplicationStateSchema,
}

/// Specifies maximums on the number of each type that may be stored.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ApplicationStateSchema {
    /// `nbs` num of byte slices.
    #[serde(rename = "num-byte-slice")]
    pub num_byte_slice: u64,

    /// `nui` num of uints.
    #[serde(rename = "num-uint")]
    pub num_uint: u64,
}

/// Specifies both the unique identifier and the parameters for an asset
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Asset {
    /// Round during which this asset was created.
    #[serde(rename = "created-at-round")]
    pub created_at_round: Option<Round>,

    /// Whether or not this asset is currently deleted.
    pub deleted: Option<bool>,

    /// Round during which this asset was destroyed.
    #[serde(rename = "destroyed-at-round")]
    pub destroyed_at_round: Option<Round>,

    /// unique asset identifier
    pub index: u64,

    /// Params.
    pub params: AssetParams,
}

/// Describes an asset held by an account.
/// Definition: data/basics/userBalance.go : AssetHolding
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssetHolding {
    /// `a` number of units held.
    pub amount: u64,

    ///Asset ID of the holding.
    #[serde(rename = "asset-id")]
    pub asset_id: u64,

    /// Address that created this asset. This is the address where the parameters for this asset can
    /// be found, and also the address where unwanted asset units can be sent in the worst case.
    pub creator: String,

    /// Whether or not the asset holding is currently deleted from its account.
    pub deleted: Option<bool>,

    /// `f` whether or not the holding is frozen.
    #[serde(rename = "is-frozen")]
    pub is_frozen: bool,

    /// Round during which the account opted into this asset holding.
    #[serde(rename = "opted-in-at-round")]
    pub opted_in_at_round: Option<Round>,

    /// Round during which the account opted out of this asset holding.
    #[serde(rename = "opted-out-at-round")]
    pub opted_out_at_round: Option<Round>,
}

/// AssetParams specifies the parameters for an asset.
/// `apar` when part of an AssetConfig transaction.
/// Definition: data/transactions/asset.go : AssetParams
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssetParams {
    /// `c` Address of account used to clawback holdings of this asset. If empty, clawback is not
    /// permitted.
    pub clawback: String,

    /// The address that created this asset. This is the address where the parameters for this
    /// asset can be found, and also the address where unwanted asset units can be sent in the worst
    /// case.
    pub creator: String,

    /// `dc` The number of digits to use after the decimal point when displaying this asset.
    /// If 0, the asset is not divisible. If 1, the base unit of the asset is in tenths.
    /// If 2, the base unit of the asset is in hundredths, and so on. This value must be
    /// between 0 and 19 (inclusive).
    /// Minimum value : 0
    /// Maximum value : 19
    pub decimals: u64,

    /// `df` Whether holdings of this asset are frozen by default.
    #[serde(rename = "default-frozen")]
    pub default_frozen: bool,

    /// `f` Address of account used to freeze holdings of this asset. If empty, freezing is not
    /// permitted.
    pub freeze: String,

    /// `m` Address of account used to manage the keys of this asset and to destroy it.
    pub manager: String,

    /// `am` A commitment to some unspecified asset metadata. The format of this metadata is up
    /// to the application.
    /// Pattern : "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==\|[A-Za-z0-9+/]{3}=)?$"
    #[serde(
        rename = "metadata-hash",
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "deserialize_bytes"
    )]
    pub metadata_hash: Vec<u8>,

    /// `an` Name of this asset, as supplied by the creator.
    pub name: String,

    /// `r` Address of account holding reserve (non-minted) units of this asset.
    pub reserve: String,

    /// `t` The total number of units of this asset.
    pub total: u64,

    /// `un` Name of a unit of this asset, as supplied by the creator.
    #[serde(rename = "unit-name")]
    pub unit_name: String,

    /// `au` URL where more information about the asset can be retrieved.
    pub url: String,
}

/// Block information.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Block {
    /// `gh` hash to which this block belongs.
    ///
    /// Pattern : "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==\|[A-Za-z0-9+/]{3}=)?$"
    #[serde(rename = "genesis-hash", deserialize_with = "deserialize_hash")]
    pub genesis_hash: HashDigest,

    /// `gen` ID to which this block belongs.
    #[serde(rename = "genesis-id")]
    pub genesis_id: String,

    /// `prev` Previous block hash.
    ///
    /// Pattern : "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==\|[A-Za-z0-9+/]{3}=)?$"
    #[serde(rename = "previous-block-hash")]
    pub previous_block_hash: String,

    /// Block rewards.
    pub rewards: Option<BlockRewards>,

    /// `rnd` Current round on which this block was appended to the chain.
    pub round: Round,

    /// `seed` Sortition seed.
    ///
    /// Pattern : "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==\|[A-Za-z0-9+/]{3}=)?$"
    pub seed: String,

    /// `ts` Block creation timestamp in seconds since epoch.
    pub timestamp: u64,

    /// `txns` list of transactions corresponding to a given round.
    pub transactions: Option<Vec<Transaction>>,

    /// `txn` TransactionsRoot authenticates the set of transactions appearing in the block.
    /// More specifically, it's the root of a merkle tree whose leaves are the block's Txids,
    /// in lexicographic order. For the empty block, it's 0. Note that the TxnRoot does not
    /// authenticate the signatures on the transactions, only the transactions themselves.
    /// Two blocks with the same transactions but in a different order and with different signatures
    /// will have the same TxnRoot.
    ///
    /// Pattern : "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==\|[A-Za-z0-9+/]{3}=)?$"
    #[serde(rename = "transactions-root")]
    pub transactions_root: String,

    /// `tc` TxnCounter counts the number of transactions committed in the ledger, from the time at
    /// which support for this feature was introduced.
    ///
    /// Specifically, TxnCounter is the number of the next transaction that will be committed after
    /// this block. It is 0 when no transactions have ever been committed (since TxnCounter started
    /// being supported).
    #[serde(rename = "txn-counter")]
    pub txn_counter: Option<u64>,

    /// Block upgrade state.
    #[serde(rename = "upgrade-state")]
    pub upgrade_state: Option<BlockUpgradeState>,

    /// Block upgrade vote.
    #[serde(rename = "upgrade-vote")]
    pub upgrade_vote: Option<BlockUpgradeVote>,
}

/// Fields relating to rewards.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlockRewards {
    /// `fees` accepts transaction fees, it can only spend to the incentive pool.
    #[serde(rename = "fee-sink")]
    pub fee_sink: String,

    /// `rwcalr` number of leftover MicroAlgos after the distribution of rewards-rate MicroAlgos
    /// for every reward unit in the next round.
    #[serde(rename = "rewards-calculation-round")]
    pub rewards_calculation_round: MicroAlgos,

    /// `earn` How many rewards, in MicroAlgos, have been distributed to each RewardUnit of
    /// MicroAlgos since genesis.
    #[serde(rename = "rewards-level")]
    pub rewards_level: MicroAlgos,

    /// `rwd` accepts periodic injections from the fee-sink and continually redistributes them as
    /// rewards.
    #[serde(rename = "rewards-pool")]
    pub rewards_pool: String,

    /// `rate` Number of new MicroAlgos added to the participation stake from rewards at the next
    /// round.
    #[serde(rename = "rewards-rate")]
    pub rewards_rate: MicroAlgos,

    /// `frac` Number of leftover MicroAlgos after the distribution of RewardsRate/rewardUnits
    /// MicroAlgos for every reward unit in the next round.
    #[serde(rename = "rewards-residue")]
    pub rewards_residue: MicroAlgos,
}

/// Fields relating to a protocol upgrade.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlockUpgradeState {
    /// `proto` The current protocol version.
    #[serde(rename = "current-protocol")]
    pub current_protocol: String,

    /// `nextproto` The next proposed protocol version.
    #[serde(rename = "next-protocol")]
    pub next_protocol: Option<String>,

    /// `nextyes` Number of blocks which approved the protocol upgrade.
    #[serde(rename = "next-protocol-approvals")]
    pub next_protocol_approvals: Option<u64>,

    /// `nextswitch` Round on which the protocol upgrade will take effect.
    #[serde(rename = "next-protocol-switch-on")]
    pub next_protocol_switch_on: Option<Round>,

    /// `nextbefore` Deadline round for this protocol upgrade (No votes will be consider after
    /// this round).
    #[serde(rename = "next-protocol-vote-before")]
    pub next_protocol_vote_before: Option<Round>,
}

/// Fields relating to voting for a protocol upgrade.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlockUpgradeVote {
    /// `upgradeyes` Indicates a yes vote for the current proposal.
    #[serde(rename = "upgrade-approve")]
    pub upgrade_approve: Option<bool>,

    /// `upgradedelay` Indicates the time between acceptance and execution.
    #[serde(rename = "upgrade-delay")]
    pub upgrade_delay: Option<u64>,

    /// `upgradeprop` Indicates a proposed upgrade.
    #[serde(rename = "upgrade-propose")]
    pub upgrade_propose: Option<String>,
}

/// An error response with optional data field.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ErrorResponse<T> {
    /// Error data.
    pub data: Option<T>,

    /// Error message.
    pub message: String,
}

/// Represents a TEAL value delta.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvalDelta {
    /// `at` delta action.
    pub action: u64,

    /// `bs` bytes value.
    pub bytes: Option<String>,

    /// `ui` uint value.
    pub uint: Option<u64>,
}

/// Key-value pairs for StateDelta.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvalDeltaKeyValue {
    pub key: String,

    pub value: EvalDelta,
}

/// A health check response.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HealthCheck<T> {
    pub data: Option<T>,

    #[serde(rename = "db-available")]
    pub db_available: bool,

    #[serde(rename = "is-migrating")]
    pub is_migrating: bool,

    pub message: String,

    pub round: Round,
}

/// A simplified version of AssetHolding
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MiniAssetHolding {
    ///
    pub address: String,

    /// `a` number of units held.
    pub amount: u64,

    /// Whether or not the asset holding is currently deleted from its account.
    pub deleted: Option<bool>,

    /// `f` whether or not the holding is frozen.
    #[serde(rename = "is-frozen")]
    pub is_frozen: bool,

    /// Round during which the account opted into this asset holding.
    #[serde(rename = "opted-in-at-round")]
    pub opted_in_at_round: Option<Round>,

    /// Round during which the account opted out of this asset holding.
    #[serde(rename = "opted-out-at-round")]
    pub opted_out_at_round: Option<Round>,
}

/// `apan` defines the what additional actions occur with the transaction.
///
/// Valid types:
///   * noop
///   * optin
///   * closeout
///   * clear
///   * update
///   * delete
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum OnCompletion {
    #[serde(rename = "noop")]
    Noop,
    #[serde(rename = "optin")]
    Optin,
    #[serde(rename = "closeout")]
    Closeout,
    #[serde(rename = "clear")]
    Clear,
    #[serde(rename = "update")]
    Update,
    #[serde(rename = "delete")]
    Delete,
}

/// Application state delta.
pub type StateDelta = Vec<EvalDeltaKeyValue>;

/// Represents a `apls` local-state or `apgs` global-state schema. These schemas determine how
/// much storage may be used in a local-state or global-state for an application. The more space
/// used, the larger minimum balance must be maintained in the account holding the data.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StateSchema {
    /// Maximum number of TEAL byte slices that may be stored in the key/value store.
    #[serde(rename = "num-byte-slice")]
    pub num_byte_slice: u64,

    /// Maximum number of TEAL uints that may be stored in the key/value store.
    #[serde(rename = "num-uint")]
    pub num_uint: u64,
}

/// Represents a key-value pair in an application store.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TealKeyValue {
    pub key: String,
    pub value: TealValue,
}

/// Represents a key-value store for use in an application.
pub type TealKeyValueStore = Vec<TealKeyValue>;

/// Represents a TEAL value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TealValue {
    /// `tb` bytes value.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "deserialize_bytes"
    )]
    pub bytes: Vec<u8>,

    /// `tt` value type.
    #[serde(rename = "type")]
    pub value_type: u64,

    /// `ui` uint value.
    pub uint: u64,
}

/// Contains all fields common to all transactions and serves as an envelope to all transactions
/// type..
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    /// Application transaction.
    #[serde(rename = "application-transaction")]
    pub application_transaction: Option<TransactionApplication>,

    /// Asset config transaction.
    #[serde(rename = "asset-config-transaction")]
    pub asset_config_transaction: Option<TransactionAssetConfig>,

    /// Asset config transaction.
    #[serde(rename = "asset-freeze-transaction")]
    pub asset_freeze_transaction: Option<TransactionAssetFreeze>,

    /// Asset transfer transaction.
    #[serde(rename = "asset-transfer-transaction")]
    pub asset_transfer_transaction: Option<TransactionAssetTransfer>,

    /// `sgnr` The address used to sign the transaction. This is used for rekeyed accounts to
    /// indicate that the sender address did not sign the transaction.
    #[serde(rename = "auth-addr")]
    pub auth_addr: Option<String>,

    /// `rc` rewards applied to close-remainder-to account.
    #[serde(rename = "close-rewards")]
    pub close_rewards: Option<MicroAlgos>,

    /// `ca` closing amount for transaction.
    #[serde(rename = "closing-amount")]
    pub closing_amount: Option<MicroAlgos>,

    /// Round when the transaction was confirmed.
    #[serde(rename = "confirmed-round")]
    pub confirmed_round: Option<Round>,

    /// Specifies an application index (ID) if an application was created with this transaction.
    #[serde(rename = "created-application-index")]
    pub created_application_index: Option<u64>,

    /// Specifies an asset index (ID) if an asset was created with this transaction.
    #[serde(rename = "created-asset-index")]
    pub created_asset_index: Option<u64>,

    /// `fee` Transaction fee.
    pub fee: u64,

    /// `fv` First valid round for this transaction.
    #[serde(rename = "first-valid")]
    pub first_valid: u64,

    /// `gh` Hash of genesis block.
    ///
    /// Pattern : "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==\|[A-Za-z0-9+/]{3}=)?$"
    #[serde(rename = "genesis-hash")]
    pub genesis_hash: Option<HashDigest>,

    /// `gen` genesis block ID.
    #[serde(rename = "genesis-id")]
    pub genesis_id: Option<String>,

    /// `gd` Global state key/value changes for the application being executed by this transaction.
    #[serde(rename = "global-state-delta")]
    pub global_state_delta: Option<StateDelta>,

    /// `grp` Base64 encoded byte array of a sha512/256 digest. When present indicates that this
    /// transaction is part of a transaction group and the value is the sha512/256 hash of the
    /// transactions in that group.
    ///
    /// Pattern : "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==\|[A-Za-z0-9+/]{3}=)?$"
    pub group: Option<String>,

    /// Transaction ID
    pub id: String,

    /// Offset into the round where this transaction was confirmed.
    #[serde(rename = "intra-round-offset")]
    pub intra_round_offset: Option<u64>,

    /// Keyreg transaction.
    #[serde(rename = "keyreg-transaction")]
    pub keyreg_transaction: Option<TransactionKeyreg>,

    /// `lv` Last valid round for this transaction.
    #[serde(rename = "last-valid")]
    pub last_valid: Round,

    /// `lx` Base64 encoded 32-byte array. Lease enforces mutual exclusion of transactions.
    /// If this field is nonzero, then once the transaction is confirmed, it acquires the lease
    /// identified by the (Sender, Lease) pair of the transaction until the LastValid round passes.
    /// While this transaction possesses the lease, no other transaction specifying this lease can
    /// be confirmed.
    ///
    /// Pattern : "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==\|[A-Za-z0-9+/]{3}=)?$"
    pub lease: Option<HashDigest>,

    /// `ld` Local state key/value changes for the application being executed by this transaction.
    #[serde(rename = "local-state-delta")]
    pub local_state_delta: Option<Vec<AccountStateDelta>>,

    /// `note` Free form data.
    ///
    /// Pattern : "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==\|[A-Za-z0-9+/]{3}=)?$"
    pub note: Option<String>,

    /// Payment.
    #[serde(rename = "payment-transaction")]
    pub payment_transaction: Option<TransactionPayment>,

    /// `rr` rewards applied to receiver account.
    #[serde(rename = "receiver-rewards")]
    pub receiver_rewards: Option<MicroAlgos>,

    /// `rekey` when included in a valid transaction, the accounts auth addr will be updated with
    /// this value and future signatures must be signed with the key represented by this address.
    #[serde(rename = "rekey-to")]
    pub rekey_to: Option<String>,

    /// Time when the block this transaction is in was confirmed.
    #[serde(rename = "round-time")]
    pub round_time: Option<u64>,

    /// `snd` Sender's address.
    pub sender: String,

    /// `rs` rewards applied to sender account.
    #[serde(rename = "sender-rewards")]
    pub sender_rewards: Option<u64>,

    /// Signature.
    pub signature: TransactionSignature,

    /// `type` Indicates what type of transaction this is. Different types have different fields.
    /// Valid types, and where their fields are stored:
    ///   * `pay` payment-transaction
    ///   * `keyreg` keyreg-transaction
    ///   * `acfg` asset-config-transaction
    ///   * `axfer` asset-transfer-transaction
    ///   * `afrz` asset-freeze-transaction
    ///   * `appl` application-transaction
    #[serde(rename = "tx-type")]
    pub tx_type: TransactionType,
}

/// All the possible types of transactions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TransactionType {
    #[serde(rename = "pay")]
    Payment,
    #[serde(rename = "keyreg")]
    KeyRegistration,
    #[serde(rename = "acfg")]
    AssetConfigurationTransaction,
    #[serde(rename = "axfer")]
    AssetTransferTransaction,
    #[serde(rename = "axfrz")]
    AssetFreezeTransaction,
    #[serde(rename = "appl")]
    ApplicationTransaction,
}

/// Fields for application transactions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransactionApplication {
    /// `apat` List of accounts in addition to the sender that may be accessed from the application's
    /// approval-program and clear-state-program.
    pub accounts: Option<Vec<String>>,

    /// `apaa` transaction specific arguments accessed from the application's approval-program and
    /// clear-state-program.
    #[serde(rename = "application-args")]
    pub application_args: Option<Vec<String>>,

    /// `apid` ID of the application being configured or empty if creating.
    #[serde(rename = "application-id")]
    pub application_id: u64,

    /// `apap` Logic executed for every application transaction, except when on-completion is set
    /// to "clear". It can read and write global state for the application, as well as
    /// account-specific local state. Approval programs may reject the transaction.
    ///
    /// Pattern : "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==\|[A-Za-z0-9+/]{3}=)?$"
    #[serde(rename = "approval-program")]
    pub approval_program: Option<String>,

    /// `apsu` Logic executed for application transactions with on-completion set to "clear".
    /// It can read and write global state for the application, as well as account-specific local
    /// state. Clear state programs cannot reject the transaction.
    ///
    /// Pattern : "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==\|[A-Za-z0-9+/]{3}=)?$"
    #[serde(rename = "clear-state-program")]
    pub clear_state_program: Option<String>,

    /// `apfa` Lists the applications in addition to the application-id whose global states may be
    /// accessed by this application's approval-program and clear-state-program. The access is read-only.
    #[serde(rename = "foreign-apps")]
    pub foreign_apps: Option<Vec<u64>>,

    /// `apas` lists the assets whose parameters may be accessed by this application's ApprovalProgram
    /// and ClearStateProgram. The access is read-only.
    #[serde(rename = "foreign-assets")]
    pub foreign_assets: Option<Vec<u64>>,

    /// Global state schema.
    #[serde(rename = "global-state-schema")]
    pub global_state_schema: Option<StateSchema>,

    /// Local state schema.
    #[serde(rename = "local-state-schema")]
    pub local_state_schema: Option<StateSchema>,

    /// On completion.
    #[serde(rename = "on-completion")]
    pub on_completion: OnCompletion,
}

/// Fields for asset allocation, re-configuration, and destruction.
///
/// A zero value for asset-id indicates asset creation. A zero value for the params indicates asset
/// destruction.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransactionAssetConfig {
    /// `xaid` ID of the asset being configured or empty if creating.
    #[serde(rename = "asset-id")]
    pub asset_id: Option<u64>,

    /// Params.
    pub params: Option<AssetParams>,
}

/// Fields for an asset freeze transaction.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransactionAssetFreeze {
    /// `fadd` Address of the account whose asset is being frozen or thawed.
    pub address: String,

    /// `faid` ID of the asset being frozen or thawed.
    #[serde(rename = "asset-id")]
    pub asset_id: u64,

    /// `afrz` The new freeze status.
    #[serde(rename = "new-freeze-status")]
    pub new_freeze_status: bool,
}

/// Fields for an asset transfer transaction.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransactionAssetTransfer {
    /// `aamt` Amount of asset to transfer. A zero amount transferred to self allocates that asset
    /// in the account's Assets map.
    pub amount: u64,

    /// `xaid` ID of the asset being transferred.
    #[serde(rename = "asset-id")]
    pub asset_id: u64,

    /// Number of assets transfered to the close-to account as part of the transaction.
    #[serde(rename = "close-amount")]
    pub close_amount: Option<u64>,

    /// `aclose` Indicates that the asset should be removed from the account's Assets map,
    /// and specifies where the remaining asset holdings should be transferred. It's always valid
    /// to transfer remaining asset holdings to the creator account.
    #[serde(rename = "close-to")]
    pub close_to: Option<String>,

    /// `arcv` Recipient address of the transfer.
    pub receiver: String,

    /// `asnd` The effective sender during a clawback transactions. If this is not a zero value,
    /// the real transaction sender must be the Clawback address from the AssetParams.
    pub sender: Option<String>,
}

/// Fields for a keyreg transaction.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransactionKeyreg {
    /// `nonpart` Mark the account as participating or non-participating.
    #[serde(rename = "non-participation")]
    pub non_participation: Option<bool>,

    /// `selkey` Public key used with the Verified Random Function (VRF) result during committee
    /// selection.
    ///
    /// Pattern : "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==\|[A-Za-z0-9+/]{3}=)?$"
    #[serde(rename = "selection-participation-key")]
    pub selection_participation_key: Option<String>,

    /// `votefst` First round this participation key is valid.
    #[serde(rename = "vote-first-valid")]
    pub vote_first_valid: Option<u64>,

    /// `votekd` Number of subkeys in each batch of participation keys.
    #[serde(rename = "vote-key-dilution")]
    pub vote_key_dilution: Option<u64>,

    /// `votelst` Last round this participation key is valid.
    #[serde(rename = "vote-key-dilution")]
    pub vote_last_valid: Option<u64>,

    /// `votekey` Participation public key used in key registration transactions.
    ///
    /// Pattern : "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==\|[A-Za-z0-9+/]{3}=)?$"
    #[serde(rename = "vote-participation-key")]
    pub vote_participation_key: Option<String>,
}

/// Fields for a payment transaction.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransactionPayment {
    /// `amt` number of MicroAlgos intended to be transferred.
    pub amount: MicroAlgos,

    /// Number of MicroAlgos that were sent to the close-remainder-to address when closing the
    /// sender account.
    #[serde(rename = "close-amount")]
    pub close_amount: Option<MicroAlgos>,

    /// `close` when set, indicates that the sending account should be closed and all remaining
    /// funds be transferred to this address.
    #[serde(rename = "close-remainder-to")]
    pub close_remainder_to: Option<String>,

    /// `rcv` receiver's address.
    pub receiver: String,
}

/// Validation signature associated with some data. Only one of the signatures should be provided.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransactionSignature {
    /// Logic signature.
    pub logicsig: Option<TransactionSignatureLogicsig>,

    /// Multisignature.
    pub multisig: Option<TransactionSignatureMultisig>,

    /// `sig` Standard ed25519 signature.
    ///
    /// Pattern : "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==\|[A-Za-z0-9+/]{3}=)?$"
    pub sig: Option<String>,
}

/// `lsig` Programatic transaction signature.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransactionSignatureLogicsig {
    /// `arg` Logic arguments, base64 encoded.
    pub args: Option<Vec<String>>,

    /// `l` Program signed by a signature or multi signature, or hashed to be the address of an
    /// account. Base64 encoded TEAL program.
    ///
    /// Pattern : "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==\|[A-Za-z0-9+/]{3}=)?$"
    pub logic: String,

    ///
    #[serde(rename = "multisig-signature")]
    pub multisig_signature: Option<TransactionSignatureMultisig>,

    /// `sig` ed25519 signature.
    ///
    /// Pattern : "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==\|[A-Za-z0-9+/]{3}=)?$"
    pub signature: String,
}

/// `msig` structure holding multiple subsignatures.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransactionSignatureMultisig {
    /// `subsig` holds pairs of public key and signatures.
    pub subsignature: Option<Vec<TransactionSignatureMultisigSubsignature>>,

    /// `thr`
    pub threshold: Option<u64>,

    /// `v`
    pub version: Option<u64>,
}

///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransactionSignatureMultisigSubsignature {
    /// `pk`
    ///
    /// Pattern : "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==\|[A-Za-z0-9+/]{3}=)?$"
    #[serde(rename = "public-key")]
    pub public_key: Option<String>,

    /// `s`
    ///
    /// Pattern : "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==\|[A-Za-z0-9+/]{3}=)?$"
    pub signature: Option<String>,
}

/// Role types.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Role {
    #[serde(rename = "sender")]
    Sender,
    #[serde(rename = "receiver")]
    Receiver,
    #[serde(rename = "freeze-target")]
    FreezeTarget,
}
