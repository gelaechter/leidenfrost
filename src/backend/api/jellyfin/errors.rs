use std::fmt::Debug;

use reqwest::StatusCode;
use sea_orm::DbErr;
use thiserror::Error;
use tokio::task::JoinError;

use crate::backend::api::endpoint_api::EndpointType;

/// An enum representing the different types of [`ApiError::RequestError`] \
/// Automatically coerces into an [`ApiError::RequestError`]
#[derive(Debug, Clone, Error)]
pub enum RequestError {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    /// Should contain the serde Error, a message and a Source
    #[error("{0}: {1} in source:\n{2}")]
    SerdeError(String, String, String),
    #[error("{0}")]
    MalformedUrl(String),
    #[error("{0}")]
    Unknown(String),
}

impl From<RequestError> for ApiError {
    fn from(value: RequestError) -> Self {
        ApiError::RequestError(value)
    }
}

/// An Error representing different problems with Tauri's backend API \
/// Should be used as Error type on all Tauri commands
#[derive(Error, Debug, Clone)]
pub enum ApiError {
    #[error("Request Unauthorized")]
    Unauthorized,
    #[error("Request to server failed: {0}")]
    RequestError(RequestError),
    #[error("Unsupported: Endpoint {endpoint} does not support {call}")]
    Unsupported { call: String, endpoint: String },
    #[error("The requested data point does not exist")]
    DataNotFound,
    #[error("Cannot access database directory")]
    DbPathInaccessible,
    #[error("Database Error: {0}")]
    DbError(#[from] DbErr),
    #[error("Async Error")]
    JoinError,
}

impl From<reqwest::Error> for ApiError {
    fn from(value: reqwest::Error) -> Self {
        match value.status() {
            Some(StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) => ApiError::Unauthorized,
            _ => RequestError::Unknown(value.to_string()).into(),
        }
    }
}

impl From<url::ParseError> for ApiError {
    fn from(value: url::ParseError) -> Self {
        ApiError::RequestError(RequestError::MalformedUrl(value.to_string()))
    }
}

impl From<JoinError> for ApiError {
    fn from(value: JoinError) -> Self {
        ApiError::JoinError
    }
}
