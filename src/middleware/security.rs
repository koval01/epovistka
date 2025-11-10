use tower_http::set_header::SetResponseHeaderLayer;
use axum::http::HeaderValue;

#[allow(dead_code)]
pub fn security_headers() -> SetResponseHeaderLayer<HeaderValue> {
    SetResponseHeaderLayer::overriding(
        axum::http::header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache, no-store, must-revalidate"),
    )
}