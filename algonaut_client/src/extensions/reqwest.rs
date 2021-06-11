use crate::error::HttpError;
use async_trait::async_trait;
use reqwest::Response;
use serde::Deserialize;

// reqwest::Response has thread unsafe contents with the WASM target,
// so it's required to implement Send, which is not possible.
// Since WASM is single threaded, this can be skipped, using ?Send
// https://docs.rs/async-trait/0.1.50/async_trait/#non-threadsafe-futures
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
// reqwest::Response is thread safe with non WASM targets
// async_trait doesn't need additional parameters.
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub(crate) trait ResponseExt {
    async fn http_error_for_status(self) -> Result<Response, HttpError>;
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl ResponseExt for Response {
    async fn http_error_for_status(self) -> Result<Response, HttpError> {
        match self.error_for_status_ref() {
            Ok(_) => Ok(self),
            Err(error) => match self.json::<HttpErrorPayload>().await {
                Ok(error_payload) => Err(HttpError {
                    reqwest_error: error,
                    message: error_payload.message,
                }),
                // JSON error is optional: if parsing fails we assume it's not present and return the call error.
                Err(_) => Err(HttpError {
                    reqwest_error: error,
                    message: "".to_owned(),
                }),
            },
        }
    }
}

#[derive(Deserialize)]
struct HttpErrorPayload {
    message: String,
}
