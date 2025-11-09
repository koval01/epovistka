use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct GenerateRequest {
    pub name: String,
    pub address: String,
    pub issuer: String,
}

impl GenerateRequest {
    pub fn validate(&self) -> Result<(), GenerateError> {
        if self.name.trim().is_empty() {
            return Err(GenerateError::ValidationError("Name cannot be empty".to_string()));
        }

        if self.address.trim().is_empty() {
            return Err(GenerateError::ValidationError("Address cannot be empty".to_string()));
        }

        if self.issuer.trim().is_empty() {
            return Err(GenerateError::ValidationError("Issuer cannot be empty".to_string()));
        }

        if self.name.len() > 100 {
            return Err(GenerateError::ValidationError("Name is too long".to_string()));
        }

        if self.address.len() > 200 {
            return Err(GenerateError::ValidationError("Address is too long".to_string()));
        }

        if self.issuer.len() > 100 {
            return Err(GenerateError::ValidationError("Issuer is too long".to_string()));
        }

        Ok(())
    }

    pub fn sanitize(&mut self) {
        self.name = self.name.trim().to_string();
        self.address = self.address.trim().to_string();
        self.issuer = self.issuer.trim().to_string();
    }
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct GenerateResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Error, Debug)]
pub enum GenerateError {
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Image generation error: {0}")]
    GenerationError(String),

    #[error("Initialization error: {0}")]
    InitializationError(String),

    #[allow(dead_code)]
    #[error("Invalid input data")]
    InvalidInput,
}

impl IntoResponse for GenerateError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            GenerateError::ValidationError(msg) => (http::StatusCode::BAD_REQUEST, msg),
            GenerateError::GenerationError(msg) => (http::StatusCode::INTERNAL_SERVER_ERROR, msg),
            GenerateError::InitializationError(msg) => (http::StatusCode::INTERNAL_SERVER_ERROR, msg),
            GenerateError::InvalidInput => (http::StatusCode::BAD_REQUEST, "Invalid input data".to_string()),
        };

        let body = axum::Json(serde_json::json!({
            "error": error_message,
            "success": false
        }));

        (status, body).into_response()
    }
}
