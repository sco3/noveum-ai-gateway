use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::{io, convert::Infallible};
use http::status::InvalidStatusCode;

#[derive(Debug)]
pub enum AppError {
    ReqwestError(reqwest::Error),
    IoError(io::Error),
    AxumError(axum::Error),
    InvalidMethod,
    InvalidStatus(InvalidStatusCode),
    InvalidHeader,
    UnsupportedProvider,
    MissingApiKey,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::ReqwestError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
            }
            AppError::IoError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
            }
            AppError::AxumError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
            }
            AppError::InvalidMethod => {
                (StatusCode::BAD_REQUEST, "Invalid HTTP method").into_response()
            }
            AppError::InvalidStatus(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Invalid status code received").into_response()
            }
            AppError::InvalidHeader => {
                (StatusCode::BAD_REQUEST, "Invalid header value").into_response()
            }
            AppError::UnsupportedProvider => {
                (StatusCode::BAD_REQUEST, "Unsupported provider").into_response()
            }
            AppError::MissingApiKey => {
                (StatusCode::UNAUTHORIZED, "Missing or invalid API key").into_response()
            }
        }
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::ReqwestError(err)
    }
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::IoError(err)
    }
}

impl From<axum::Error> for AppError {
    fn from(err: axum::Error) -> Self {
        AppError::AxumError(err)
    }
}

impl From<Infallible> for AppError {
    fn from(_: Infallible) -> Self {
        unreachable!("Infallible error cannot occur")
    }
}

impl From<InvalidStatusCode> for AppError {
    fn from(err: InvalidStatusCode) -> Self {
        AppError::InvalidStatus(err)
    }
} 