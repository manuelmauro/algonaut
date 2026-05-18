//! Builder for [`algonaut_algod::models::DryrunRequest`].
//!
//! Constructing a `DryrunRequest` by hand is noisy — accounts, apps,
//! sources, and txns each have to be laid out separately, and the
//! ledger-context fields (round, timestamp, protocol version) are
//! required by the wire format even when callers don't care. This
//! builder folds the common patterns into a fluent API:
//!
//! ```ignore
//! use algonaut::dryrun::DryrunRequestBuilder;
//! use algonaut::transaction::SignedTransaction;
//!
//! let req = DryrunRequestBuilder::from_signed_txns(&signed)?
//!     .add_compiled_source(compiled, "approv", 0, 0)
//!     .build()?;
//! ```

use algonaut_algod::models::{Account, Application, DryrunRequest, DryrunSource};
use algonaut_core::CompiledTeal;
use algonaut_encoding::Bytes;
use algonaut_model::transaction::ApiSignedTransaction;
use algonaut_transaction::SignedTransaction;
use algonaut_transaction::error::TransactionError;

/// Fluent builder for [`DryrunRequest`].
#[derive(Debug, Clone, Default)]
pub struct DryrunRequestBuilder {
    accounts: Vec<Account>,
    apps: Vec<Application>,
    sources: Vec<DryrunSource>,
    txns: Vec<ApiSignedTransaction>,
    round: u64,
    latest_timestamp: u64,
    protocol_version: String,
}

impl DryrunRequestBuilder {
    /// Start with an empty request.
    pub fn new() -> Self {
        Self::default()
    }

    /// Convenience entry point that wires up the request's `txns` from
    /// a list of already-signed transactions.
    pub fn from_signed_txns(signed: &[SignedTransaction]) -> Result<Self, TransactionError> {
        let mut me = Self::new();
        for tx in signed {
            me.txns.push(ApiSignedTransaction::try_from(tx.clone())?);
        }
        Ok(me)
    }

    /// Replace the accounts list.
    pub fn accounts(mut self, accounts: Vec<Account>) -> Self {
        self.accounts = accounts;
        self
    }

    /// Replace the apps list.
    pub fn apps(mut self, apps: Vec<Application>) -> Self {
        self.apps = apps;
        self
    }

    /// Set the simulated round.
    pub fn round(mut self, round: u64) -> Self {
        self.round = round;
        self
    }

    /// Set the latest timestamp (Unix seconds).
    pub fn latest_timestamp(mut self, ts: u64) -> Self {
        self.latest_timestamp = ts;
        self
    }

    /// Set the consensus protocol version. Defaults to empty, which
    /// asks algod to use its own current protocol.
    pub fn protocol_version(mut self, version: impl Into<String>) -> Self {
        self.protocol_version = version.into();
        self
    }

    /// Append a compiled TEAL source for use at `txn_index` /
    /// `app_index`. `field_name` is one of `"lsig"`, `"approv"`, or
    /// `"clearp"`.
    pub fn add_compiled_source(
        mut self,
        program: CompiledTeal,
        field_name: impl Into<String>,
        txn_index: u64,
        app_index: u64,
    ) -> Self {
        self.sources.push(DryrunSource {
            app_index,
            field_name: field_name.into(),
            source: Bytes(program.0),
            txn_index,
        });
        self
    }

    /// Append a TEAL source (raw text) for use at `txn_index` /
    /// `app_index`. Algod will compile the source as part of the
    /// dryrun.
    pub fn add_text_source(
        mut self,
        source: impl Into<Vec<u8>>,
        field_name: impl Into<String>,
        txn_index: u64,
        app_index: u64,
    ) -> Self {
        self.sources.push(DryrunSource {
            app_index,
            field_name: field_name.into(),
            source: Bytes(source.into()),
            txn_index,
        });
        self
    }

    /// Append an already-built [`DryrunSource`]. Useful when callers
    /// need a `field_name` we don't recognise.
    pub fn add_source(mut self, source: DryrunSource) -> Self {
        self.sources.push(source);
        self
    }

    /// Append a signed transaction.
    pub fn add_signed_txn(mut self, tx: &SignedTransaction) -> Result<Self, TransactionError> {
        self.txns.push(ApiSignedTransaction::try_from(tx.clone())?);
        Ok(self)
    }

    /// Materialise the underlying [`DryrunRequest`].
    pub fn build(self) -> DryrunRequest {
        DryrunRequest {
            accounts: self.accounts,
            apps: self.apps,
            latest_timestamp: self.latest_timestamp,
            protocol_version: self.protocol_version,
            round: self.round,
            sources: self.sources,
            txns: self.txns,
        }
    }
}

/// `field_name` values that algod accepts on a [`DryrunSource`]. Not an
/// enum because the wire format is a free-form string and we don't want
/// to lock callers out of any future additions, but exposing the
/// well-known constants stops cucumber tests from sprinkling magic
/// strings.
pub mod field_name {
    pub const LSIG: &str = "lsig";
    pub const APPROV: &str = "approv";
    pub const CLEARP: &str = "clearp";
}

/// Convenience accessors for a dryrun response. Algod returns
/// app-call-messages / logic-sig-messages whose last entry is `"PASS"`
/// or `"REJECT"`; collapse that into a typed result.
pub mod result {
    use algonaut_algod::models::{DryrunTxnResult, TealDryrun200Response};

    /// Last message in the app-call trace, if any.
    pub fn app_call_status(txn: &DryrunTxnResult) -> Option<&str> {
        txn.app_call_messages
            .as_ref()
            .and_then(|msgs: &Vec<String>| msgs.last().map(String::as_str))
    }

    /// Last message in the logic-sig trace, if any.
    pub fn logic_sig_status(txn: &DryrunTxnResult) -> Option<&str> {
        txn.logic_sig_messages
            .as_ref()
            .and_then(|msgs: &Vec<String>| msgs.last().map(String::as_str))
    }

    /// Combined status: whichever of the two trace kinds populated a
    /// status string. Returns `None` if neither did.
    pub fn overall_status(txn: &DryrunTxnResult) -> Option<&str> {
        logic_sig_status(txn).or_else(|| app_call_status(txn))
    }

    /// Convenience for the first txn in a response.
    pub fn first_status(resp: &TealDryrun200Response) -> Option<&str> {
        resp.txns.first().and_then(overall_status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_starts_empty() {
        let req = DryrunRequestBuilder::new().build();
        assert!(req.accounts.is_empty());
        assert!(req.apps.is_empty());
        assert!(req.sources.is_empty());
        assert!(req.txns.is_empty());
        assert_eq!(req.round, 0);
        assert_eq!(req.protocol_version, "");
    }

    #[test]
    fn add_source_round_trip() {
        let program = CompiledTeal(vec![0x02, 0x20, 0x01, 0x01, 0x22]);
        let req = DryrunRequestBuilder::new()
            .add_compiled_source(program.clone(), field_name::APPROV, 0, 0)
            .round(123)
            .protocol_version("future")
            .build();

        assert_eq!(req.sources.len(), 1);
        assert_eq!(req.sources[0].field_name, "approv");
        assert_eq!(req.sources[0].source.0, program.0);
        assert_eq!(req.round, 123);
        assert_eq!(req.protocol_version, "future");
    }
}
