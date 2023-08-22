use actix_web::{body::BoxBody, HttpResponse, ResponseError};
use http::StatusCode;
use serde::Serialize;
use serde_json::json;
use std::fmt::{Debug, Display};
use thiserror::Error;

#[derive(Debug, Error)]
#[error(transparent)]
pub struct HttpError<T>(T);

impl<T> HttpError<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

pub trait ToHttpStatus {
    fn status_code(&self) -> StatusCode;
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    message: String,
}

impl<T: Debug + Display + ToHttpStatus> ResponseError for HttpError<T> {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.0.status_code()).json(json!({
            "message": self.to_string(),
        }))
    }
}
