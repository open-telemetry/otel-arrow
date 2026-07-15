// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Debug endpoints.
//!
//! - /api/v1/debug/pprof/heap - dump profile of unfreed heap allocations. This fails unless
//!   jemalloc is the active allocator and profiling is enabled.
//!    
//!   

use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::Response;
use axum::routing::get;

use std::panic::AssertUnwindSafe;

use crate::AppState;

pub(crate) fn routes() -> Router<AppState> {
    let router = Router::new().route("/debug/pprof/heap", get(get_heap_profile));

    router
}

async fn get_heap_profile(
    State(_state): State<AppState>,
) -> Result<Response, (StatusCode, String)> {
    // Accessing `PROF_CTL` can panic if jemalloc is not the active allocator or was compiled w/out profiling
    // feature. Catch this panic so we return a proper HTTP error instead.
    let prof_ctl = std::panic::catch_unwind(AssertUnwindSafe(|| jemalloc_pprof::PROF_CTL.as_ref()));
    let mut prof_ctl = match prof_ctl {
        Ok(Some(prof_ctl)) => prof_ctl,
        Ok(None) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Memory Profiling not activated".into(),
            ))
        }
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Heap profiling not available \
                 (jemalloc with profiling support is not the active allocator)."
                    .into(),
            ));
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
            let resp = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/x-protobuf")
                .body(body)
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Could not dump heap pprof: {e}"),
                    )
                })?;

            Ok(resp)
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Could not dump heap pprof: {e}"),
            ));
        }
    }
}
