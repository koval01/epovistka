use axum::{
    extract::State,
    response::Response,
    Json,
};
use std::sync::Arc;

use crate::{
    handlers::generate::GenerateImageHandler,
    models::generate::{GenerateRequest, GenerateError},
};

pub async fn generate_image(
    State(handler): State<Arc<GenerateImageHandler>>,
    Json(payload): Json<GenerateRequest>,
) -> Result<Response, GenerateError> {
    handler.handle_generate_request(payload).await
}
