use axum::{
    extract::Path,
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
};
use include_dir::{include_dir, Dir};

static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/static");
static TEMPLATES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates");

pub async fn serve_index() -> Result<Html<String>, StatusCode> {
    let index_html = TEMPLATES_DIR
        .get_file("index.html")
        .ok_or(StatusCode::NOT_FOUND)?
        .contents_utf8()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(index_html.to_string()))
}

pub async fn serve_static_files(Path(path): Path<String>) -> Result<Response, StatusCode> {
    let file = STATIC_DIR
        .get_file(&path)
        .ok_or(StatusCode::NOT_FOUND)?;

    let content = file.contents();
    let mime_type = mime_guess::from_path(&path).first_or_octet_stream();

    let mut headers = HeaderMap::new();
    headers.insert(
        http::header::CONTENT_TYPE,
        mime_type.as_ref().parse().unwrap(),
    );

    if path.starts_with("css/") || path.starts_with("js/") || path.starts_with("icons/") || path.starts_with("fonts") {
        headers.insert(
            http::header::CACHE_CONTROL,
            "public, max-age=31536000".parse().unwrap(),
        );
    }

    Ok((headers, content).into_response())
}
