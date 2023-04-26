use self::error::IndexerError;
use crate::Error;
use algonaut_indexer::{
    apis::configuration::{ApiKey, Configuration},
    models::{
        Block, HealthCheck, LookupAccountAppLocalStates200Response, LookupAccountAssets200Response,
        LookupAccountById200Response, LookupAccountCreatedApplications200Response,
        LookupAccountCreatedAssets200Response, LookupAccountTransactions200Response,
        LookupApplicationById200Response, LookupApplicationLogsById200Response,
        LookupAssetBalances200Response, LookupAssetById200Response, LookupTransaction200Response,
        SearchForAccounts200Response, SearchForApplicationBoxes200Response,
    },
};

/// Error class wrapping errors from algonaut_indexer
pub(crate) mod error;

#[derive(Debug, Clone)]
pub struct Indexer {
    pub(crate) configuration: Configuration,
}

impl Indexer {
    /// Build a v2 client for Algorand's indexer.
    pub fn new(url: &str, token: &str) -> Result<Self, Error> {
        let conf = Configuration {
            base_path: url.to_owned(),
            user_agent: Some("algonaut".to_owned()),
            client: reqwest::Client::new(),
            basic_auth: None,
            oauth_access_token: None,
            bearer_access_token: None,
            api_key: Some(ApiKey {
                prefix: None,
                key: token.to_owned(),
            }),
        };

        Ok(Self {
            configuration: conf,
        })
    }

    /// Health check.
    pub async fn health(&self) -> Result<HealthCheck, Error> {
        Ok(
            algonaut_indexer::apis::common_api::make_health_check(&self.configuration)
                .await
                .map_err(Into::<IndexerError>::into)?,
        )
    }

    /// Lookup an account's asset holdings, optionally for a specific ID.
    pub async fn lookup_account_app_local_states(
        &self,
        account_id: &str,
        application_id: Option<i32>,
        include_all: Option<bool>,
        limit: Option<i32>,
        next: Option<&str>,
    ) -> Result<LookupAccountAppLocalStates200Response, Error> {
        Ok(
            algonaut_indexer::apis::lookup_api::lookup_account_app_local_states(
                &self.configuration,
                account_id,
                application_id,
                include_all,
                limit,
                next,
            )
            .await
            .map_err(Into::<IndexerError>::into)?,
        )
    }

    /// Lookup an account's asset holdings, optionally for a specific ID.
    pub async fn lookup_account_assets(
        &self,
        account_id: &str,
        asset_id: Option<i32>,
        include_all: Option<bool>,
        limit: Option<i32>,
        next: Option<&str>,
    ) -> Result<LookupAccountAssets200Response, Error> {
        Ok(algonaut_indexer::apis::lookup_api::lookup_account_assets(
            &self.configuration,
            account_id,
            asset_id,
            include_all,
            limit,
            next,
        )
        .await
        .map_err(Into::<IndexerError>::into)?)
    }

    /// Lookup account information.
    pub async fn lookup_account_by_id(
        &self,
        account_id: &str,
        round: Option<i32>,
        include_all: Option<bool>,
        exclude: Option<Vec<String>>,
    ) -> Result<LookupAccountById200Response, Error> {
        Ok(algonaut_indexer::apis::lookup_api::lookup_account_by_id(
            &self.configuration,
            account_id,
            round,
            include_all,
            exclude,
        )
        .await
        .map_err(Into::<IndexerError>::into)?)
    }

    /// Lookup an account's created application parameters, optionally for a specific ID.
    pub async fn lookup_account_created_applications(
        &self,
        account_id: &str,
        application_id: Option<i32>,
        include_all: Option<bool>,
        limit: Option<i32>,
        next: Option<&str>,
    ) -> Result<LookupAccountCreatedApplications200Response, Error> {
        Ok(
            algonaut_indexer::apis::lookup_api::lookup_account_created_applications(
                &self.configuration,
                account_id,
                application_id,
                include_all,
                limit,
                next,
            )
            .await
            .map_err(Into::<IndexerError>::into)?,
        )
    }

    /// Lookup an account's created asset parameters, optionally for a specific ID.
    pub async fn lookup_account_created_assets(
        &self,
        account_id: &str,
        asset_id: Option<i32>,
        include_all: Option<bool>,
        limit: Option<i32>,
        next: Option<&str>,
    ) -> Result<LookupAccountCreatedAssets200Response, Error> {
        Ok(
            algonaut_indexer::apis::lookup_api::lookup_account_created_assets(
                &self.configuration,
                account_id,
                asset_id,
                include_all,
                limit,
                next,
            )
            .await
            .map_err(Into::<IndexerError>::into)?,
        )
    }

    /// Lookup account transactions. Transactions are returned newest to oldest.
    pub async fn lookup_account_transactions(
        &self,
        account_id: &str,
        limit: Option<i32>,
        next: Option<&str>,
        note_prefix: Option<&str>,
        tx_type: Option<&str>,
        sig_type: Option<&str>,
        txid: Option<&str>,
        round: Option<i32>,
        min_round: Option<i32>,
        max_round: Option<i32>,
        asset_id: Option<i32>,
        before_time: Option<String>,
        after_time: Option<String>,
        currency_greater_than: Option<i32>,
        currency_less_than: Option<i32>,
        rekey_to: Option<bool>,
    ) -> Result<LookupAccountTransactions200Response, Error> {
        Ok(
            algonaut_indexer::apis::lookup_api::lookup_account_transactions(
                &self.configuration,
                account_id,
                limit,
                next,
                note_prefix,
                tx_type,
                sig_type,
                txid,
                round,
                min_round,
                max_round,
                asset_id,
                before_time,
                after_time,
                currency_greater_than,
                currency_less_than,
                rekey_to,
            )
            .await
            .map_err(Into::<IndexerError>::into)?,
        )
    }

    /// Given an application ID and box name, returns base64 encoded box name and value. Box names must be in the goal app call arg form 'encoding:value'. For ints, use the form 'int:1234'. For raw bytes, encode base 64 and use 'b64' prefix as in 'b64:A=='. For printable strings, use the form 'str:hello'. For addresses, use the form 'addr:XYZ...'.
    pub async fn lookup_application_box_by_id_and_name(
        &self,
        application_id: i32,
        name: &str,
    ) -> Result<algonaut_indexer::models::Box, Error> {
        Ok(
            algonaut_indexer::apis::lookup_api::lookup_application_box_by_id_and_name(
                &self.configuration,
                application_id,
                name,
            )
            .await
            .map_err(Into::<IndexerError>::into)?,
        )
    }

    /// Lookup application.
    pub async fn lookup_application_by_id(
        &self,
        application_id: i32,
        include_all: Option<bool>,
    ) -> Result<LookupApplicationById200Response, Error> {
        Ok(
            algonaut_indexer::apis::lookup_api::lookup_application_by_id(
                &self.configuration,
                application_id,
                include_all,
            )
            .await
            .map_err(Into::<IndexerError>::into)?,
        )
    }

    /// Lookup application logs.
    pub async fn lookup_application_logs_by_id(
        &self,
        application_id: i32,
        limit: Option<i32>,
        next: Option<&str>,
        txid: Option<&str>,
        min_round: Option<i32>,
        max_round: Option<i32>,
        sender_address: Option<&str>,
    ) -> Result<LookupApplicationLogsById200Response, Error> {
        Ok(
            algonaut_indexer::apis::lookup_api::lookup_application_logs_by_id(
                &self.configuration,
                application_id,
                limit,
                next,
                txid,
                min_round,
                max_round,
                sender_address,
            )
            .await
            .map_err(Into::<IndexerError>::into)?,
        )
    }

    /// Lookup the list of accounts who hold this asset
    pub async fn lookup_asset_balances(
        &self,
        asset_id: i32,
        include_all: Option<bool>,
        limit: Option<i32>,
        next: Option<&str>,
        currency_greater_than: Option<i32>,
        currency_less_than: Option<i32>,
    ) -> Result<LookupAssetBalances200Response, Error> {
        Ok(algonaut_indexer::apis::lookup_api::lookup_asset_balances(
            &self.configuration,
            asset_id,
            include_all,
            limit,
            next,
            currency_greater_than,
            currency_less_than,
        )
        .await
        .map_err(Into::<IndexerError>::into)?)
    }

    /// Lookup asset information.
    pub async fn lookup_asset_by_id(
        &self,
        asset_id: i32,
        include_all: Option<bool>,
    ) -> Result<LookupAssetById200Response, Error> {
        Ok(algonaut_indexer::apis::lookup_api::lookup_asset_by_id(
            &self.configuration,
            asset_id,
            include_all,
        )
        .await
        .map_err(Into::<IndexerError>::into)?)
    }

    /// Lookup transactions for an asset. Transactions are returned oldest to newest.
    pub async fn lookup_asset_transactions(
        &self,
        asset_id: i32,
        limit: Option<i32>,
        next: Option<&str>,
        note_prefix: Option<&str>,
        tx_type: Option<&str>,
        sig_type: Option<&str>,
        txid: Option<&str>,
        round: Option<i32>,
        min_round: Option<i32>,
        max_round: Option<i32>,
        before_time: Option<String>,
        after_time: Option<String>,
        currency_greater_than: Option<i32>,
        currency_less_than: Option<i32>,
        address: Option<&str>,
        address_role: Option<&str>,
        exclude_close_to: Option<bool>,
        rekey_to: Option<bool>,
    ) -> Result<LookupAccountTransactions200Response, Error> {
        Ok(
            algonaut_indexer::apis::lookup_api::lookup_asset_transactions(
                &self.configuration,
                asset_id,
                limit,
                next,
                note_prefix,
                tx_type,
                sig_type,
                txid,
                round,
                min_round,
                max_round,
                before_time,
                after_time,
                currency_greater_than,
                currency_less_than,
                address,
                address_role,
                exclude_close_to,
                rekey_to,
            )
            .await
            .map_err(Into::<IndexerError>::into)?,
        )
    }

    /// Lookup block.
    pub async fn lookup_block(
        &self,
        round_number: i32,
        header_only: Option<bool>,
    ) -> Result<Block, Error> {
        Ok(algonaut_indexer::apis::lookup_api::lookup_block(
            &self.configuration,
            round_number,
            header_only,
        )
        .await
        .map_err(Into::<IndexerError>::into)?)
    }

    /// Lookup a single transaction.
    pub async fn lookup_transaction(
        &self,
        txid: &str,
    ) -> Result<LookupTransaction200Response, Error> {
        Ok(
            algonaut_indexer::apis::lookup_api::lookup_transaction(&self.configuration, txid)
                .await
                .map_err(Into::<IndexerError>::into)?,
        )
    }

    /// Search for accounts.
    pub async fn search_for_accounts(
        &self,
        asset_id: Option<i32>,
        limit: Option<i32>,
        next: Option<&str>,
        currency_greater_than: Option<i32>,
        include_all: Option<bool>,
        exclude: Option<Vec<String>>,
        currency_less_than: Option<i32>,
        auth_addr: Option<&str>,
        round: Option<i32>,
        application_id: Option<i32>,
    ) -> Result<SearchForAccounts200Response, Error> {
        Ok(algonaut_indexer::apis::search_api::search_for_accounts(
            &self.configuration,
            asset_id,
            limit,
            next,
            currency_greater_than,
            include_all,
            exclude,
            currency_less_than,
            auth_addr,
            round,
            application_id,
        )
        .await
        .map_err(Into::<IndexerError>::into)?)
    }

    /// Given an application ID, returns the box names of that application sorted lexicographically.
    pub async fn search_for_application_boxes(
        &self,
        application_id: i32,
        limit: Option<i32>,
        next: Option<&str>,
    ) -> Result<SearchForApplicationBoxes200Response, Error> {
        Ok(
            algonaut_indexer::apis::search_api::search_for_application_boxes(
                &self.configuration,
                application_id,
                limit,
                next,
            )
            .await
            .map_err(Into::<IndexerError>::into)?,
        )
    }

    /// Search for applications
    pub async fn search_for_applications(
        &self,
        application_id: Option<i32>,
        creator: Option<&str>,
        include_all: Option<bool>,
        limit: Option<i32>,
        next: Option<&str>,
    ) -> Result<LookupAccountCreatedApplications200Response, Error> {
        Ok(algonaut_indexer::apis::search_api::search_for_applications(
            &self.configuration,
            application_id,
            creator,
            include_all,
            limit,
            next,
        )
        .await
        .map_err(Into::<IndexerError>::into)?)
    }

    /// Search for assets.
    pub async fn search_for_assets(
        &self,
        include_all: Option<bool>,
        limit: Option<i32>,
        next: Option<&str>,
        creator: Option<&str>,
        name: Option<&str>,
        unit: Option<&str>,
        asset_id: Option<i32>,
    ) -> Result<LookupAccountCreatedAssets200Response, Error> {
        Ok(algonaut_indexer::apis::search_api::search_for_assets(
            &self.configuration,
            include_all,
            limit,
            next,
            creator,
            name,
            unit,
            asset_id,
        )
        .await
        .map_err(Into::<IndexerError>::into)?)
    }

    /// Search for transactions. Transactions are returned oldest to newest unless the address parameter is used, in which case results are returned newest to oldest.
    pub async fn search_for_transactions(
        &self,
        limit: Option<i32>,
        next: Option<&str>,
        note_prefix: Option<&str>,
        tx_type: Option<&str>,
        sig_type: Option<&str>,
        txid: Option<&str>,
        round: Option<i32>,
        min_round: Option<i32>,
        max_round: Option<i32>,
        asset_id: Option<i32>,
        before_time: Option<String>,
        after_time: Option<String>,
        currency_greater_than: Option<i32>,
        currency_less_than: Option<i32>,
        address: Option<&str>,
        address_role: Option<&str>,
        exclude_close_to: Option<bool>,
        rekey_to: Option<bool>,
        application_id: Option<i32>,
    ) -> Result<LookupAccountTransactions200Response, Error> {
        Ok(algonaut_indexer::apis::search_api::search_for_transactions(
            &self.configuration,
            limit,
            next,
            note_prefix,
            tx_type,
            sig_type,
            txid,
            round,
            min_round,
            max_round,
            asset_id,
            before_time,
            after_time,
            currency_greater_than,
            currency_less_than,
            address,
            address_role,
            exclude_close_to,
            rekey_to,
            application_id,
        )
        .await
        .map_err(Into::<IndexerError>::into)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_with_valid_url() {
        let indexer = Indexer::new("http://example.com", "");
        assert!(indexer.ok().is_some());
    }
}
