use algonaut_abi::abi_error::AbiError;
use algonaut_transaction::error::TransactionError;
use std::fmt::Debug;
use std::time::Duration;
use thiserror::Error;

// `Error` is intentionally not `Clone`/`PartialEq`/`Eq`: it carries source
// errors (`TransactionError`, `AbiError`) via `#[from]` to preserve the
// `std::error::Error::source()` chain, and those sources aren't `Eq`.
// Match on the variant rather than comparing error values.
#[derive(Error, Debug)]
pub enum Error {
    /// URL parse error.
    #[error("Url parsing error.")]
    BadUrl(String),
    /// Token parse error.
    #[error("Token parsing error.")]
    BadToken,
    /// Header parse error.
    #[error("Headers parsing error.")]
    BadHeader(String),
    /// Missing the base URL of the REST API server.
    #[error("Set an URL before calling build.")]
    UnitializedUrl,
    /// Missing the authentication token for the REST API server.
    #[error("Set a token before calling build.")]
    UnitializedToken,
    /// HTTP calls errors
    #[error("http error: {0}")]
    Request(RequestError),

    /// Tried to
    /// [`build`](crate::atomic::AtomicGroupBuilder::build) a
    /// transaction group with zero transactions. Equivalent of
    /// [`algonaut_transaction::error::TransactionError::EmptyTransactionListError`]
    /// at the top-level error layer.
    #[error("transaction group is empty")]
    EmptyTransactionGroup,
    /// The transaction group is at maximum capacity (16 txns by protocol).
    #[error("composer group full (max {max} transactions)")]
    ComposerGroupFull { max: usize },
    /// An ABI method call returned but the application did not emit a
    /// matching `log` entry, so the return value cannot be decoded.
    #[error("app call transaction did not log a return value")]
    MissingReturnLog,
    /// `algod` was asked to compile TEAL with a source-map but the response
    /// did not carry one.
    #[error("algod did not return a sourcemap")]
    MissingSourcemap,
    /// Algod reported that the transaction was kicked out of its pool
    /// before being included in a block (expired `LastValid`, underfunded,
    /// group invalid, etc.). The `reason` string is algod's `pool-error`
    /// verbatim and is diagnostic only — match on the variant, not the
    /// message. Note: this is a node-local view; a tx evicted from one
    /// node's pool could in principle still be alive in peers' pools.
    #[error("transaction pool error: {reason}")]
    PendingTransactionPoolError { reason: String },
    /// [`PendingSubmission::confirm_with`] (or the equivalent internal
    /// helper on the atomic-transaction-composer) reached its deadline
    /// without observing a confirmation.
    ///
    /// [`PendingSubmission::confirm_with`]: crate::algod::v2::PendingSubmission::confirm_with
    #[error("pending transaction timed out ({timeout:?})")]
    PendingTransactionTimeout { timeout: Duration },
    /// A [`Signer`](algonaut_transaction::Signer) returned output that does
    /// not match the request the composer made: wrong count, wrong order,
    /// or a signature wrapping a different transaction than the one asked
    /// for. The signed group is rejected rather than submitted. `reason`
    /// is diagnostic only.
    #[error("signer returned invalid output: {reason}")]
    SignerOutputInvalid { reason: String },
    /// [`sign`](crate::atomic::UnsignedAtomicGroup::sign)
    /// was called on a group whose slot at `index` has no signer
    /// (`TransactionWithSigner::unsigned`). An unsigned slot cannot
    /// produce a submittable signature; it is only valid for `simulate`.
    /// Attach a signer, or simulate the group instead of signing it.
    #[error("transaction at index {index} has no signer (only valid for simulate)")]
    MissingSigner { index: usize },

    /// A transaction construction or signing error from
    /// [`algonaut_transaction`], preserved as the error source.
    #[error(transparent)]
    Transaction(#[from] TransactionError),
    /// An ABI encode/decode error from [`algonaut_abi`], preserved as the
    /// error source.
    #[error(transparent)]
    Abi(#[from] AbiError),

    /// General text-only errors. Dedicated error variants can be created, if needed.
    #[error("Msg: {0}")]
    Msg(String),
    /// Clearly SDK caused errors (please open an [issue](https://github.com/manuelmauro/algonaut/issues)!)
    /// TODO rename in unexpected
    #[error("Internal error: {0}")]
    Internal(String),
}

impl Error {
    /// Returns if the error is a `RequestError` that failed with a status code of 404.
    pub fn is_404(&self) -> bool {
        if let Some(e) = self.as_request_error() {
            e.is_404()
        } else {
            false
        }
    }

    /// Gets the details of a request error, or none otherwise.
    fn as_request_error(&self) -> Option<&RequestError> {
        match self {
            Self::Request(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Error, Clone, Debug, PartialEq, Eq)]
#[error("{:?}, {}", url, details)]
pub struct RequestError {
    pub url: Option<String>,
    pub details: RequestErrorDetails,
}

impl RequestError {
    pub fn new(url: Option<String>, details: RequestErrorDetails) -> RequestError {
        RequestError { url, details }
    }

    /// Returns if the cause of the error is a 404 response from the client.
    fn is_404(&self) -> bool {
        self.details.status() == Some(404)
    }
}

#[derive(Error, Clone, Debug, PartialEq, Eq)]
pub enum RequestErrorDetails {
    /// Http call error with optional message (returned by remote API)
    #[error("Http error: {}, {}", status, message)]
    Http { status: u16, message: String },
    /// Timeout
    #[error("Timeout connecting to the server.")]
    Timeout,
    /// Client generated errors (while e.g. building request or decoding response)
    #[error("Client error: {}", description)]
    Client { description: String },
}

impl RequestErrorDetails {
    /// Gets the status code of the request.
    ///
    /// Returns `None` if the request did not receive a response.
    fn status(&self) -> Option<u16> {
        match self {
            Self::Http { status, .. } => Some(*status),
            _ => None,
        }
    }
}

impl From<crate::algod::v2::error::AlgodError> for Error {
    fn from(error: crate::algod::v2::error::AlgodError) -> Self {
        match error {
            crate::algod::v2::error::AlgodError::Msg(msg) => Error::Msg(msg),
        }
    }
}

impl From<crate::indexer::v2::error::IndexerError> for Error {
    fn from(error: crate::indexer::v2::error::IndexerError) -> Self {
        match error {
            crate::indexer::v2::error::IndexerError::Msg(msg) => Error::Msg(msg),
        }
    }
}

impl From<algonaut_kmd::error::ClientError> for Error {
    fn from(error: algonaut_kmd::error::ClientError) -> Self {
        match error {
            algonaut_kmd::error::ClientError::BadUrl(msg) => Error::BadUrl(msg),
            algonaut_kmd::error::ClientError::BadToken => Error::BadToken,
            algonaut_kmd::error::ClientError::BadHeader(msg) => Error::BadHeader(msg),
            algonaut_kmd::error::ClientError::Request(e) => Error::Request(e.into()),
            algonaut_kmd::error::ClientError::Msg(msg) => Error::Msg(msg),
        }
    }
}

impl From<algonaut_kmd::error::RequestError> for RequestError {
    fn from(error: algonaut_kmd::error::RequestError) -> Self {
        RequestError::new(error.url.clone(), error.details.into())
    }
}

impl From<algonaut_kmd::error::RequestErrorDetails> for RequestErrorDetails {
    fn from(details: algonaut_kmd::error::RequestErrorDetails) -> Self {
        match details {
            algonaut_kmd::error::RequestErrorDetails::Http { status, message } => {
                RequestErrorDetails::Http { status, message }
            }
            algonaut_kmd::error::RequestErrorDetails::Timeout => RequestErrorDetails::Timeout {},
            algonaut_kmd::error::RequestErrorDetails::Client { description } => {
                RequestErrorDetails::Client { description }
            }
        }
    }
}

impl From<rmp_serde::encode::Error> for Error {
    fn from(error: rmp_serde::encode::Error) -> Self {
        Error::Internal(error.to_string())
    }
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Error::Internal(error)
    }
}

#[test]
fn check_404() {
    let not_found_error = Error::Request(RequestError::new(
        Some("testing".to_owned()),
        RequestErrorDetails::Http {
            status: 404,
            message: "not found".to_owned(),
        },
    ));

    let bad_request_error = Error::Request(RequestError::new(
        None,
        RequestErrorDetails::Http {
            status: 400,
            message: "bad request".to_owned(),
        },
    ));

    let unrelated_error = Error::UnitializedToken;

    assert!(
        not_found_error.is_404(),
        "a 404 request error is saying that it is not a 404 error"
    );
    assert!(
        !bad_request_error.is_404(),
        "a 400 request error is saying that it is a 404 error"
    );
    assert!(
        !unrelated_error.is_404(),
        "an unrelated request error is saying that it is a 404 error"
    );
}
