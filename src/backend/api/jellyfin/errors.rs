use std::fmt::Debug;
use std::fmt::Display;

use reqwest::StatusCode;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

/// An enum representing the different types of [`ApiError::RequestError`] \
/// Automatically coerces into an [`ApiError::RequestError`]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RequestError {
    Unauthorized,
    Forbidden,
    /// Should contain the serde Error, a message and a Source
    SerdeError(String, String, String),
    MalformedUrl(String),
    Unknown(String),
}

impl From<RequestError> for ApiError {
    fn from(value: RequestError) -> Self {
        ApiError::RequestError(value)
    }
}

impl Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestError::Unauthorized => f.write_str("Unauthorized"),
            RequestError::Forbidden => f.write_str("Forbidden"),
            RequestError::Unknown(s) => f.write_str(s),
            RequestError::MalformedUrl(s) => f.write_str(s),
            RequestError::SerdeError(error, message , source) => f.write_str(&format!("{error}: {message} in source:\n{source}")),
        }
    }
}

/// An Error representing different problems with Tauri's backend API \
/// Should be used as Error type on all Tauri commands
#[derive(Error, Debug, Serialize, Deserialize, Clone)]
pub enum ApiError {
    #[error("Request Unauthorized")]
    ApiUnauthorized,
    #[error("Request to server failed: {0}")]
    RequestError(RequestError),
    #[error("Unsupported API call! {0} for Endpoint {1}")]
    Unsupported(String, String),
}

impl From<reqwest::Error> for ApiError {
    fn from(value: reqwest::Error) -> Self {
        let reason = match value.status() {
            Some(status) => match status {
                StatusCode::UNAUTHORIZED => RequestError::Unauthorized,
                StatusCode::FORBIDDEN => RequestError::Forbidden,
                _ => RequestError::Unknown(value.to_string()),
            },
            None => RequestError::Unknown(value.to_string()),
        };

        ApiError::RequestError(reason)
    }
}

impl From<url::ParseError> for ApiError {
    fn from(value: url::ParseError) -> Self {
        ApiError::RequestError(RequestError::MalformedUrl(value.to_string()))
    }
}
