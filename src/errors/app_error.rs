use crate::{ErrorCode, HttpResponse};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use jsonwebtoken::errors::Error as JwtError;
use serde::Serialize;
use serde_json;
use std::fmt;

#[derive(Debug, Serialize)]
pub enum AppError {
    // Database errors
    DatabaseError(String),

    // Authentication errors
    Unauthorized(String),
    InvalidToken,
    TokenExpired,

    // Resource errors
    NotFound(String),

    // Validation errors
    ValidationError(String, Option<serde_json::Value>),

    // Business logic errors
    Forbidden(String),
    Conflict(String),

    // System errors
    InternalError(String),
}

impl std::error::Error for AppError {}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::DatabaseError(msg) => write!(f, "Database Error: {}", msg),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::InvalidToken => write!(f, "Invalid Token"),
            AppError::TokenExpired => write!(f, "Token Expired"),
            AppError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            AppError::ValidationError(msg, _) => write!(f, "Validation Error: {}", msg),
            AppError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            AppError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            AppError::InternalError(msg) => write!(f, "Internal Error: {}", msg),
        }
    }
}

// Helper conversions to make '?' work in handlers
impl From<String> for AppError {
    fn from(error: String) -> Self {
        AppError::InternalError(error)
    }
}
impl From<JwtError> for AppError {
    fn from(error: JwtError) -> Self {
        AppError::Unauthorized(format!("JWT Error: {}", error))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message, details) = match self {
            AppError::DatabaseError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::DatabaseError,
                msg,
                None,
            ),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, ErrorCode::Unauthorized, msg, None),
            AppError::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                ErrorCode::InvalidToken,
                "Invalid token".to_string(),
                None,
            ),
            AppError::TokenExpired => (
                StatusCode::UNAUTHORIZED,
                ErrorCode::TokenExpired,
                "Token expired".to_string(),
                None,
            ),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, ErrorCode::NotFound, msg, None),
            AppError::ValidationError(msg, details) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ErrorCode::ValidationError,
                msg,
                details,
            ),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, ErrorCode::Forbidden, msg, None),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, ErrorCode::Conflict, msg, None),
            AppError::InternalError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::InternalError,
                msg,
                None,
            ),
        };

        if let Some(details) = details {
            HttpResponse::error_with_details(status, code.to_string(), message, details)
                .into_response()
        } else {
            HttpResponse::error(status, code.to_string(), message).into_response()
        }
    }
}
