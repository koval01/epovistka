use axum::{
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use tracing::info;

use crate::{
    models::generate::{GenerateRequest, GenerateError},
    services::image_generator::ImageGenerator,
};

#[derive(Clone)]
pub struct GenerateImageHandler {
    image_generator: Arc<ImageGenerator>,
}

impl GenerateImageHandler {
    pub fn new() -> Result<Self, GenerateError> {
        let image_generator = ImageGenerator::new()
            .map_err(|e| GenerateError::InitializationError(e.to_string()))?;

        Ok(Self {
            image_generator: Arc::new(image_generator),
        })
    }

    pub async fn handle_generate_request(
        &self,
        mut request: GenerateRequest,
    ) -> Result<Response, GenerateError> {
        request.sanitize();
        request.validate()?;

        info!("Processing generate request for: {}", request.name);

        let image_data = self.image_generator.generate_image(&request).await?;

        let headers = [
            (http::header::CONTENT_TYPE, "image/png"),
            (
                http::header::CONTENT_DISPOSITION,
                "inline; filename=\"povistka.png\"",
            ),
            (
                http::header::CACHE_CONTROL,
                "no-cache, no-store, must-revalidate",
            ),
        ];

        Ok((headers, image_data).into_response())
    }
}
