use crate::error::HttpError;
use reqwest::blocking::Response;
use serde::Deserialize;

pub(crate) trait ResponseExt {
    fn http_error_for_status(self) -> Result<Response, HttpError>;
}

impl ResponseExt for Response {
    fn http_error_for_status(self) -> Result<Response, HttpError> {
        match self.error_for_status_ref() {
            Ok(_) => Ok(self),
            Err(error) => match self.json::<HttpErrorPayload>() {
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
