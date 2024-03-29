/*
 * Indexer
 *
 * Algorand ledger analytics API.
 *
 * The version of the OpenAPI document: 2.0
 *
 * Generated by: https://openapi-generator.tech
 */

/// OnCompletion : \\[apan\\] defines the what additional actions occur with the transaction.  Valid types: * noop * optin * closeout * clear * update * update * delete

/// \\[apan\\] defines the what additional actions occur with the transaction.  Valid types: * noop * optin * closeout * clear * update * update * delete
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
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

impl ToString for OnCompletion {
    fn to_string(&self) -> String {
        match self {
            Self::Noop => String::from("noop"),
            Self::Optin => String::from("optin"),
            Self::Closeout => String::from("closeout"),
            Self::Clear => String::from("clear"),
            Self::Update => String::from("update"),
            Self::Delete => String::from("delete"),
        }
    }
}

impl Default for OnCompletion {
    fn default() -> OnCompletion {
        Self::Noop
    }
}
