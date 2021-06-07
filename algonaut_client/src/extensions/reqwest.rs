use crate::error::HttpError;
use async_trait::async_trait;
use reqwest::Response;
use serde::Deserialize;

#[async_trait]
pub(crate) trait ResponseExt {
    async fn http_error_for_status(self) -> Result<Response, HttpError>;
}

#[async_trait]
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
