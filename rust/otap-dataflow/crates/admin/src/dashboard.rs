// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Embedded admin UI endpoints.
//!
//! Serves the embedded admin UI at `/` and `/dashboard`.
//! Static assets are served from `/static/*`.

use crate::AppState;
use axum::Router;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::http::header;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use include_dir::Dir;

/// Embedded UI files compiled into the binary.
static UI_FILES: Dir<'_> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/ui");
const CACHE_CONTROL_NO_STORE: &str = "no-store, no-cache, must-revalidate";

/// Routes for the embedded UI.
pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/dashboard", get(index))
        .route("/static/{*path}", get(static_asset))
}

/// Handler that serves the UI index page with no-cache headers.
async fn index() -> Response {
    let Some(file) = UI_FILES.get_file("index.html") else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Embedded UI is missing index.html",
        )
            .into_response();
    };

    let Some(index_html) = file.contents_utf8() else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Embedded index.html is not valid UTF-8",
        )
            .into_response();
    };

    (
        [(header::CACHE_CONTROL, CACHE_CONTROL_NO_STORE)],
        Html(index_html),
    )
        .into_response()
}

/// Handler that serves embedded static assets for the UI.
async fn static_asset(Path(path): Path<String>) -> Response {
    let relative_path = path.trim_start_matches('/');
    if relative_path.is_empty() {
        return (StatusCode::NOT_FOUND, "Asset path is empty").into_response();
    }

    let Some(file) = UI_FILES.get_file(relative_path) else {
        return (StatusCode::NOT_FOUND, "Asset not found").into_response();
    };

    let mime = mime_guess::from_path(relative_path).first_or_octet_stream();
    (
        [
            (header::CACHE_CONTROL, CACHE_CONTROL_NO_STORE),
            (header::CONTENT_TYPE, mime.as_ref()),
        ],
        file.contents(),
    )
        .into_response()
}
