use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::fmt;

// Define custom error types
#[derive(Debug)]
pub enum AppError {
    // Internal server errors
    Internal(anyhow::Error),
    
    // Database related errors
    DatabaseError(anyhow::Error),
    
    // Validation errors
    ValidationError(String),
    
    // Not found errors
    NotFound(String),
    
    // Authentication errors
    Unauthorized(String),
    
    // Authorization errors
    Forbidden(String),
    
    // Bad request errors
    BadRequest(String),
}

// Implement Display for AppError
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Internal(e) => write!(f, "Internal server error: {}", e),
            AppError::DatabaseError(e) => write!(f, "Database error: {}", e),
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
        }
    }
}

// Implement Error trait for AppError
impl std::error::Error for AppError {}

// Convert AppError into Response
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Internal(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal server error: {}", e),
            ),
            AppError::DatabaseError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error occurred: {}", e),
            ),
            AppError::ValidationError(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Validation error: {}", msg),
            ),
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                format!("Not found: {}", msg),
            ),
            AppError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                format!("Unauthorized: {}", msg),
            ),
            AppError::Forbidden(msg) => (
                StatusCode::FORBIDDEN,
                format!("Forbidden: {}", msg),
            ),
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                format!("Bad request: {}", msg),
            ),
        };

        // Create a JSON response with error details
        let body = Json(json!({
            "success": false,
            "error": {
                "message": error_message,
                "code": status.as_u16()
            }
        }));

        (status, body).into_response()
    }
}

// Implement From traits for common error conversions
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err)
    }
}

impl From<diesel::result::Error> for AppError {
    fn from(err: diesel::result::Error) -> Self {
        AppError::DatabaseError(err.into())
    }
}

// Example usage in route handlers
pub async fn example_handler() -> Result<Json<serde_json::Value>, AppError> {
    // Example of validation error
    if false {
        return Err(AppError::ValidationError("Invalid input".to_string()));
    }

    // Example of not found error
    if false {
        return Err(AppError::NotFound("Resource not found".to_string()));
    }

    // Example of successful response
    Ok(Json(json!({
        "success": true,
        "data": "Operation completed successfully"
    })))
}

// Helper function to create validation errors
pub fn validation_err(msg: impl Into<String>) -> AppError {
    AppError::ValidationError(msg.into())
}

// Helper function to create not found errors
pub fn not_found_err(msg: impl Into<String>) -> AppError {
    AppError::NotFound(msg.into())
}

// Helper function to create unauthorized errors
pub fn unauthorized_err(msg: impl Into<String>) -> AppError {
    AppError::Unauthorized(msg.into())
}

// Helper function to create forbidden errors
pub fn forbidden_err(msg: impl Into<String>) -> AppError {
    AppError::Forbidden(msg.into())
}

// Helper function to create bad request errors
pub fn bad_request_err(msg: impl Into<String>) -> AppError {
    AppError::BadRequest(msg.into())
}