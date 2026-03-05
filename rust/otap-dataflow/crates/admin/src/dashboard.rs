// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Dashboard endpoint.
//!
//! Serves a self-contained HTML page at `/dashboard` that auto-refreshes
//! engine-level metrics (RSS, CPU, heap, uptime, core count) by polling
//! the existing `/metrics?format=json` endpoint.

use crate::AppState;
use axum::Router;
use axum::http::header;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;

/// The embedded dashboard HTML (compiled into the binary).
const DASHBOARD_HTML: &str = include_str!("dashboard.html");

/// Routes for the dashboard.
pub(crate) fn routes() -> Router<AppState> {
    Router::new().route("/dashboard", get(dashboard))
}

/// Handler that serves the dashboard page with no-cache headers.
async fn dashboard() -> Response {
    (
        [(header::CACHE_CONTROL, "no-store, no-cache, must-revalidate")],
        Html(DASHBOARD_HTML),
    )
        .into_response()
}
