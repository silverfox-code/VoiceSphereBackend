// Standard API response format
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{ Deserialize, Serialize};

use crate::ErrorCode;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppResponse<T> {
    pub success: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorDetails>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

pub struct HttpResponse<T> {
    pub status: StatusCode,
    pub body: AppResponse<T>,
}

impl<T: Serialize> HttpResponse<T> {
    pub fn new(status: StatusCode, body: AppResponse<T>) -> Self {
        HttpResponse { status, body }
    }

    pub fn success(status: StatusCode, data: T) -> Self {
        HttpResponse {
            status,
            body: AppResponse {
                success: true,
                data: Some(data),
                message: None,
                error: None,
            },
        }
    }

    pub fn ok(data: T) -> Self {
        Self::success(StatusCode::OK, data)
    }
}

impl HttpResponse<()> {
    pub fn ok_message(message: impl Into<String>) -> HttpResponse<()> {
        HttpResponse {
            status: StatusCode::OK,
            body: AppResponse {
                success: true,
                data: None,
                message: Some(message.into()),
                error: None,
            },
        }
    }

    pub fn error(
        status: StatusCode,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> HttpResponse<()> {
        HttpResponse {
            status,
            body: AppResponse {
                success: false,
                data: None,
                message: None,
                error: Some(ErrorDetails {
                    code: code.into(),
                    message: message.into(),
                    details: None,
                }),
            },
        }
    }

    pub fn error_with_details(
        status: StatusCode,
        code: impl Into<String>,
        message: impl Into<String>,
        details: serde_json::Value,
    ) -> HttpResponse<()> {
        HttpResponse {
            status,
            body: AppResponse {
                success: false,
                data: None,
                message: None,
                error: Some(ErrorDetails {
                    code: code.into(),
                    message: message.into(),
                    details: Some(details),
                }),
            },
        }
    }

    /// HTTP 400 BAD REQUEST
    pub fn bad_request(code: impl Into<String>, message: impl Into<String>) -> HttpResponse<()> {
        Self::error(StatusCode::BAD_REQUEST, code, message)
    }

    /// HTTP 401 UNAUTHORIZED
    pub fn unauthorized(code: impl Into<String>, message: impl Into<String>) -> HttpResponse<()> {
        Self::error(StatusCode::UNAUTHORIZED, code, message)
    }

    /// HTTP 403 FORBIDDEN
    pub fn forbidden(code: impl Into<String>, message: impl Into<String>) -> HttpResponse<()> {
        Self::error(StatusCode::FORBIDDEN, code, message)
    }

    /// HTTP 404 NOT FOUND
    pub fn not_found(code: impl Into<String>, message: impl Into<String>) -> HttpResponse<()> {
        Self::error(StatusCode::NOT_FOUND, code, message)
    }

    /// HTTP 409 CONFLICT
    pub fn conflict(code: impl Into<String>, message: impl Into<String>) -> HttpResponse<()> {
        Self::error(StatusCode::CONFLICT, code, message)
    }

    /// HTTP 422 UNPROCESSABLE ENTITY (for validation errors)
    pub fn validation_error(
        message: impl Into<String>,
        details: serde_json::Value,
    ) -> HttpResponse<()> {
        Self::error_with_details(
            StatusCode::UNPROCESSABLE_ENTITY,
            ErrorCode::ValidationError.to_string(),
            message,
            details,
        )
    }

    /// HTTP 500 INTERNAL SERVER ERROR
    pub fn internal_error(code: impl Into<String>, message: impl Into<String>) -> HttpResponse<()> {
        Self::error(StatusCode::INTERNAL_SERVER_ERROR, code, message)
    }

    /// HTTP 503 SERVICE UNAVAILABLE
    pub fn service_unavailable(message: impl Into<String>) -> HttpResponse<()> {
        Self::error(
            StatusCode::SERVICE_UNAVAILABLE,
            ErrorCode::ServiceUnavailable.to_string(),
            message.into(),
        )
    }
}

impl<T: Serialize> IntoResponse for HttpResponse<T> {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}