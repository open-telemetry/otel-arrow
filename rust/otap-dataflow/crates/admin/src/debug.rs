// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Debug endpoints.
//!
//! - /api/v1/debug/pprof/heap - dump profile of unfreed heap allocations
//!   (only returns 200 on linux with jemalloc allocator otherwise returns 500)

use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::Response;
use axum::routing::get;

use crate::AppState;

pub(crate) fn routes() -> Router<AppState> {
    let router = Router::new().route("/debug/pprof/heap", get(get_heap_profile));

    router
}

async fn get_heap_profile(State(_state): State<AppState>) -> Result<Response, (StatusCode, String)> {
    // TODO fix how this can panic if the jemalloc profiling feature not enabled.
    let mut prof_ctl = match jemalloc_pprof::PROF_CTL.as_ref() {
        Some(prof_ctl) => prof_ctl,
        None => {
            // TODO no panic
            panic!("no pprof ctl");
        }
    }
    .lock()
    .await;

    if !prof_ctl.activated() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Memory Profiling not activated".into(),
        ));
    }

    match prof_ctl.dump_pprof() {
        Ok(pprof) => {
            let body = Body::from(pprof);
            Ok(
                Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "application/x-protobuf")
                    .body(body)
                    .unwrap(), // TODO no unwrap
            )
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Could not dump heap pprof: {e}"),
            ));
        }
    }
}
