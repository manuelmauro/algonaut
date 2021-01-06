use crate::error::Error;
use crate::models::{
    Account, Block, NodeStatus, PendingTransactions, Round, Supply, Transaction, TransactionFee,
    TransactionID, TransactionList, TransactionParams, Version,
};
use crate::transaction::SignedTransaction;
use derive_more::Display;
use reqwest::header::HeaderMap;

const AUTH_HEADER: &str = "X-Algo-API-Token";

/// Url
#[derive(Display)]
#[display(fmt = "{}", url)]
pub struct Url {
    url: url::Url,
}

impl Url {
    pub fn parse(url: &str) -> Result<Self, Error> {
        Ok(Url {
            url: url::Url::parse(url).map_err(|_err| Error::Url)?,
        })
    }
}

/// Token
#[derive(Display)]
#[display(fmt = "{}", token)]
pub struct ApiToken {
    token: String,
}

impl ApiToken {
    pub fn parse(token: &str) -> Result<Self, Error> {
        Ok(ApiToken {
            token: token.to_string(),
        })
    }
}

/// Algod
/// ```
/// use algorust::algod::Algod;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let algod = Algod::new()
///         .bind("http://localhost:4001")?
///         .auth("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")?
///         .client()?;
///
///     Ok(())
/// }
/// ```
pub struct Algod {
    url: Option<Url>,
    token: Option<ApiToken>,
    headers: HeaderMap,
}

impl Algod {
    /// Start the creation of a client.
    pub fn new() -> Self {
        Algod {
            url: None,
            token: None,
            headers: HeaderMap::new(),
        }
    }

    /// Bind to a URL.
    pub fn bind(mut self, url: &str) -> Result<Self, Error> {
        self.url = Some(Url::parse(url)?);
        Ok(self)
    }

    /// Use a token to authenticate.
    pub fn auth(mut self, token: &str) -> Result<Self, Error> {
        self.token = Some(ApiToken::parse(token)?);
        Ok(self)
    }

    /// Build a client for Algorand protocol daemon.
    pub fn client(self) -> Result<Client, Error> {
        if let None = self.url {
            return Err(Error::Url);
        }

        if let None = self.token {
            return Err(Error::Url);
        }

        Ok(Client {
            url: self.url.unwrap().to_string(),
            token: self.token.unwrap().to_string(),
            headers: self.headers,
        })
    }
}

/// Client for interacting with the Algorand protocol daemon.
pub struct Client {
    url: String,
    token: String,
    headers: HeaderMap,
}

impl Client {
    /// Returns Ok if healthy
    pub fn health(&self) -> Result<(), Error> {
        let _ = reqwest::Client::new()
            .get(&format!("{}health", self.url))
            .headers(self.headers.clone())
            .send()?
            .error_for_status()?;
        Ok(())
    }

    /// Retrieves the current version
    pub fn versions(&self) -> Result<Version, Error> {
        let response = reqwest::Client::new()
            .get(&format!("{}versions", self.url))
            .headers(self.headers.clone())
            .header(AUTH_HEADER, &self.token)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Gets the current node status
    pub fn status(&self) -> Result<NodeStatus, Error> {
        let response = reqwest::Client::new()
            .get(&format!("{}v1/status", self.url))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Waits for a block to appear after the specified round and returns the node status at the time
    pub fn status_after_block(&self, round: Round) -> Result<NodeStatus, Error> {
        let response = reqwest::Client::new()
            .get(&format!(
                "{}v1/status/wait-for-block-after/{}",
                self.url, round.0
            ))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Get the block for the given round
    pub fn block(&self, round: Round) -> Result<Block, Error> {
        let response = reqwest::Client::new()
            .get(&format!("{}v1/block/{}", self.url, round.0))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Gets the current supply reported by the ledger
    pub fn ledger_supply(&self) -> Result<Supply, Error> {
        let response = reqwest::Client::new()
            .get(&format!("{}v1/ledger/supply", self.url))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    pub fn account_information(&self, address: &str) -> Result<Account, Error> {
        let response = reqwest::Client::new()
            .get(&format!("{}v1/account/{}", self.url, address))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Gets a list of unconfirmed transactions currently in the transaction pool
    ///
    /// Sorted by priority in decreasing order and truncated at the specified limit, or returns all if specified limit is 0
    pub fn pending_transactions(&self, limit: u64) -> Result<PendingTransactions, Error> {
        let response = reqwest::Client::new()
            .get(&format!("{}v1/transactions/pending", self.url))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .query(&[("max", limit.to_string())])
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Get a specified pending transaction
    pub fn pending_transaction_information(
        &self,
        transaction_id: &str,
    ) -> Result<Transaction, Error> {
        let response = reqwest::Client::new()
            .get(&format!(
                "{}v1/transactions/pending/{}",
                self.url, transaction_id
            ))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Get a list of confirmed transactions, limited to filters if specified
    pub fn transactions(
        &self,
        address: &str,
        first_round: Option<Round>,
        last_round: Option<Round>,
        from_date: Option<String>,
        to_date: Option<String>,
        limit: Option<u64>,
    ) -> Result<TransactionList, Error> {
        let mut query = Vec::new();
        if let Some(first_round) = first_round {
            query.push(("firstRound", first_round.0.to_string()))
        }
        if let Some(last_round) = last_round {
            query.push(("lastRound", last_round.0.to_string()))
        }
        if let Some(from_date) = from_date {
            query.push(("fromDate", from_date))
        }
        if let Some(to_date) = to_date {
            query.push(("toDate", to_date))
        }
        if let Some(limit) = limit {
            query.push(("max", limit.to_string()))
        }
        let response = reqwest::Client::new()
            .get(&format!("{}v1/account/{}/transactions", self.url, address))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .query(&query)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Broadcasts a transaction to the network
    pub fn send_transaction(
        &self,
        signed_transaction: &SignedTransaction,
    ) -> Result<TransactionID, Error> {
        let bytes = rmp_serde::to_vec_named(signed_transaction)?;
        self.raw_transaction(&bytes)
    }

    /// Broadcasts a raw transaction to the network
    pub fn raw_transaction(&self, raw: &[u8]) -> Result<TransactionID, Error> {
        let response = reqwest::Client::new()
            .post(&format!("{}v1/transactions", self.url))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .body(raw.to_vec())
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Gets the information of a single transaction
    pub fn transaction(&self, transaction_id: &str) -> Result<Transaction, Error> {
        let response = reqwest::Client::new()
            .get(&format!("{}v1/transaction/{}", self.url, transaction_id))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Gets a specific confirmed transaction
    pub fn transaction_information(
        &self,
        address: &str,
        transaction_id: &str,
    ) -> Result<Transaction, Error> {
        let response = reqwest::Client::new()
            .get(&format!(
                "{}/v1/account/{}/transaction/{}",
                self.url, address, transaction_id
            ))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Gets suggested fee in units of micro-Algos per byte
    pub fn suggested_fee(&self) -> Result<TransactionFee, Error> {
        let response = reqwest::Client::new()
            .get(&format!("{}/v1/transactions/fee", self.url))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Gets parameters for constructing a new transaction
    pub fn transaction_params(&self) -> Result<TransactionParams, Error> {
        let response = reqwest::Client::new()
            .get(&format!("{}/v1/transactions/params", self.url))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
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
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: Url")]
    fn test_bad_url() {
        Url::parse("bad_url").unwrap();
    }

    #[test]
    fn test_proper_client_builder() -> Result<(), Box<dyn std::error::Error>> {
        let algod = Algod::new()
            .bind("http://localhost:4001")?
            .auth("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")?
            .client();

        assert!(algod.ok().is_some());

        Ok(())
    }

    #[test]
    fn test_client_builder_with_no_token() -> Result<(), Box<dyn std::error::Error>> {
        let algod = Algod::new().bind("http://localhost:4001")?.client();

        assert!(algod.err().is_some());

        Ok(())
    }

    #[test]
    fn test_client_builder_with_no_url() -> Result<(), Box<dyn std::error::Error>> {
        let algod = Algod::new()
            .auth("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")?
            .client();

        assert!(algod.err().is_some());

        Ok(())
    }
}
