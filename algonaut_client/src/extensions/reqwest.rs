use crate::error::{RequestError, RequestErrorDetails};
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
    /// Maps error to custom error, with a possible message returned by API.
    async fn http_error_for_status(self) -> Result<Response, RequestError>;
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl ResponseExt for Response {
    async fn http_error_for_status(self) -> Result<Response, RequestError> {
        match self.error_for_status_ref() {
            // The response is not an error
            Ok(_) => Ok(self),
            // The response is an error
            Err(_) => Err(RequestError::new(
                Some(self.url().to_string()),
                RequestErrorDetails::Http {
                    status: self.status().as_u16(),
                    message: parse_error_message_or_empty_string(self).await,
                },
            )),
        }
    }
}

/// Try to retrieve error message from JSON.
/// If there's no message, return an empty string.
async fn parse_error_message_or_empty_string(response: Response) -> String {
    response
        .json::<HttpErrorPayload>()
        .await
        .map(|p| p.message)
        .unwrap_or_else(|_| "".to_owned())
}

#[derive(Deserialize)]
struct HttpErrorPayload {
    message: String,
}
