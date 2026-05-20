//! Thin wrappers around [`wiremock::MockServer`] for the **unit** features.
//!
//! Two flavours, matched to the two feature families:
//!
//! - [`MockServer`] — used by `v2algodclient_paths` / `v2indexerclient_paths`.
//!   Answers every request with `200 OK` + `{}`; the step-defs assert on the
//!   *request path* the SDK emitted via [`MockServer::last_request`].
//!
//! - [`ResponseMockServer`] — used by `v2algodclient_responses` /
//!   `v2indexerclient_responses`. Configured up front with a canned body and a
//!   content type (the base64-decoded fixture, served as msgpack or JSON), it
//!   answers every accepted request with that exact body byte-for-byte. Each
//!   scenario starts a fresh server, so there is no per-request state.

use std::fmt;
use wiremock::matchers::any;
use wiremock::{Mock, MockServer as WireMockServer, ResponseTemplate};

/// A single recorded HTTP request: the method and the full request target
/// (path plus query string), exactly as the client put it on the wire.
#[derive(Clone, Debug, Default)]
pub struct RecordedRequest {
    pub method: String,
    pub path: String,
}

/// Recording mock server. Every request is answered with `200 OK` + `{}` —
/// the typed models may fail to deserialise that body, which the path steps
/// deliberately ignore. The interesting state is the captured request path.
pub struct MockServer {
    inner: WireMockServer,
    /// `http://127.0.0.1:<port>` base URL the SDK clients are pointed at.
    pub base_url: String,
}

impl MockServer {
    /// Bind an ephemeral loopback port and start serving in the background.
    pub async fn start() -> MockServer {
        let inner = WireMockServer::start().await;
        let base_url = inner.uri();
        Mock::given(any())
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(b"{}".to_vec(), "application/json"),
            )
            .mount(&inner)
            .await;
        MockServer { inner, base_url }
    }

    /// Return the most recently recorded request, panicking if the client
    /// never reached the server.
    pub async fn last_request(&self) -> RecordedRequest {
        let requests = self
            .inner
            .received_requests()
            .await
            .expect("request recording disabled on wiremock server");
        let req = requests.last().expect("no request reached the mock server");
        let path = match req.url.query() {
            Some(q) if !q.is_empty() => format!("{}?{}", req.url.path(), q),
            _ => req.url.path().to_string(),
        };
        RecordedRequest {
            method: req.method.to_string(),
            path,
        }
    }
}

impl fmt::Debug for MockServer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MockServer")
            .field("base_url", &self.base_url)
            .finish()
    }
}

/// Canned-response mock server. Every request is answered with the body and
/// content type supplied at construction time — let the SDK's content
/// negotiation pick the right decoder (`application/msgpack` for `*.base64`
/// fixtures, `application/json` for `*.json`).
pub struct ResponseMockServer {
    _inner: WireMockServer,
    /// `http://127.0.0.1:<port>` base URL the SDK clients are pointed at.
    pub base_url: String,
}

impl ResponseMockServer {
    /// Bind an ephemeral loopback port and start serving `body` (the raw HTTP
    /// response body — already base64-decoded) for every accepted request,
    /// labelled with `content_type` as the response `Content-Type` header.
    pub async fn start(body: Vec<u8>, content_type: &str) -> ResponseMockServer {
        let inner = WireMockServer::start().await;
        let base_url = inner.uri();
        Mock::given(any())
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, content_type))
            .mount(&inner)
            .await;
        ResponseMockServer {
            _inner: inner,
            base_url,
        }
    }
}

impl fmt::Debug for ResponseMockServer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResponseMockServer")
            .field("base_url", &self.base_url)
            .finish()
    }
}
