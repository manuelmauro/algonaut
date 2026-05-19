//! A minimal recording HTTP server for the **unit** path features.
//!
//! The `v2algodclient_paths` / `v2indexerclient_paths` features assert the
//! exact request *path* (including query string) that the SDK's HTTP client
//! emits. None of them need a real algod / indexer node — only a server that
//! captures the request line and answers with something parseable enough not
//! to panic the client before the request has been recorded.
//!
//! For every accepted connection the server reads the HTTP request line
//! (`<METHOD> <PATH> HTTP/1.1`), stores the method + full path into shared
//! state, and replies with `200 OK` and a tiny `{}` body. The SDK call may
//! still fail to *parse* that body — the path-assertion steps ignore the
//! call's `Result`, so that is harmless.

use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// A single recorded HTTP request: the method and the full request target
/// (path plus query string), exactly as the client put it on the wire.
#[derive(Clone, Debug, Default)]
pub struct RecordedRequest {
    pub method: String,
    pub path: String,
}

/// Handle to a running mock server. Dropping it leaves the background task
/// running until the process exits — fine for a test binary.
#[derive(Clone, Debug)]
pub struct MockServer {
    /// `http://127.0.0.1:<port>` base URL the SDK clients are pointed at.
    pub base_url: String,
    /// The most recent request the server observed.
    recorded: Arc<Mutex<Option<RecordedRequest>>>,
}

impl MockServer {
    /// Bind an ephemeral port on loopback and start serving in the background.
    pub async fn start() -> MockServer {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind mock server");
        let port = listener.local_addr().expect("no local addr").port();
        let recorded: Arc<Mutex<Option<RecordedRequest>>> = Arc::new(Mutex::new(None));

        let recorded_for_task = recorded.clone();
        tokio::spawn(async move {
            loop {
                let Ok((mut stream, _)) = listener.accept().await else {
                    continue;
                };
                let recorded = recorded_for_task.clone();
                tokio::spawn(async move {
                    // Read enough bytes to cover the request line. Requests in
                    // these features are tiny, so a single read is plenty;
                    // loop a few times just in case the line is split.
                    let mut buf = Vec::with_capacity(1024);
                    let mut chunk = [0u8; 1024];
                    for _ in 0..4 {
                        match stream.read(&mut chunk).await {
                            Ok(0) => break,
                            Ok(n) => {
                                buf.extend_from_slice(&chunk[..n]);
                                if buf.windows(2).any(|w| w == b"\r\n") {
                                    break;
                                }
                            }
                            Err(_) => break,
                        }
                    }

                    if let Some(req) = parse_request_line(&buf) {
                        *recorded.lock().unwrap() = Some(req);
                    }

                    // Minimal valid response. `{}` is valid JSON; the client
                    // may still fail to deserialize it into a typed model,
                    // which the path steps deliberately ignore.
                    let _ = stream
                        .write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\n\r\n{}")
                        .await;
                    let _ = stream.flush().await;
                });
            }
        });

        MockServer {
            base_url: format!("http://127.0.0.1:{port}"),
            recorded,
        }
    }

    /// Return the most recently recorded request, panicking if the client
    /// never reached the server.
    pub fn last_request(&self) -> RecordedRequest {
        self.recorded
            .lock()
            .unwrap()
            .clone()
            .expect("no request reached the mock server")
    }
}

/// Parse `<METHOD> <TARGET> HTTP/1.1` out of the start of an HTTP request.
fn parse_request_line(buf: &[u8]) -> Option<RecordedRequest> {
    let text = String::from_utf8_lossy(buf);
    let line = text.lines().next()?;
    let mut parts = line.split_whitespace();
    let method = parts.next()?.to_string();
    let path = parts.next()?.to_string();
    Some(RecordedRequest { method, path })
}
