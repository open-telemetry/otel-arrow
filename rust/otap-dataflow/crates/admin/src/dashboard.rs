// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Embedded admin UI endpoints.
//!
//! Serves the embedded admin UI at `/` and `/dashboard`.
//! Static assets are served from `/static/*`.

use crate::AppState;
use axum::Router;
use axum::extract::Path;
use axum::http::header;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use include_dir::Dir;

/// Embedded UI files compiled into the binary.
static UI_FILES: Dir<'_> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/ui");
// RFC 9111 section 5.2.2 - <https://www.rfc-editor.org/rfc/rfc9111#section-5.2.2>
// Admin UI responses carry live operational metadata. `no-store` is the
// strongest directive and forbids any caching. `no-cache` and
// `must-revalidate` are included as defensive coverage for older proxies.
const CACHE_CONTROL_NO_STORE: &str = "no-store, no-cache, must-revalidate";
// W3C Content Security Policy Level 3 - <https://www.w3.org/TR/CSP3/>
// Restrict resource loading and execution to same-origin assets. This reduces
// XSS impact, blocks plugin/object execution, prevents clickjacking framing,
// and disallows <base> URL rewriting attacks. `frame-ancestors 'none'` is the
// modern replacement for X-Frame-Options (see below).
const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self' data:; connect-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'";
// WHATWG Fetch Standard - <https://fetch.spec.whatwg.org/#x-content-type-options-header>
// Instructs the browser to honor the declared Content-Type and not MIME-sniff,
// preventing responses from being misinterpreted as executable script or HTML.
const X_CONTENT_TYPE_OPTIONS_NO_SNIFF: &str = "nosniff";
// RFC 7034 (Informational, deprecated) - <https://www.rfc-editor.org/rfc/rfc7034>
// Legacy clickjacking protection kept as a fallback for browsers that do not
// fully implement CSP's `frame-ancestors` directive. When both headers are
// present, `frame-ancestors` takes precedence in modern browsers.
const X_FRAME_OPTIONS_DENY: &str = "DENY";
// W3C Referrer Policy - <https://www.w3.org/TR/referrer-policy/>
// `no-referrer` strips the Referer header entirely so admin endpoint URLs
// do not leak to third parties through outbound links.
const REFERRER_POLICY_NO_REFERRER: &str = "no-referrer";

// Build a consistent hardened header set for all UI/static responses.
fn build_ui_headers(content_type: Option<&str>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let _ = headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(CACHE_CONTROL_NO_STORE),
    );
    let _ = headers.insert(
        header::HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static(CONTENT_SECURITY_POLICY),
    );
    let _ = headers.insert(
        header::HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static(X_CONTENT_TYPE_OPTIONS_NO_SNIFF),
    );
    let _ = headers.insert(
        header::HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static(X_FRAME_OPTIONS_DENY),
    );
    let _ = headers.insert(
        header::HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static(REFERRER_POLICY_NO_REFERRER),
    );
    if let Some(value) = content_type.and_then(|v| HeaderValue::from_str(v).ok()) {
        let _ = headers.insert(header::CONTENT_TYPE, value);
    }
    headers
}

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

    let live_logs_ws = if cfg!(feature = "live-logs-ws") {
        "true"
    } else {
        "false"
    };
    let index_html = index_html.replace("__LIVE_LOGS_WS__", live_logs_ws);

    let headers = build_ui_headers(Some("text/html; charset=utf-8"));
    (headers, Html(index_html)).into_response()
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
    let headers = build_ui_headers(Some(mime.as_ref()));
    (headers, file.contents()).into_response()
}
